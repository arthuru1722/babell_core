use super::models::{FlareSolverrRawResponse, FlareSolverrRequest};
use std::error::Error;

pub async fn request_url(
    flaresolverr_url: &str,
    target_url: &str,
) -> Result<String, Box<dyn Error>> {
    let payload = FlareSolverrRequest {
        cmd: "request.get".to_string(),
        url: target_url.to_string(),
        max_timeout: 60000,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(flaresolverr_url)
        .json(&payload)
        .send()
        .await?
        .json::<FlareSolverrRawResponse>()
        .await?;

    if response.status != "ok" {
        return Err("FlareSolverr failed to process the request".into());
    }

    Ok(response.solution.response)
}
