use lettre::message::{Mailbox, MultiPart, SinglePart, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use thiserror::Error;
use tracing::instrument;
use ulid::Ulid;

use crate::dto::AtcApplicationStatus;
use crate::repository::atc::atc_application::AtcApplicationRecord;
use crate::repository::atc_training::training_application::TrainingApplicationRecord;
use crate::repository::atc_training::training_application_response::TrainingApplicationResponseRecord;
use crate::settings::Email;

const SITE_URL: &str = "https://www.vatprc.net";

#[derive(Clone)]
pub struct EmailClient {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: Option<Mailbox>,
}

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("invalid email address: {0}")]
    Address(#[from] lettre::address::AddressError),
    #[error("failed to build email: {0}")]
    Build(#[from] lettre::error::Error),
    #[error("SMTP error: {0}")]
    Smtp(#[from] lettre::transport::smtp::Error),
}

pub struct EmailContent {
    subject: String,
    plain_text: String,
    html: String,
}

impl EmailClient {
    pub fn new(settings: &Email) -> Result<Self, EmailError> {
        if !settings.enabled {
            return Ok(Self {
                transport: None,
                from: None,
            });
        }

        let smtp = &settings.smtp;
        let credentials = Credentials::new(smtp.username.clone(), smtp.password.clone());
        let transport = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.server)?
            .port(smtp.port)
            .credentials(credentials)
            .build();

        Ok(Self {
            transport: Some(transport),
            from: Some(smtp.from.parse()?),
        })
    }

    #[instrument(skip(self, content), fields(email.to = %to, email.subject = %content.subject))]
    pub async fn send(&self, to: &str, content: EmailContent) -> Result<(), EmailError> {
        let (Some(transport), Some(from)) = (&self.transport, &self.from) else {
            tracing::debug!("email delivery is disabled");
            return Ok(());
        };

        let message = Message::builder()
            .from(from.clone())
            .to(to.parse()?)
            .subject(format!("[VATPRC]{}", content.subject))
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(content.plain_text),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(content.html),
                    ),
            )?;

        transport.send(message).await?;
        tracing::info!("email sent");
        Ok(())
    }
}

impl EmailContent {
    pub fn atc_application_status_change(application: &AtcApplicationRecord) -> Self {
        let status = AtcApplicationStatus::from_db_str(&application.status)
            .map(|status| match status {
                AtcApplicationStatus::Submitted => "pending review",
                AtcApplicationStatus::InWaitlist => "in the waitlist queue",
                AtcApplicationStatus::Approved => "approved",
                AtcApplicationStatus::Rejected => "rejected",
                AtcApplicationStatus::Aborted => "training aborted",
            })
            .unwrap_or("changed status");
        let intro = format!(
            "Your ATC application is now {status}.\nYou can view the details in the ATC application list."
        );
        let action_url = format!("/controllers/applications/{}", Ulid::from(application.id));

        Self::notification(
            "Your ATC application status has changed",
            "ATC Application Status Update",
            &intro,
            "View application",
            &action_url,
            None,
            "you have submitted an ATC application",
        )
    }

    pub fn training_application_response(
        application: &TrainingApplicationRecord,
        response: &TrainingApplicationResponseRecord,
    ) -> Self {
        let decision = if response.slot_id.is_some() {
            "Approved"
        } else {
            "Rejected"
        };
        let details = format!(
            "Response Details\nStudent: {}/{}\nTitle: {}\nResponse: {}\nComments: {}",
            application.trainee_full_name,
            application.trainee_cid,
            application.name,
            decision,
            response.comment,
        );

        Self::notification(
            "New response to training request",
            "New response for your training request",
            "We have received a new response for your training request.\nYou can view the training request details in the training request list.",
            "View all training requests",
            "/controllers/trainings",
            Some(&details),
            "you have filed a training application",
        )
    }

    fn notification(
        subject: &str,
        title: &str,
        intro: &str,
        action_text: &str,
        action_path: &str,
        details: Option<&str>,
        reason: &str,
    ) -> Self {
        let action_url = format!("{SITE_URL}{action_path}");
        let details_text = details
            .map(|value| format!("\n\n{value}"))
            .unwrap_or_default();
        let plain_text = format!(
            "{title}\n---\n\n{intro}\n\n{action_text}: {action_url}{details_text}\n\n---\nYou are receiving this email because {reason} in VATPRC.\nThis is a notification email which you cannot unsubscribe from.\nTo manage other email communication preferences, visit {SITE_URL}/users/me.\nFor further questions, contact VATPRC Support at feedback@vatprc.net."
        );
        let intro_html = paragraphs(intro);
        let details_html = details
            .map(|value| {
                format!(
                    "<hr><div style=\"white-space:pre-line\">{}</div>",
                    escape_html(value)
                )
            })
            .unwrap_or_default();
        let html = format!(
            "<!doctype html><html><body style=\"margin:0;background:#f9fafb;font-family:Arial,sans-serif;color:#111827\"><div style=\"max-width:640px;margin:auto;padding:32px 16px\"><div style=\"background:#fff;padding:32px\"><h1 style=\"font-size:24px\">{}</h1>{}<p><a href=\"{}\" style=\"display:inline-block;padding:12px 20px;background:#ab1615;color:#fff;text-decoration:none;border-radius:4px\">{}</a></p>{}</div><div style=\"color:#6b7280;font-size:13px;padding:20px 0\"><p>You are receiving this email because {} in VATPRC.</p><p>This is a notification email which you cannot unsubscribe from. Manage other email communication preferences in your <a href=\"{}/users/me\">account settings</a>. For further questions, contact <a href=\"mailto:feedback@vatprc.net\">VATPRC Support</a>.</p></div></div></body></html>",
            escape_html(title),
            intro_html,
            escape_html(&action_url),
            escape_html(action_text),
            details_html,
            escape_html(reason),
            SITE_URL,
        );

        Self {
            subject: subject.to_string(),
            plain_text,
            html,
        }
    }
}

fn paragraphs(value: &str) -> String {
    value
        .lines()
        .map(|line| format!("<p>{}</p>", escape_html(line)))
        .collect()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_escapes_user_content() {
        let email = EmailContent::notification(
            "subject",
            "title",
            "hello <script>",
            "open",
            "/path",
            Some("Comments: <b>unsafe</b>"),
            "testing",
        );

        assert!(email.html.contains("hello &lt;script&gt;"));
        assert!(email.html.contains("&lt;b&gt;unsafe&lt;/b&gt;"));
        assert!(!email.html.contains("<script>"));
    }
}
