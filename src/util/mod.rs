pub mod redis_pool;
pub mod postgresql_pool;

pub use self::redis_pool::{ create_redis_pool, RedisPool, Redis };
pub use self::postgresql_pool::{ create_pg_pool, Postgresql };

use rand::{ thread_rng, Rng };
use tiny_keccak::Keccak;
use std::fmt::Write;
use comrak::{ markdown_to_html, ComrakOptions };
use std::sync::Arc;
use typemap::Key;

/// Get random value
#[inline]
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

/// Convert text to sha3_256 hex
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
    markdown_to_html(md, &option)
}

/// Get the real password, the first six is a random number
#[inline]
pub fn get_password(raw: &str) -> String {
    let (_, password) = raw.split_at(6);
    password.to_string()
}

#[inline]
pub fn admin_verification_cookie(cookie: Option<&String>, redis_pool: &Arc<RedisPool>) -> bool {
    match cookie {
        Some(cookie) => {
            let redis_key = "admin_".to_string() + cookie;
            redis_pool.exists(&redis_key)
        }
        None => {
            false
        }
    }
}

#[inline]
pub fn user_verification_cookie(cookie: Option<&String>, redis_pool: &Arc<RedisPool>) -> bool {
    match cookie {
        Some(cookie) => {
            let admin_redis_key = "admin_".to_string() + cookie;
            let user_redis_key = "user_".to_string() + &cookie;
            redis_pool.exists(&admin_redis_key) || redis_pool.exists(&user_redis_key)
        }
        None => {
            false
        }
    }
}

pub struct UserSession;

impl Key for UserSession {
    type Value = bool;
}

pub struct AdminSession;

impl Key for AdminSession {
    type Value = bool;
}
