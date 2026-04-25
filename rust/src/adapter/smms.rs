use chrono::Utc;
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use ulid::Ulid;

const BASE_URL: &str = "https://s.ee/api/v1/file";

#[derive(Clone)]
pub struct SmmsClient {
    http: reqwest::Client,
    secret_token: String,
}

#[derive(Debug)]
pub enum SmmsError {
    MissingSecretToken,
    Request(reqwest::Error),
    Rejected(String),
    MissingUrl,
}

impl SmmsClient {
    pub fn new(secret_token: String) -> Self {
        Self {
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
        let upload_name = format!("vatprc-{}-{file_name}", Utc::now().format("%Y-%m-%d"));

        let mut part = Part::bytes(image).file_name(upload_name);
        if let Some(content_type) = content_type {
            part = part
                .mime_str(&content_type)
                .map_err(|err| SmmsError::Rejected(err.to_string()))?;
        }

        let response = self
            .http
            .post(format!("{BASE_URL}/upload"))
            .header("Authorization", &self.secret_token)
            .multipart(Form::new().part("file", part))
            .send()
            .await
            .map_err(SmmsError::Request)?
            .error_for_status()
            .map_err(SmmsError::Request)?
            .json::<SmmsResponse>()
            .await
            .map_err(SmmsError::Request)?;

        if response.code != 200 {
            return Err(SmmsError::Rejected(response.message));
        }

        response
            .data
            .map(|data| data.url)
            .ok_or(SmmsError::MissingUrl)
    }
}

fn normalize_file_name(file_name: Option<String>) -> String {
    match file_name {
        Some(file_name) if file_name.chars().all(char::is_alphanumeric) => file_name,
        _ => Ulid::new().to_string(),
    }
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
