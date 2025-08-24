use std::io::Cursor;
use sha2::{Sha256, Digest};
use lettre::Message;
use tiny_http::{Server, Response, Header, StatusCode};
use lettre::{message::MultiPart, SmtpTransport, Transport, message::Mailbox};
use clap::Parser;

mod error;
use error::Error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
   /// Address to listen on
   #[clap(short, long, value_parser, default_value = "localhost:8000")]
   addr: String,

   /// Hashed api key (with sha256)
   #[clap(long, short='k', value_parser)]
   api_key: Vec<String>,
}

/// Handle a single HTTP request
fn handle_request(headers: &[Header], body: String, hashed_api_keys: &[String]) -> Result<(), Error> {
    // Extract parameters
    let mut to = None;
    let mut from = None;
    let mut subject = None;
    let mut reply_to = None;
    let mut api_key = None;
    for header in headers {
        match header.field.as_str().to_ascii_lowercase().as_str() {
            "to" => to = Some(header.value.to_string()),
            "from" => from = Some(header.value.to_string()),
            "subject" => subject = Some(header.value.to_string()),
            "reply-to" => reply_to = Some(header.value.to_string()),
            "api-key" => api_key = Some(header.value.to_string()),
            _ => {}
        }
    }

    // Check api key
    match api_key {
        Some(api_key) => {
            let mut hasher = Sha256::new();
            hasher.update(api_key);
            let hashed_api_key = hasher.finalize();
            let hashed_api_key = format!("{:x}", hashed_api_key);
            if !hashed_api_keys.contains(&hashed_api_key) {
                return Err(Error::Unauthorized(hashed_api_key));
            }
        }
        None => return Err(Error::MissingApiKey),
    }

    // Parse and validate parameters
    let to = to.map(|to| to.parse::<Mailbox>()).transpose()?.ok_or(Error::MissingTo)?;
    let from = from.map(|from| from.parse::<Mailbox>()).transpose()?.ok_or(Error::MissingFrom)?;
    let reply_to = reply_to.map(|reply_to| reply_to.parse::<Mailbox>()).transpose()?;
    let subject = subject.ok_or(Error::MissingSubject)?;

    // Build the message
    let mut email = Message::builder()
        .from(from.clone())
        .to(to.clone())
        .subject(subject);
    if let Some(reply_to) = reply_to {
        email = email.reply_to(reply_to);
    }
    let email = if let Some(idx) = body.find("\n-----END-TEXT-BEGIN-HTML-----\n") {
        let body_text = &body[..idx];
        let body_html = &body[idx + 31..];
        email.multipart(MultiPart::alternative_plain_html(
            String::from(body_text),
            String::from(body_html),
        ))?
    } else {
        email.body(body.clone())?
    };

    // Send the message
    let mailer = SmtpTransport::unencrypted_localhost();
    mailer.send(&email)?;

    // Log
    println!("Sent an email from {from} to {to} ({} bytes)", body.len());

    Ok(())
}

fn main() {
    // Read cli arguments
    let cli = Cli::parse();
    let hashed_api_keys = cli.api_key.as_slice();
    for hashed_api_key in hashed_api_keys {
        if hashed_api_key.len() != 64 {
            eprintln!("WARNING: Invalid api key {hashed_api_key:?} (size should be 64)");
        }
    }

    // Boot server
    let server = Server::http(cli.addr.clone()).expect("Failed to launch server");
    println!("Listening on {}", cli.addr);

    // Listen for connections
    for mut request in server.incoming_requests() {
        // Redirect root to github
        if request.url() == "/" {
            let location = Header::from_bytes(&b"Location"[..], &b"https://github.com/Les-Schtroumpfs/http-mailer"[..]).unwrap();
            let _ = request.respond(Response::new_empty(StatusCode(301)).with_header(location));
            continue;
        }

        // Check path
        if request.url() != "/send-email" {
            let _ = request.respond(Response::new_empty(StatusCode(404)).with_data(Cursor::new("This is an http mailer server"), Some(29)));
            continue;
        }

        // Read body
        let mut body = String::new();
        match request.as_reader().read_to_string(&mut body) {
            Ok(_) => (),
            Err(_) => {
                let _ = request.respond(Response::new_empty(StatusCode(400)).with_data(Cursor::new("Failed to read request body"), Some(27)));
                continue;
            },
        }

        // Handle requests
        let res = match handle_request(request.headers(), body, hashed_api_keys) {
            Ok(_) => request.respond(Response::new_empty(StatusCode(200))),
            Err(e) => {
                if e.status_code() != 401 {
                    eprintln!("ERROR: {}", e.description());
                }
                request.respond(e.into())
            },
        };
        if let Err(e) = res {
            eprintln!("ERROR: Failed to respond {}", e);
        }
    }
}
