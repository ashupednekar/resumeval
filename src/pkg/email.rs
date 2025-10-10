use mail_send::{mail_builder::MessageBuilder, SmtpClientBuilder};
use standard_error::{Interpolate, StandardError};

use crate::conf::settings;

pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<(), StandardError> {
    tracing::info!("email body: {}", &body);
    return Ok(());
    let message = MessageBuilder::new()
        .from(settings.from_email.clone())
        .to(vec![to.to_string()])
        .subject(subject.to_string())
        //.html_body("<h1>Hello, world!</h1>")
        .text_body(body.to_string());

    tracing::debug!(
        "{:?}",
        (settings.from_email.clone(), settings.smtp_pass.clone())
    );

    SmtpClientBuilder::new(settings.smtp_server.clone(), settings.smtp_port)
        .implicit_tls(false)
        .credentials((settings.from_email.clone(), settings.smtp_pass.clone()))
        .connect()
        .await
        .unwrap() //map_err(|e| StandardError::new("ER-SMTP").interpolate_err(e.to_string()))?
        .send(message)
        .await
        .map_err(|e| StandardError::new("ER-SMTP").interpolate_err(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use standard_error::StandardError;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_send_email() -> Result<(), StandardError> {
        let to = "ashupednekar49@gmail.com";
        let subject = "Test Email from Rust";
        let body = "Hello! This is a test email sent from a Rust application.";
        send_email(to, subject, body).await?;
        Ok(())
    }
}
