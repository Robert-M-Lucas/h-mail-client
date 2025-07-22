mod pow;

use crate::pow::solve_challenge;
use h_mail::interface::pow_request::{PowRequest, PowResponse};
use h_mail::interface::send_email::{SendEmail, SendEmailStatus};
use h_mail::interface::shared::PowClassification;
use h_mail::shared::big_uint_to_base64;
use reqwest::Client;
use rsa::BigUint;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Write, stdin, stdout};
use tokio::time::Instant;

struct HMailErr(String);

impl Debug for HMailErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for HMailErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for HMailErr {}

fn read_line() -> String {
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!("Enter user: ");
        let user = read_line();
        print!("Enter server: ");
        let server = read_line();
        print!("Enter message: ");
        let message = read_line();

        let mut selection = 0;
        while !(1..=3).contains(&selection) {
            print!("Enter POW class [1: MIN, 2: ACCEPT, 3: PERSONAL]: ");
            let input = read_line();
            let Ok(x) = input.parse::<i32>() else {
                continue;
            };
            selection = x;
        }

        let classification = match selection {
            1 => PowClassification::Minimum,
            2 => PowClassification::Accepted,
            3 => PowClassification::Personal,
            _ => unreachable!(),
        };

        println!("Sending...");
        match send_email(&user, &server, &message, classification).await {
            Ok(_) => println!("Sent!"),
            Err(e) => println!("Error sending message: {e}"),
        }
        println!();
    }
}

async fn send_email(
    user: &str,
    server: &str,
    message: &str,
    classification: PowClassification,
) -> Result<(), Box<dyn std::error::Error>> {
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
