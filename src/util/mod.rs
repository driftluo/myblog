pub mod redis_pool;
pub mod postgresql_pool;
pub mod github_information;

pub use self::redis_pool::{create_redis_pool, Redis, RedisPool};
pub use self::postgresql_pool::{create_pg_pool, Postgresql};
pub use self::github_information::{get_github_account_nickname_address, get_github_primary_email,
                                   get_github_token};

use rand::{thread_rng, Rng};
use tiny_keccak::Keccak;
use std::fmt::Write;
use comrak::{markdown_to_html, ComrakOptions};
use std::sync::Arc;
use sapper::{Client, Error as SapperError, Key, Request};
use chrono::Utc;
use ammonia::clean;
use sapper_std::{Context, SessionVal};
use super::{UserInfo, UserNotify};
use serde_json;
use std::thread;
use std::io::Read;

/// Get random value
#[inline]
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

/// Convert text to `sha3_256` hex
#[inline]
pub fn sha3_256_encode(s: String) -> String {
    let mut sha3 = Keccak::new_sha3_256();
    sha3.update(s.as_ref());
    let mut res: [u8; 32] = [0; 32];
    sha3.finalize(&mut res);
    let mut hex = String::with_capacity(64);
    for byte in res.iter() {
        write!(hex, "{:02x}", byte).expect("Can't fail on writing to string");
    }
    hex
}

/// Convert markdown to html
#[inline]
pub fn markdown_render(md: &str) -> String {
    let option = ComrakOptions {
        ext_strikethrough: true,
        ext_table: true,
        ext_tasklist: true,
        ext_superscript: true,
        safe: true,
        ext_tagfilter: true,
        hardbreaks: true,
        smart: true,
        github_pre_lang: true,
        ..ComrakOptions::default()
    };
    clean(&markdown_to_html(md, &option))
}

/// Get the real password, the first six is a random number
#[inline]
pub fn get_password(raw: &str) -> String {
    let (_, password) = raw.split_at(6);
    password.to_string()
}

/// Get visitor's permission and user info
/// `0` means Admin
/// `1` means User
pub fn get_identity_and_web_context(req: &Request) -> (Option<i16>, Context) {
    let mut web = Context::new();
    let cookie = req.ext().get::<SessionVal>();
    let redis_pool = req.ext().get::<Redis>().unwrap();
    match cookie {
        Some(cookie) => {
            if redis_pool.exists(cookie) {
                let info = serde_json::from_str::<UserInfo>(&redis_pool.hget::<String>(cookie, "info"))
                    .unwrap();
                let notifys = UserNotify::get_notifys(info.id, redis_pool);
                web.add("user", &info);
                web.add("notifys", &notifys);
                (Some(info.groups), web)
            } else {
                (None, web)
            }
        }
        None => (None, web),
    }
}

/// Get visitor ip information and access time, and then push it to redis key `visitor_log`
#[inline]
pub fn visitor_log(req: &Request, redis_pool: &Arc<RedisPool>) {
    let ip = String::from_utf8(
        req.headers().get_raw("X-Real-IP").unwrap()[0]
            .as_slice()
            .to_vec(),
    ).unwrap();
    let timestamp = Utc::now();
    let redis_pool = redis_pool.clone();

    // https://ipstack.com/documentation
    thread::spawn(move || {
        let url = format!("http://api.ipstack.com/{}?access_key=****", &ip);
        let data = Client::new()
            .get(&url)
            .send()
            .map_err(|e| SapperError::Custom(format!("hyper's io error: '{}'", e)))
            .and_then(|mut response| {
                let mut body = String::new();
                response
                    .read_to_string(&mut body)
                    .map(|_| body)
                    .map_err(|e| SapperError::Custom(format!("read body error: '{}'", e)))
            })
            .and_then(|ref body| {
                #[derive(Deserialize)]
                struct Inner {
                    country_name: Option<String>,
                    region_name: Option<String>,
                    city: Option<String>,
                }
                serde_json::from_str::<Inner>(body)
                    .map_err(|_| SapperError::Custom(String::from("serde error")))
                    .map(|inner| inner)
            })
            .unwrap();
        redis_pool.lua_push(
            "visitor_log",
            &json!({"ip": &ip, "timestamp": &timestamp, "country_name": data.country_name, "region_name": data.region_name, "city": data.city})
                .to_string(),
        );
    });
}

pub struct Permissions;

impl Key for Permissions {
    type Value = Option<i16>;
}

pub struct WebContext;

impl Key for WebContext {
    type Value = Context;
}
