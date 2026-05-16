use reqwest::Client;
use serde_json::json;

use crate::problem::Problem;

const SCOPE: &str = "https://www.googleapis.com/auth/spreadsheets";

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

    Client::new()
        .post(url)
        .bearer_auth(token.as_str())
        .json(&json!({ "values": [problem.to_sheet_row()] }))
        .send()
        .await?
        .error_for_status()?;

    Ok(())
}
