mod pow;

use h_mail::interface::check_pow::{CheckPow, CheckPowStatus};
use h_mail::interface::pow_request::{PowRequest, PowResponse};
use h_mail::shared::big_uint_to_base64;
use reqwest::Client;
use rsa::BigUint;
use sha2::{Digest, Sha256};
use tokio::time::Instant;
use crate::pow::solve_challenge;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let params = PowRequest::new(String::from("robert01.lucas"));

    println!("Sending...");

    let response = client
        .get("http://localhost:8081/pow_request")
        .query(&params)
        .send()
        .await?;

    println!("Waiting for response...");
    let body = response.text().await?;
    let pow_response = serde_json::from_str::<PowResponse>(&body).unwrap().get().unwrap().decode().unwrap();

    println!(
        "Response: {:#?}",
        pow_response
    );

    let challenge = "challenge";
    
    let mut s = Sha256::new();
    s.update(challenge.as_bytes());
    let challenge_hash = BigUint::from_bytes_le(&s.finalize());
    let iters = pow_response.policy().accepted();
    println!("Using accepted policy iters: {iters}");
    
    let start = Instant::now();
    let result = solve_challenge(challenge_hash.clone(), pow_response.pow_token().token(), iters);
    let elapsed = start.elapsed();
    println!("Solved in {elapsed:?}");

    let params = CheckPow::new(
        big_uint_to_base64(pow_response.pow_token().token()),
        iters,
        big_uint_to_base64(&challenge_hash),
        big_uint_to_base64(&result),
    );
    
    println!("Sending verification...");
    let response = client
        .get("http://localhost:8081/check_pow")
        .query(&params)
        .send()
        .await?;
    
    println!("Waiting for response...");
    let body = response.text().await?;
    let pow_check_response = serde_json::from_str::<CheckPowStatus>(&body).unwrap();
    
    println!("{pow_check_response:?}");
    
    Ok(())
}
