use ::futures::future::OptionFuture;
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use tracing::instrument;
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

    #[instrument(skip(self), fields(cid = %cid))]
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

    #[instrument(skip(self, full_name, email), fields(cid = %cid))]
    pub async fn create_user(
        &self,
        cid: &str,
        full_name: &str,
        email: Option<&str>,
    ) -> Result<Vec<MoodleCreateUserResponseItem>, MoodleError> {
        if self.api_key.is_empty() {
            return Ok(Vec::new());
        }

        let (firstname, lastname) = split_full_name(full_name, cid);
        let fallback_email = format!("{cid}@noreply.users.vatprc.net");
        let email = email.unwrap_or(&fallback_email);

        Ok(self
            .http
            .post(MOODLE_ENDPOINT)
            .form(&[
                ("wstoken", self.api_key.as_str()),
                ("wsfunction", "core_user_create_users"),
                ("moodlewsrestformat", "json"),
                ("users[0][username]", cid),
                ("users[0][idnumber]", cid),
                ("users[0][createpassword]", "1"),
                ("users[0][firstname]", firstname.as_str()),
                ("users[0][lastname]", lastname.as_str()),
                ("users[0][email]", email),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<MoodleCreateUserResponseItem>>()
            .await?)
    }
}

#[derive(Debug, Deserialize)]
pub struct MoodleUser {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct MoodleCreateUserResponseItem {
    pub id: i64,
    pub username: String,
}

fn split_full_name(full_name: &str, cid: &str) -> (String, String) {
    let full_name = full_name.trim();
    if full_name.is_empty() {
        return (cid.to_string(), cid.to_string());
    }

    match full_name.split_once(' ') {
        Some((firstname, lastname))
            if !firstname.trim().is_empty() && !lastname.trim().is_empty() =>
        {
            (firstname.trim().to_string(), lastname.trim().to_string())
        }
        _ => (full_name.to_string(), cid.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::split_full_name;

    #[test]
    fn split_full_name_splits_on_first_space() {
        assert_eq!(
            split_full_name("Ada Lovelace Byron", "1234567"),
            ("Ada".to_string(), "Lovelace Byron".to_string())
        );
    }

    #[test]
    fn split_full_name_uses_cid_as_last_name_fallback() {
        assert_eq!(
            split_full_name("Ada", "1234567"),
            ("Ada".to_string(), "1234567".to_string())
        );
    }
}
