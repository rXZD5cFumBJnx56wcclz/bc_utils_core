use hex;
use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn hmac_(
    key: &[u8],
    msg: &[u8],
) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).unwrap();
    mac.update(msg);
    hex::encode(mac.finalize().into_bytes())
}
