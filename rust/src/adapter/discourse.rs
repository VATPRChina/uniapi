use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;

#[derive(Clone)]
pub struct DiscourseClient {
    endpoint: String,
    api_key: String,
    http: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum DiscourseError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl DiscourseClient {
    pub fn new(endpoint: String, api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("vatprc-uniapi-rust/0.1"),
        );

        Self {
            endpoint,
            api_key,
            http: reqwest::Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(15))
                .build()
                .expect("discourse reqwest client should build"),
        }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn get_notam_topics(&self) -> Result<CategoryResult, DiscourseError> {
        let mut request = self.http.get(format!(
            "{}/c/69-category/notam/79.json",
            self.endpoint.trim_end_matches('/')
        ));
        if !self.api_key.is_empty() {
            request = request.header("Api-Key", &self.api_key);
        }

        Ok(request
            .send()
            .await?
            .error_for_status()?
            .json::<CategoryResult>()
            .await?)
    }
}

#[derive(Debug, Deserialize)]
pub struct CategoryResult {
    pub topic_list: TopicList,
}

#[derive(Debug, Deserialize)]
pub struct TopicList {
    pub topics: Vec<Topic>,
}

#[derive(Debug, Deserialize)]
pub struct Topic {
    pub id: u32,
    pub title: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
