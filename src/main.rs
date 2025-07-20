use h_mail::interface::pow_request::{PowRequest, PowResponse};
use reqwest::Client;

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
    let pow_response: Result<PowResponse, _> = serde_json::from_str(&body);

    println!(
        "Response: {:#?}",
        pow_response.unwrap().get().unwrap().decode().unwrap()
    );

    Ok(())
}
