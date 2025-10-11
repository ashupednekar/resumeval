use std::fmt::{self, Display};

use super::{SendEmail, send_email};

pub struct AuthnCodeTemplate<'a> {
    pub name: &'a str,
    pub code: &'a str,
}

impl<'a> Display for AuthnCodeTemplate<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let html_template = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Verify Your Email</title>
                <style>
                    body {{
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
                        line-height: 1.6;
                        margin: 0;
                        padding: 0;
                        background-color: #f9fafb;
                    }}
                    .container {{
                        max-width: 600px;
                        margin: 0 auto;
                        padding: 20px;
                    }}
                    .code-container {{
                        text-align: center;
                        margin: 40px 0;
                        padding: 30px;
                        background-color: #ffffff;
                        border-radius: 8px;
                        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
                    }}
                    .verification-code {{
                        font-size: 32px;
                        font-weight: bold;
                        letter-spacing: 4px;
                        color: #059669;
                        margin: 20px 0;
                    }}
                    .message {{
                        color: #4b5563;
                        font-size: 14px;
                        margin: 20px 0;
                    }}
                    .warning {{
                        color: #dc2626;
                        font-size: 12px;
                        margin-top: 20px;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="code-container">
                        <h2 style="color: #111827; margin: 0;">Your Verification Code</h2>
                        <div class="verification-code">{}</div>
                        <p class="message">
                            This code is for one-time use and will expire in 10 minutes.<br>
                            You'll receive a new code if this one expires.
                        </p>
                        <p class="warning">
                            ⚠️ Do not share this code with anyone.<br>
                            Our team will never ask for this code.
                        </p>
                    </div>
                </div>
            </body>
            </html>
            "#,
            self.code
        );
        write!(f, "{}", html_template)
    }
}

impl<'a> SendEmail for AuthnCodeTemplate<'a> {
    fn send(&self, email: &str) -> crate::prelude::Result<()> {
        send_email(
            &email,
            "Here's your LWS authentication code",
            &format!("{}", &self),
            true,
        )?;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::prelude::Result;

    #[tokio::test]
    #[traced_test]
    async fn test_send_mail_template() -> Result<()> {
        AuthnCodeTemplate {
            name: "Ashu",
            code: "394u93",
        }
        .send("ashupednekar49@gmail.com")?;
        Ok(())
    }
}
