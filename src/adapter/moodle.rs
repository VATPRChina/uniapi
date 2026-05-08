use ::futures::future::OptionFuture;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;
use tokio::task::futures;

const MOODLE_ENDPOINT: &str = "https://moodle.vatprc.net/webservice/rest/server.php";

#[derive(Clone)]
pub struct MoodleClient {
    api_key: String,
    http: reqwest::Client,
}

#[derive(Debug, Error)]
pub enum MoodleError {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl MoodleClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http: reqwest::Client::new(),
        }
    }

    pub async fn get_user_by_cid(&self, cid: &str) -> Result<Option<MoodleUser>, MoodleError> {
        if self.api_key.is_empty() {
            return Ok(None);
        }

        let res = self
            .http
            .post(MOODLE_ENDPOINT)
            .form(&[
                ("wstoken", self.api_key.as_str()),
                ("wsfunction", "core_user_get_users_by_field"),
                ("moodlewsrestformat", "json"),
                ("field", "idnumber"),
                ("values[0]", cid),
            ])
            .send()
            .await?
            .error_for_status()
            .map(Some)
            .or_else(|e| {
                if e.is_status() && e.status() == Some(StatusCode::NOT_FOUND) {
                    Ok(None)
                } else {
                    Err(e)
                }
            })?;
        let users = (OptionFuture::from(res.map(|r| r.json::<Vec<MoodleUser>>())))
            .await
            .unwrap_or(Ok(Vec::new()))?;

        Ok(users.into_iter().next())
    }
}

#[derive(Debug, Deserialize)]
pub struct MoodleUser {
    pub id: i64,
}
