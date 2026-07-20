use reqwest::Client;
use serde::Deserialize;

use crate::igdb::pipeline::DynError;

#[derive(Debug, Deserialize)]
pub struct TwitchTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

pub async fn get_access_token(
    client_id: &str,
    client_secret: &str,
) -> Result<TwitchTokenResponse, DynError> {
    let client = Client::builder().build()?;

    let response = client
        .post(format!(
            "https://id.twitch.tv/oauth2/token?client_id={client_id}&client_secret={client_secret}&grant_type=client_credentials"
        ))
        .send()
        .await?
        .error_for_status()?;

    Ok(response.json().await?)
}