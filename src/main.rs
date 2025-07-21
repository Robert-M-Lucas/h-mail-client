mod pow;

use h_mail::interface::check_pow::{CheckPow, CheckPowStatus};
use h_mail::interface::pow_request::{PowRequest, PowResponse};
use h_mail::interface::send_email::{SendEmail, SendEmailStatus};
use h_mail::shared::big_uint_to_base64;
use reqwest::Client;
use rsa::BigUint;
use sha2::{Digest, Sha256};
use tokio::time::Instant;
use crate::pow::solve_challenge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    send_email("robert01.lucas", "Hello, world!").await?;
    Ok(())
}

async fn send_email(destination: &str, email: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let params = PowRequest::new(destination.to_string());

    println!("Sending POW request");
    let response = client
        .get("http://localhost:8081/pow_request")
        .query(&params)
        .send()
        .await?;

    println!("Waiting for response");
    let body = response.text().await?;
    let pow_response = serde_json::from_str::<PowResponse>(&body).unwrap().get().unwrap().decode().unwrap();

    println!("POW received. Calculating hash");
    let mut s = Sha256::new();
    s.update(email.as_bytes());
    let challenge_hash = BigUint::from_bytes_le(&s.finalize());
    let iters = pow_response.policy().accepted();

    println!("Calculating POW with accepted policy iters: {iters}");
    let start = Instant::now();
    let result = solve_challenge(challenge_hash.clone(), pow_response.pow_token().token(), iters);
    let elapsed = start.elapsed();
    println!("Solved POW in {elapsed:?}");

    let params = SendEmail::new(
        email.to_string(),
        iters,
        big_uint_to_base64(pow_response.pow_token().token()),
        big_uint_to_base64(&result),
        destination.to_string()
    );

    println!("Sending email");
    let response = client
        .post("http://localhost:8081/send_email")
        .json(&params)
        .send()
        .await?;

    println!("Waiting for response...");
    let body = response.text().await?;
    let send_email_response = serde_json::from_str::<SendEmailStatus>(&body).unwrap();

    println!("Result: {send_email_response:#?}");

    Ok(())
}
