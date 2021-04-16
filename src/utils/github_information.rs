pub async fn get_github_token(code: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let params = serde_urlencoded::to_string([
        ("client_id", "52b10cd3fff369999cd9"),
        ("client_secret", "212d9729ead001da8844c1dcc79d45240166bd4f"),
        ("code", code),
        ("accept", "json"),
    ])
    .unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.append("application", "x-www-form-urlencoded".parse().unwrap());

    let res = client
        .post("https://github.com/login/oauth/access_token")
        .headers(headers)
        .body(params)
        .send()
        .await
        .map_err(|e| format!("reqwest's io error: '{}'", e))?
        .text()
        .await
        .map_err(|e| format!("read body err: {}", e))?;
    #[derive(serde::Deserialize)]
    struct Inner {
        access_token: String,
    }
    serde_urlencoded::from_str::<Inner>(&res)
        .map_err(|_| String::from("No permission"))
        .map(|inner| inner.access_token)
}

pub async fn get_github_account_nickname_address(
    raw_token: &str,
) -> Result<(String, String, String), String> {
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let user_url = format!("https://api.github.com/user?{}", token);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.append("User-Agent", "rustcc".parse().unwrap());

    let res = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
        .get(&user_url)
        .send()
        .await
        .map_err(|e| format!("reqwest's io error: '{}'", e))?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("read body error: '{}'", e))?;

    let nickname = match res["name"].as_str() {
        Some(data) => data.to_string(),
        None => return Err("Your github account is missing a nickname setting".to_string()),
    };
    let github_address = match res["html_url"].as_str() {
        Some(data) => data.to_string(),
        None => return Err("read body error".to_string()),
    };
    let account = match res["login"].as_str() {
        Some(data) => data.to_string(),
        None => return Err("read body error".to_string()),
    };
    Ok((account, nickname, github_address))
}

pub async fn get_github_primary_email(raw_token: &str) -> Result<String, String> {
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();
    let email_url = format!("https://api.github.com/user/emails?{}", token);

    let mut headers = reqwest::header::HeaderMap::new();
    headers.append("User-Agent", "rustcc".parse().unwrap());
    let res = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap()
        .get(&email_url)
        .send()
        .await
        .map_err(|e| format!("reqwest's io error: '{}'", e))?
        .json::<Vec<serde_json::Value>>()
        .await
        .map_err(|e| format!("read body error: '{}'", e))?;
    let primary_email = res
        .iter()
        .filter(|x| x["primary"].as_bool().unwrap())
        .map(|x| x["email"].as_str().unwrap())
        .collect::<Vec<&str>>()[0];
    Ok(primary_email.to_string())
}
