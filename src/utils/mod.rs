use crate::{
    COOKIE, USER_INFO,
    db_wrapper::get_redis,
    models::{notify::UserNotify, user::UserInfo},
    web::Cache,
};
use http_body_util::BodyExt;
use pulldown_cmark::{Options, Parser, html};
use rand::Rng;
use salvo::http::header;
use salvo::{
    Depot, Request, Response,
    http::{
        ResBody, StatusCode, StatusError,
        cookie::{Cookie, time},
    },
    prelude::handler,
    routing::FlowCtrl,
};
use std::str::FromStr;
use std::{fmt::Write, iter};
use tiny_keccak::Hasher;

pub mod github_information;

const COOKIE_NAME: &str = "blog_session";

#[inline]
pub fn markdown_render(src: &str) -> String {
    let options = Options::all();
    let parser = Parser::new_ext(src, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

#[inline]
pub fn random_string(limit: usize) -> String {
    iter::repeat(())
        .map(|_| rand::thread_rng().sample(rand::distributions::Alphanumeric))
        .map(char::from)
        .take(limit)
        .collect()
}

#[inline]
pub fn sha3_256_encode(s: String) -> String {
    let mut sha3 = tiny_keccak::Sha3::v256();
    sha3.update(s.as_ref());
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    let mut hex = String::with_capacity(64);
    for byte in res.iter() {
        write!(hex, "{:02x}", byte).expect("Can't fail on writing to string");
    }
    hex
}

/// Extract real password from frontend input
/// Frontend adds 6 random characters as prefix for obfuscation
/// Returns None if password length is less than 6 characters
#[inline]
pub fn get_password(raw: &str) -> Option<String> {
    if raw.len() < 6 {
        return None;
    }
    let (_, password) = raw.split_at(6);
    Some(password.to_string())
}

/// Get visitor's permission and user info
/// `0` means Admin
/// `1` means User
pub async fn get_identity_and_web_context(
    req: &Request,
    depot: &mut Depot,
) -> (Option<i16>, tera::Context) {
    let mut web = tera::Context::new();
    let redis_pool = get_redis();

    match req.cookies().get(COOKIE_NAME) {
        Some(v) => match redis_pool.hget::<Option<String>>(v.value(), "info").await {
            Ok(Some(info)) => {
                let info = serde_json::from_str::<UserInfo>(&info).unwrap();
                let notifys = UserNotify::get_notifys(info.id).await;
                web.insert("user", &info);
                web.insert("notifys", &notifys.unwrap_or_default());
                let groups = info.groups;

                depot.insert(USER_INFO, info);
                depot.insert(COOKIE, v.value().to_owned());

                (Some(groups), web)
            }
            _ => (None, web),
        },
        None => (None, web),
    }
}

pub fn set_cookie(
    res: &mut Response,
    value: String,
    domain: Option<&str>,
    path: Option<&str>,
    secure: Option<bool>,
    max_age: Option<i64>,
) {
    let mut cookie = Cookie::new(COOKIE_NAME, value);

    if let Some(i) = domain {
        cookie.set_domain(i.to_string());
    }
    if let Some(i) = path {
        cookie.set_path(i.to_string());
    }
    if let Some(i) = secure {
        cookie.set_secure(i);
    }
    if let Some(i) = max_age {
        cookie.set_max_age(time::Duration::hours(i))
    }

    res.add_cookie(cookie);
}

pub fn parse_query<T: FromStr>(req: &Request, name: &str) -> Result<T, StatusError> {
    if let Some(q) = req.uri().query() {
        let query_iter = url::form_urlencoded::parse(q.as_bytes());
        for (key, val) in query_iter {
            if key == name {
                return val
                    .parse()
                    .map_err(|_| from_code(StatusCode::BAD_REQUEST, "Query Param Not Found"));
            }
        }
    }

    Err(from_code(StatusCode::BAD_REQUEST, "Query Param Not Found"))
}

pub fn parse_last_path<T: FromStr>(req: &Request) -> Result<T, StatusError> {
    let path = req.uri().path();

    if let Some(k) = path.rsplit('/').next() {
        k.parse()
            .map_err(|_| from_code(StatusCode::BAD_REQUEST, "Path Param is Incorrect"))
    } else {
        Err(from_code(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Router Location Failed",
        ))
    }
}

pub async fn parse_json_body<T>(req: &mut Request) -> Option<T>
where
    T: for<'de> serde::de::Deserialize<'de>,
{
    if let Some(ctype) = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        && (ctype.starts_with("application/json") || ctype.starts_with("text/"))
    {
        let body = req.take_body();
        let bytes = body.collect().await.ok()?.to_bytes();
        return serde_json::from_slice(&bytes).ok();
    }

    None
}

pub async fn parse_form_body<T>(req: &mut Request, name: &str) -> Option<String> {
    if let Some(ctype) = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        && ctype == "application/x-www-form-urlencoded"
    {
        let body = req.take_body();
        let data = body.collect().await.ok()?.to_bytes();
        let form_iter = url::form_urlencoded::parse(&data);
        for (key, val) in form_iter {
            if key == name {
                return Some(val.to_string());
            }
        }
    }

    None
}

pub fn set_json_response<T: serde::Serialize>(res: &mut Response, size: usize, json: T) {
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json; charset=utf-8"),
    );
    let mut cache = Cache::with_capacity(size);
    serde_json::to_writer(&mut cache, &json).unwrap();
    res.body(ResBody::Once(cache.into_inner()));
    res.status_code(StatusCode::OK);
}

pub fn set_plain_text_response(res: &mut Response, text: bytes::BytesMut) {
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    res.body(ResBody::Once(text.freeze()));
    res.status_code(StatusCode::OK);
}

pub fn set_xml_text_response(res: &mut Response, text: bytes::BytesMut) {
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    res.body(ResBody::Once(text.freeze()));
    res.status_code(StatusCode::OK);
}

pub fn from_code<T>(code: StatusCode, name: T) -> StatusError
where
    T: Into<String>,
{
    StatusError::from_code(code).unwrap().brief(name)
}

#[handler]
pub async fn visitor_log(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    if let Some(ip) = req.header::<String>("X-Real-IP")
        && let Ok(key) = ::std::env::var("IPSTACK_KEY")
    {
        let timestamp = chrono::Utc::now();
        tokio::spawn(async move {
            let url = format!("http://api.ipstack.com/{}?access_key={}", &ip, key,);
            if let Ok(res) = reqwest::Client::new().get(&url).send().await {
                #[derive(serde::Deserialize, serde::Serialize)]
                struct Inner {
                    country_name: Option<String>,
                    region_name: Option<String>,
                    city: Option<String>,
                }

                #[derive(serde::Deserialize, serde::Serialize)]
                struct Dump {
                    ip: String,
                    timestamp: chrono::DateTime<chrono::Utc>,
                    #[serde(flatten)]
                    inner: Inner,
                }

                if let Ok(data) = res.json::<Inner>().await {
                    get_redis()
                        .lua_push(
                            "visitor_log",
                            &serde_json::to_string(&Dump {
                                ip,
                                timestamp,
                                inner: data,
                            })
                            .unwrap(),
                        )
                        .await;
                }
            }
        });
    }
    ctrl.call_next(req, depot, res).await;
}

#[cfg(test)]
mod test {
    use super::{parse_last_path, parse_query};
    use salvo::Request;

    fn build_request(uri: &str) -> Request {
        let mut req = Request::default();
        req.set_uri(uri.parse().unwrap());
        req
    }

    #[test]
    fn test_get_query() {
        let res = build_request("/hello/world?key=value&foo=bar");
        let v = parse_query::<String>(&res, "key").unwrap();
        let v2 = parse_query::<String>(&res, "foo").unwrap();

        assert_eq!(v, "value");
        assert_eq!(v2, "bar");
    }

    #[test]
    fn test_get_last_path() {
        let res = build_request("/hello/world?key=value&foo=bar");
        let v = parse_last_path::<String>(&res).unwrap();
        assert_eq!(v, "world");

        let res = build_request("/hello");
        let v = parse_last_path::<String>(&res).unwrap();
        assert_eq!(v, "hello");

        let res = build_request("/hello/world/you");
        let v = parse_last_path::<String>(&res).unwrap();
        assert_eq!(v, "you");

        let res = build_request("/hello/world/your");
        let v = parse_last_path::<String>(&res).unwrap();
        assert_eq!(v, "your");

        let res = build_request("/hello/world/your/b/c/d");
        let v = parse_last_path::<String>(&res).unwrap();
        assert_eq!(v, "d");
    }
}
