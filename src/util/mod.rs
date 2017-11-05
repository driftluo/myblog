use rand::{ thread_rng, Rng };
use md5;

#[inline]
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

#[inline]
pub fn md5_encode(s: String) -> String {
    format!("{:x}", md5::compute(s))
}
