use crate::config;
use crate::core::wacraft::models::{Conversation, MessagePayloadBase, Order};
use crate::core::wacraft::{
    client::WacraftClient,
    models::{MessagePayload, SendWhatsAppMessage},
};
use crate::core::{email, http_request};
use anyhow::{Context, Result, anyhow};
use chrono::{Duration, Utc};
use clap::Subcommand;
use log::info;

/// Actions for managing and sending reminders.
#[derive(Subcommand, Debug)]
pub enum RemindersAction {
    /// Sends the appropriate reminder to a specific contact based on their inactivity.
    Send {
        /// The unique ID of the messaging product contact to send the reminder to.
        #[arg(long)]
        contact_id: String,

        /// (Internal) Skips actual message sending, useful for testing.
        #[arg(long, hide = true)]
        mock: bool,
    },
}

/// Handles the `reminders` subcommand.
pub async fn handle_reminders_command(action: RemindersAction) -> Result<()> {
    match action {
        RemindersAction::Send { contact_id, mock } => {
            // We need to load settings here to pass the email config down.
            let settings = config::load_settings().context("Failed to load settings.json")?;
            send_reminder_to_contact(&contact_id, &settings, None, mock).await?;
        }
    }
    Ok(())
}

/// The core logic for sending a reminder to a single contact.
pub async fn send_reminder_to_contact(
    contact_id: &str,
    settings: &config::models::Settings,
    conversation: Option<&Conversation>,
    mock: bool,
) -> Result<()> {
    info!("Preparing to send reminder to contact: {}", contact_id);

    // 1. Load reminder rules
    let reminders = config::load_reminders().context("Failed to load reminders.json.")?;
    if reminders.is_empty() {
        info!(
            "No reminder rules found. Nothing to do for contact {}.",
            contact_id
        );
        return Ok(());
    }

    // 2. Initialize Wacraft Client
    let client = WacraftClient::new(settings.wacraft.clone());

    // 3. Fetch the latest conversation for the contact
    // Fetch the latest conversation for user if is not provided.
    let latest_conversation = if let Some(conv) = conversation {
        conv
    } else {
        let conversations = client
            .get_conversation_messages(contact_id, 1, 0, None, Some(Order::Desc), None)
            .await?;

        &conversations
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No conversation found"))?
    };

    const NIL_UUID: &str = "00000000-0000-0000-0000-000000000000";

    // First, find a valid contact reference from the conversation, if any.
    let contact_ref = latest_conversation
        .to_contact
        .as_ref()
        .filter(|c| c.id != NIL_UUID)
        .or_else(|| {
            latest_conversation
                .from_contact
                .as_ref()
                .filter(|c| c.id != NIL_UUID)
        });

    // Now, get an owned `Contact`.
    // If we found a reference, clone it. Otherwise, fetch it from the client.
    // This assumes the async function returns a `Result<Contact, Error>` and `Contact` implements `Clone`.
    let contact = if let Some(ctt) = contact_ref {
        ctt
    } else {
        &client
            .get_messaging_product_contact_by_id(contact_id)
            .await?
            .ok_or_else(|| anyhow!("No messaging product contact found"))?
    };

    // 4. Determine which reminder rule applies
    let last_message_time = latest_conversation.updated_at;
    let inactive_duration = Utc::now().signed_duration_since(last_message_time);

    let mut applicable_rules = reminders;
    applicable_rules.sort_by(|a, b| b.inactive_for_hours.cmp(&a.inactive_for_hours));

    let rule_to_apply = applicable_rules
        .into_iter()
        .find(|rule| inactive_duration >= Duration::hours(rule.inactive_for_hours as i64));

    if let Some(rule) = rule_to_apply {
        println!(
            "Contact has been inactive for {} hours. Applying rule: '{}'",
            inactive_duration.num_days(),
            rule.name
        );

        let wrp_contact = &contact.contact.as_ref().ok_or_else(|| {
            anyhow!(
                "Messaging product {} is missing contact details",
                contact_id
            )
        })?;

        // 5. Execute the action defined in the rule
        match &rule.action {
            Some(config::models::Action::WacraftMessage(action)) => {
                let product_details = contact
                    .product_details
                    .as_ref()
                    .ok_or_else(|| anyhow!("Contact {} missing product details", contact_id))?;

                let payload_base: MessagePayloadBase = action.sender_data.clone();
                let payload = MessagePayload {
                    base: payload_base,
                    to: product_details.wa_id.clone(),
                };

                let message_to_send = SendWhatsAppMessage {
                    to_id: contact_id.to_string(),
                    sender_data: payload,
                };

                println!("Sending Wacraft message to {}...", wrp_contact.name);
                if !mock {
                    client.send_message(&message_to_send).await?;
                }
                println!("✅ Successfully sent Wacraft reminder to {}.", contact_id);
            }
            Some(config::models::Action::Email(action)) => {
                println!("Sending email reminder to {}...", wrp_contact.name);
                if !mock {
                    email::send_reminder_email(&settings.email, &wrp_contact, action).await?;
                }
                println!("✅ Successfully sent email reminder to {}.", contact_id);
            }
            Some(config::models::Action::HttpRequest(action)) => {
                println!("Executing HTTP request for rule '{}'...", rule.name);
                if !mock {
                    http_request::send_http_request(action, &wrp_contact).await?;
                }
                println!("✅ Successfully executed HTTP request for {}.", contact_id);
            }
            None => {
                println!("✅ No action for {}.", contact_id);
            }
        }
    } else {
        info!(
            "Contact {} is not inactive long enough for any reminder rule to apply.",
            contact_id
        );
    }

    Ok(())
}
