use crate::config::models::{EmailAction, EmailConfig};
use crate::core::wacraft::models::Contact;
use anyhow::{Context, Result};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::fs;

/// Sends a reminder email to a contact based on a rule.
pub async fn send_reminder_email(
    email_config: &EmailConfig,
    contact: &Contact,
    action: &EmailAction,
) -> Result<()> {
    // Ensure the contact has an email address.
    let recipient_email = contact
        .email
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Contact '{}' has no email address.", contact.name))?;

    // 1. Read and prepare the email template.
    let template_content = fs::read_to_string(&action.template)
        .with_context(|| format!("Failed to read email template from '{}'", action.template))?;

    // Perform simple placeholder replacement.
    let email_body = template_content.replace("{contact_name}", &contact.name);

    // 2. Build the email message.
    let email = Message::builder()
        .from(email_config.from_address.parse()?)
        .to(recipient_email.parse()?)
        .subject(&action.subject)
        .header(lettre::message::header::ContentType::TEXT_HTML)
        .body(email_body)?;

    // 3. Configure the SMTP transport.
    let creds = Credentials::new(
        email_config.smtp_user.clone(),
        email_config.smtp_password.clone(),
    );

    // Build the mailer transport.
    let mailer = SmtpTransport::relay(&email_config.smtp_server)?
        .credentials(creds)
        .build();

    // 4. Send the email.
    // The `send` method is synchronous, but we run it in a blocking task
    // to avoid blocking the async runtime.
    tokio::task::spawn_blocking(move || mailer.send(&email))
        .await? // Wait for the blocking task to complete
        .with_context(|| format!("Failed to send email to '{}'", recipient_email))?;

    Ok(())
}
