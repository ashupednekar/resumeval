use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub mod authtoken;
pub mod invite;

use crate::{conf::settings, prelude::Result};

pub trait SendEmail {
    fn send(&self, email: &str) -> Result<()>;
}

pub fn send_email(email: &str, subject: &str, body: &str, is_html: bool) -> Result<()> {
    let (name, _) = email.split_once("@").unwrap_or(("unknown", ""));
    tracing::debug!("name: {}", &name);
    let name = name.to_string();
    let email = email.to_string();
    let subject = subject.to_string();
    let body = body.to_string();
    let is_html = is_html;
    tracing::debug!("sending email to {}", &email);
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            let content_type = if is_html {
                ContentType::TEXT_HTML
            } else {
                ContentType::TEXT_PLAIN
            };

            let email = Message::builder()
                .from(
                    format!("{} <{}>", &settings.service_name, &settings.from_email)
                        .parse()
                        .unwrap(),
                )
                .to(format!("{} <{}>", &name, &email).parse().unwrap())
                .subject(subject)
                .header(content_type)
                .body(body)
                .unwrap();

            let creds = Credentials::new(settings.smtp_user.clone(), settings.smtp_pass.clone());

            let mailer = SmtpTransport::relay(&settings.smtp_server)
                .unwrap()
                .credentials(creds)
                .build();

            mailer.send(&email)
        })
        .await;

        match result {
            Ok(Ok(_)) => println!("Email sent successfully!"),
            Ok(Err(e)) => eprintln!("Could not send email: {e:?}"),
            Err(e) => eprintln!("Task failed to execute: {e:?}"),
        }
    });
    Ok(())
}
