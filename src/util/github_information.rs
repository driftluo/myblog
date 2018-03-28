use std::io::Read;
use sapper::Error as SapperError;
use serde_json;
use serde_urlencoded;
use hyper_native_tls::NativeTlsClient;
use hyper::net::HttpsConnector;
use hyper::Client;
use hyper::header::Headers;
use hyper::header::ContentType;

fn create_https_client() -> Client {
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    Client::with_connector(connector)
}

pub fn get_github_token(code: &str) -> Result<String, SapperError> {
    let client = create_https_client();
    let params = serde_urlencoded::to_string(
        [
            ("client_id", "52b10cd3fff369999cd9"),
            ("client_secret", "212d9729ead001da8844c1dcc79d45240166bd4f"),
            ("code", code),
            ("accept", "json"),
        ],
    ).unwrap();

    client.post("https://github.com/login/oauth/access_token")
        .header(ContentType::form_url_encoded())
        .body(&params)
        .send()
        .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
        .and_then(|mut response| {
            let mut body = String::new();
            response.read_to_string(&mut body)
                .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                .map(|_| body)
        }).and_then(|ref body| {
        #[derive(Deserialize)]
        struct Inner {
            access_token: String
        }
        serde_urlencoded::from_str::<Inner>(body)
            .map_err(|_| SapperError::Custom(String::from("No permission")))
            .map(|inner| inner.access_token)
    })
}

pub fn get_github_account_nickname_address(raw_token: &str) -> Result<(String ,String, String), SapperError> {
    let client = create_https_client();
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let user_url = format!("https://api.github.com/user?{}", token);

    let mut header = Headers::new();
    header.append_raw("User-Agent", b"rustcc".to_vec());

    client.get(&user_url)
        .headers(header)
        .send()
        .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
        .and_then(|mut response|{
            let mut body = String::new();
            response.read_to_string(&mut body)
                .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
                .map(|_| body)
        }).and_then(|ref body| {
        serde_json::from_str::<serde_json::Value>(body)
            .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
            .and_then(|inner| {
                let nickname = match inner["name"].as_str() {
                    Some(data) => data.to_string(),
                    None => return Err(SapperError::Custom(format!("read body error")))
                };
                let github_address = match inner["html_url"].as_str() {
                    Some(data) => data.to_string(),
                    None => return Err(SapperError::Custom(format!("read body error")))
                };
                let account = match inner["login"].as_str() {
                    Some(data) => data.to_string(),
                    None => return Err(SapperError::Custom(format!("read body error")))
                };
                Ok((account, nickname, github_address))
            })
    })
}

pub fn get_github_primary_email(raw_token: &str) -> Result<String, String> {
    let client = create_https_client();
    let token = serde_urlencoded::to_string([("access_token", raw_token)]).unwrap();

    let email_url = format!("https://api.github.com/user/emails?{}", token);
    let mut header = Headers::new();
    header.append_raw("User-Agent", b"rustcc".to_vec());

    client.get(&email_url)
        .headers(header)
        .send()
        .map_err(|e| format!("hyper's io error: '{}'", e))
        .and_then(|mut response|{
            let mut body = String::new();
            response.read_to_string(&mut body)
                .map_err(|e| format!("read body error: '{}'", e))
                .map(|_| body)
        }).and_then(|ref body| {
        serde_json::from_str::<Vec<serde_json::Value>>(body)
            .map_err(|e| format!("read body error: '{}'", e))
            .map(|raw_emails| {
                let primary_email = raw_emails
                    .iter()
                    .into_iter()
                    .filter(|x| x["primary"].as_bool().unwrap())
                    .map(|x| x["email"].as_str().unwrap())
                    .collect::<Vec<&str>>()
                    [0];
                primary_email.to_string()
            })
    })
}
