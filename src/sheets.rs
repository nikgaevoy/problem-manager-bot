use std::time::Duration;

use reqwest::Client;
use serde_json::json;
use tokio::time::sleep;

use crate::problem::Problem;

const SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets";
const MAX_RETRIES: u32 = 5;

pub async fn append_problem(
    problem: &Problem,
    spreadsheet_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auth = gcp_auth::AuthenticationManager::new().await?;
    let token = auth.get_token(&[SCOPE]).await?;

    let sheet = std::env::var("SHEET_NAME").unwrap_or_else(|_| "Sheet1".into());
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}!A1:append\
         ?valueInputOption=USER_ENTERED",
        spreadsheet_id, sheet
    );

    let client = Client::new();
    let body = json!({ "values": [problem.to_sheet_row()] });
    let mut delay = Duration::from_secs(1);

    for attempt in 0..MAX_RETRIES {
        let resp = client
            .post(&url)
            .bearer_auth(token.as_str())
            .json(&body)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let wait = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(delay);
            eprintln!(
                "Rate limited by Sheets API, retrying in {:.1}s (attempt {}/{})",
                wait.as_secs_f32(),
                attempt + 1,
                MAX_RETRIES
            );
            sleep(wait).await;
            delay *= 2;
            continue;
        }

        resp.error_for_status()?;
        return Ok(());
    }

    Err("Sheets API rate limit exceeded after retries".into())
}
