use h_mail::interface::shared::PowClassification;
use std::error::Error;
use reqwest::Client;
use h_mail::interface::pow_request::{PowRequest, PowResponse};
use sha2::Sha256;
use rsa::BigUint;
use tokio::time::Instant;
use h_mail::interface::send_email::{SendEmail, SendEmailStatus};
use h_mail::shared::big_uint_to_base64;
use rsa::signature::digest::Digest;
use crate::HMailErr;
use crate::pow::solve_challenge;

pub async fn send_email(
    user: &str,
    server: &str,
    message: &str,
    classification: PowClassification,
) -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let params = PowRequest::new(user.to_string());

    println!("\tSending POW request");
    let response = client
        .get(format!("http://{server}:8081/pow_request"))
        .query(&params)
        .send()
        .await?;

    println!("\tWaiting for response");
    let body = response.text().await?;
    let Some(pow_response) = serde_json::from_str::<PowResponse>(&body)?.get() else {
        return Err(Box::new(HMailErr("User does not exist".to_string())));
    };

    let pow_response = pow_response.decode()?;

    println!("\tPOW received. Calculating hash");
    let mut s = Sha256::new();
    s.update(message.as_bytes());
    let challenge_hash = BigUint::from_bytes_le(&s.finalize());

    let iters = match classification {
        PowClassification::Minimum => pow_response.policy().minimum(),
        PowClassification::Accepted => pow_response.policy().accepted(),
        PowClassification::Personal => pow_response.policy().personal(),
    };

    println!(
        "\tCalculating POW with `{}` policy iters: {iters}",
        classification.to_ident()
    );
    let start = Instant::now();
    let result = solve_challenge(
        challenge_hash.clone(),
        pow_response.pow_token().token(),
        iters,
    );
    let elapsed = start.elapsed();
    println!("\tSolved POW in {elapsed:?}");

    let params = SendEmail::new(
        "test_client@nothing.com".to_string(),
        message.to_string(),
        iters,
        big_uint_to_base64(pow_response.pow_token().token()),
        big_uint_to_base64(&result),
        user.to_string(),
    );

    println!("\tSending email");
    let response = client
        .post(format!("http://{server}:8081/send_email"))
        .json(&params)
        .send()
        .await?;

    println!("\tWaiting for response...");
    let body = response.text().await?;
    let send_email_response = serde_json::from_str::<SendEmailStatus>(&body)?;

    println!("\tResult: {send_email_response:#?}");

    Ok(())
}