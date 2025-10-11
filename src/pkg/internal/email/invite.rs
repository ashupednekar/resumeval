use crate::conf::settings;
use std::fmt::{self, Display};

use super::{SendEmail, send_email};

#[derive(Debug)]
pub struct ShowInvite {
    pub inviter: String,
    pub project_name: String,
    pub project_description: String,
    pub invite_id: String,
}

impl Display for ShowInvite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let html_template = format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Project Invitation</title>
                <style>
                    body {{
                        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
                        line-height: 1.6;
                        color: #333;
                        margin: 0;
                        padding: 0;
                    }}
                    .container {{
                        max-width: 600px;
                        margin: 0 auto;
                        padding: 20px;
                    }}
                    .header {{
                        text-align: center;
                        padding: 20px 0;
                        background-color: #0d9488;
                        color: white;
                    }}
                    .content {{
                        padding: 20px;
                        background-color: #ffffff;
                    }}
                    .button {{
                        display: inline-block;
                        padding: 12px 24px;
                        background-color: #0d9488;
                        color: white;
                        text-decoration: none;
                        border-radius: 6px;
                        margin: 20px 0;
                    }}
                    .footer {{
                        text-align: center;
                        padding: 20px;
                        color: #666;
                        font-size: 14px;
                    }}
                    .project-info {{
                        background-color: #f3f4f6;
                        padding: 15px;
                        border-radius: 6px;
                        margin: 20px 0;
                    }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>Project Invitation</h1>
                    </div>
                    <div class="content">
                        <p>Hello,</p>
                        <p>{} has invited you to join their project on Lite Web Services.</p>
                        
                        <div class="project-info">
                            <h2>{}</h2>
                            <p>{}</p>
                        </div>
            
                        <p>Click the button below to accept this invitation:</p>
                        
                        <div style="text-align: center;">
                            <a href="{}/project/accept?invite_code={}" class="button">Accept Invitation</a>
                        </div>
            
                        <p>If you did not expect this invitation, you can safely ignore this email.</p>
                    </div>
                    <div class="footer">
                        <p>Lite Web Services</p>
                        <p>&copy; 2023 All rights reserved</p>
                    </div>
                </div>
            </body>
            </html> 
            "#,
            self.inviter,
            self.project_name,
            self.project_description,
            &settings.base_url,
            self.invite_id
        );
        write!(f, "{}", html_template)
    }
}

impl SendEmail for ShowInvite {
    fn send(&self, email: &str) -> crate::prelude::Result<()> {
        send_email(
            &email,
            "You've got a project invite on LWS",
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

    #[test]
    #[traced_test]
    fn test_send_mail_template() -> Result<()> {
        ShowInvite {
            inviter: "Ashu".into(),
            invite_id: "wmklwmfklmflwmf".into(),
            project_name: "one".into(),
            project_description: "nkknlsknflksnf".into(),
        }
        .send("ashupednekar49@gmail.com")?;
        Ok(())
    }
}
