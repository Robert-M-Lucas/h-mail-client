use h_mail::interface::shared::PowClassification;
use std::error::Error;
use h_mail::interface::get_emails::{GetEmails, GetEmailsEmail, GetEmailsResponse};
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

pub async fn get_emails(
    user: &str,
    server: &str
) -> Result<Vec<GetEmailsEmail>, Box<dyn Error>> {
    let client = Client::new();

    let params = GetEmails::new(user.to_string(), -1);

    println!("\tSending emails request");
    let response = client
        .get(format!("http://{server}:8081/get_emails"))
        .query(&params)
        .send()
        .await?;

    println!("\tWaiting for response");
    let body = response.text().await?;
    let emails_response = serde_json::from_str::<GetEmailsResponse>(&body)?;
    let Some(emails_response) = emails_response.get_emails() else {
        return Err(Box::new(HMailErr("User does not exist".to_string())));
    };

    Ok(emails_response)
}