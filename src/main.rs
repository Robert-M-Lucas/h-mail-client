mod pow;
mod send_email;
mod get_emails;

use h_mail::interface::shared::PowClassification;
use sha2::Digest;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{stdin, stdout, Write};
use crate::get_emails::get_emails;

struct HMailErr(String);

impl Debug for HMailErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
async fn main() -> Result<(), Box<dyn Error>> {
    loop {
        println!();
        let mut selection = 0;
        while !(1..=2).contains(&selection) {
            print!("Get emails [1] or send email [2]: ");
            let input = read_line();
            let Ok(x) = input.parse::<i32>() else {
                continue;
            };
            selection = x;
        }
        
        if selection == 1 {
            print!("Enter user: ");
            let user = read_line();
            print!("Enter server: ");
            let server = read_line();
            let emails = match get_emails(&user, &server).await {
                Ok(e) => e,
                Err(e) => {
                    println!("Error reading emails: {e}");
                    continue
                }
            };
            
            println!("Emails:");
            if emails.is_empty() {
                println!("[None]");
            }
            for email in emails {
                println!("{} [{}]: {}", email.source(), email.pow_classification().to_ident(), email.email())
            }
        }
        else {
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
            match send_email::send_email(&user, &server, &message, classification).await {
                Ok(_) => println!("Sent!"),
                Err(e) => println!("Error sending message: {e}"),
            }
        }
    }
    
    
}

