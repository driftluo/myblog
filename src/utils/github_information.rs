/// GitHub API version header value
const GITHUB_API_VERSION: &str = "2022-11-28";

pub async fn get_github_token(code: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    #[derive(serde::Serialize)]
    struct TokenRequest<'a> {
        client_id: &'a str,
        client_secret: &'a str,
        code: &'a str,
    }

    let request_body = TokenRequest {
        client_id: "52b10cd3fff369999cd9",
        client_secret: "212d9729ead001da8844c1dcc79d45240166bd4f",
        code,
    };

    let mut headers = reqwest::header::HeaderMap::new();
    // Request JSON response format (recommended by GitHub)
    headers.append("Accept", "application/json".parse().unwrap());
    headers.append("Content-Type", "application/json".parse().unwrap());

    let res = client
        .post("https://github.com/login/oauth/access_token")
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("reqwest's io error: '{}'", e))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("read body err: {}", e))?;

    // Check for error response from GitHub
    if let Some(error) = res.get("error").and_then(|e| e.as_str()) {
        let description = res
            .get("error_description")
            .and_then(|d| d.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("GitHub OAuth error: {} - {}", error, description));
    }

    res.get("access_token")
        .and_then(|t| t.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| String::from("No access_token in response"))
}

pub async fn get_github_account_nickname_address(
    raw_token: &str,
) -> Result<(String, String, String), String> {
    let mut headers = reqwest::header::HeaderMap::new();
    // Use Bearer token authentication (required by current GitHub API)
    headers.append(
        "Authorization",
        format!("Bearer {}", raw_token).parse().unwrap(),
    );
    headers.append("Accept", "application/vnd.github+json".parse().unwrap());
    headers.append("X-GitHub-Api-Version", GITHUB_API_VERSION.parse().unwrap());
    headers.append("User-Agent", "myblog-oauth".parse().unwrap());

    let res = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
        .get("https://api.github.com/user")
        .send()
        .await
        .map_err(|e| format!("reqwest's io error: '{}'", e))?;

    // Check HTTP status code
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, body));
    }

    let res = res
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("read body error: '{}'", e))?;

    // login is required field
    let account = res["login"]
        .as_str()
        .ok_or_else(|| "Missing 'login' field in GitHub response".to_string())?
        .to_string();

    // name can be null, fallback to login
    let nickname = res["name"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| account.clone());

    // html_url should always exist
    let github_address = res["html_url"]
        .as_str()
        .ok_or_else(|| "Missing 'html_url' field in GitHub response".to_string())?
        .to_string();

    Ok((account, nickname, github_address))
}

pub async fn get_github_primary_email(raw_token: &str) -> Result<String, String> {
    let mut headers = reqwest::header::HeaderMap::new();
    // Use Bearer token authentication (required by current GitHub API)
    headers.append(
        "Authorization",
        format!("Bearer {}", raw_token).parse().unwrap(),
    );
    headers.append("Accept", "application/vnd.github+json".parse().unwrap());
    headers.append("X-GitHub-Api-Version", GITHUB_API_VERSION.parse().unwrap());
    headers.append("User-Agent", "driftluo's-blog-oauth".parse().unwrap());

    let res = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
        .get("https://api.github.com/user/emails")
        .send()
        .await
        .map_err(|e| format!("reqwest's io error: '{}'", e))?;

    // Check HTTP status code
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, body));
    }

    let emails = res
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| format!("read body error: '{}'", e))?;

    // First try to find primary email
    if let Some(primary_email) = emails
        .iter()
        .filter(|x| x["primary"].as_bool().unwrap_or(false))
        .filter_map(|x| x["email"].as_str())
        .next()
    {
        return Ok(primary_email.to_string());
    }

    // Fallback: return the first verified email
    if let Some(verified_email) = emails
        .iter()
        .filter(|x| x["verified"].as_bool().unwrap_or(false))
        .filter_map(|x| x["email"].as_str())
        .next()
    {
        return Ok(verified_email.to_string());
    }

    // Last resort: return any email
    emails
        .first()
        .and_then(|x| x["email"].as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "No email found in GitHub account".to_string())
}
