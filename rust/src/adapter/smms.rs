use chrono::Utc;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use thiserror::Error;
use ulid::Ulid;

#[derive(Clone)]
pub struct SmmsClient {
    base_url: String,
    http: reqwest::Client,
    secret_token: String,
}

#[derive(Debug, Error)]
pub enum SmmsError {
    #[error("SM.MS secret token is not configured.")]
    MissingSecretToken,

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("{0}")]
    Rejected(String),

    #[error("No URL returned")]
    MissingUrl,
}

impl SmmsClient {
    pub fn new(base_url: String, secret_token: String) -> Self {
        Self {
            base_url,
            http: reqwest::Client::new(),
            secret_token,
        }
    }

    pub async fn upload_image(
        &self,
        image: Vec<u8>,
        file_name: Option<String>,
        content_type: Option<String>,
    ) -> Result<String, SmmsError> {
        if self.secret_token.is_empty() {
            return Err(SmmsError::MissingSecretToken);
        }

        let file_name = normalize_file_name(file_name);
        let upload_name = upload_file_name(&file_name);

        let mut part = Part::bytes(image).file_name(upload_name);
        if let Some(content_type) = content_type {
            part = part
                .mime_str(&content_type)
                .map_err(|err| SmmsError::Rejected(err.to_string()))?;
        }

        let response = self
            .http
            .post(format!("{}/upload", self.base_url.trim_end_matches('/')))
            .header("Authorization", &self.secret_token)
            .multipart(Form::new().part("file", part))
            .send()
            .await?
            .error_for_status()?
            .json::<SmmsResponse>()
            .await?;

        uploaded_url_from_response(response)
    }
}

fn normalize_file_name(file_name: Option<String>) -> String {
    match file_name {
        Some(file_name) if file_name.chars().all(char::is_alphanumeric) => file_name,
        _ => Ulid::new().to_string(),
    }
}

fn upload_file_name(file_name: &str) -> String {
    format!("vatprc-{}-{file_name}", Utc::now().format("%Y-%m-%d"))
}

fn uploaded_url_from_response(response: SmmsResponse) -> Result<String, SmmsError> {
    if response.code != 200 {
        return Err(SmmsError::Rejected(response.message));
    }

    response
        .data
        .map(|data| data.url)
        .ok_or(SmmsError::MissingUrl)
}

#[derive(Deserialize)]
struct SmmsResponse {
    code: i64,
    data: Option<SmmsData>,
    message: String,
}

#[derive(Deserialize)]
struct SmmsData {
    url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_file_name_keeps_alphanumeric_names() {
        assert_eq!(
            normalize_file_name(Some("Vatprc2026".to_string())),
            "Vatprc2026"
        );
    }

    #[test]
    fn normalize_file_name_replaces_unsafe_names_with_ulid() {
        let file_name = normalize_file_name(Some("event-banner.png".to_string()));

        assert_ne!(file_name, "event-banner.png");
        assert!(file_name.parse::<Ulid>().is_ok());
    }

    #[test]
    fn upload_file_name_adds_vatprc_prefix_and_date() {
        let upload_name = upload_file_name("image");

        assert!(upload_name.starts_with("vatprc-"));
        assert!(upload_name.ends_with("-image"));
    }

    #[tokio::test]
    async fn upload_image_fails_when_secret_token_is_missing() {
        let client = SmmsClient::new(String::new(), String::new());

        let error = client
            .upload_image(
                vec![1, 2, 3],
                Some("image".to_string()),
                Some("image/png".to_string()),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, SmmsError::MissingSecretToken));
    }

    #[tokio::test]
    async fn upload_image_fails_on_invalid_content_type() {
        let client = SmmsClient::new(String::new(), "secret".to_string());

        let error = client
            .upload_image(
                vec![1, 2, 3],
                Some("image".to_string()),
                Some("not a mime".to_string()),
            )
            .await
            .unwrap_err();

        assert!(matches!(error, SmmsError::Rejected(_)));
    }

    #[test]
    fn uploaded_url_from_response_returns_uploaded_url() {
        let url = uploaded_url_from_response(SmmsResponse {
            code: 200,
            message: "success".to_string(),
            data: Some(SmmsData {
                url: "https://example.test/image.png".to_string(),
            }),
        })
        .unwrap();

        assert_eq!(url, "https://example.test/image.png");
    }

    #[test]
    fn uploaded_url_from_response_returns_rejected_error_for_non_success_smms_response() {
        let error = uploaded_url_from_response(SmmsResponse {
            code: 400,
            message: "invalid image".to_string(),
            data: None,
        })
        .unwrap_err();

        assert!(matches!(error, SmmsError::Rejected(message) if message == "invalid image"));
    }

    #[test]
    fn uploaded_url_from_response_returns_missing_url_when_success_response_has_no_data() {
        let error = uploaded_url_from_response(SmmsResponse {
            code: 200,
            message: "success".to_string(),
            data: None,
        })
        .unwrap_err();

        assert!(matches!(error, SmmsError::MissingUrl));
    }
}
