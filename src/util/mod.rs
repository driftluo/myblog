use rand::{ thread_rng, Rng };
use tiny_keccak::Keccak;
use std::fmt::Write;

#[inline]
pub fn random_string(limit: usize) -> String {
    thread_rng().gen_ascii_chars().take(limit).collect()
}

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
