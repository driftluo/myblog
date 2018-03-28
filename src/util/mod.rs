pub mod redis_pool;
pub mod postgresql_pool;
pub mod github_information;

pub use self::redis_pool::{create_redis_pool, Redis, RedisPool};
pub use self::postgresql_pool::{create_pg_pool, Postgresql};
pub use self::github_information::{get_github_primary_email, get_github_token, get_github_account_nickname_address};

use rand::{thread_rng, Rng};
use tiny_keccak::Keccak;
use std::fmt::Write;
use comrak::{markdown_to_html, ComrakOptions};
use std::sync::Arc;
use sapper::{Key, Request};
use chrono::Utc;
use ammonia::clean;
use sapper_std::{Context, SessionVal};
use super::UserInfo;
use serde_json;

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
                let info = serde_json::from_str::<UserInfo>(&redis_pool
                    .hget::<String>(cookie, "info"))
                    .unwrap();
                web.add("user", &info);
                (Some(info.groups), web)
            } else {
                (None, web)
            }
        }
        None => (None, web),
    }
}

/// Get visitors' ip and time, and then push it to redis key `visitor_log`
#[inline]
pub fn visitor_log(req: &Request, redis_pool: &Arc<RedisPool>) {
    let ip = String::from_utf8(
        req.headers().get_raw("X-Real-IP").unwrap()[0]
            .as_slice()
            .to_vec(),
    ).unwrap();
    let timestamp = Utc::now();
    redis_pool.lua_push(
        "visitor_log",
        &json!({"ip": &ip, "timestamp": &timestamp}).to_string(),
    );
}

pub struct Permissions;

impl Key for Permissions {
    type Value = Option<i16>;
}

pub struct WebContext;

impl Key for WebContext {
    type Value = Context;
}
