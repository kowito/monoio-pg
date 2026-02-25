use md5::Md5;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use crate::error::{Error, Result};

pub fn md5_encrypt(user: &str, password: &str, salt: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(password.as_bytes());
    hasher.update(user.as_bytes());
    let res1 = hasher.finalize();
    let res1_hex = hex::encode(res1);

    let mut hasher = Md5::new();
    hasher.update(res1_hex.as_bytes());
    hasher.update(salt);
    let res2 = hasher.finalize();
    format!("md5{}", hex::encode(res2))
}

use pbkdf2::pbkdf2;

pub struct ScramClient {
    password: String,
    client_first_message_bare: String,
}

impl ScramClient {
    pub fn new(user: &str, password: &str) -> Self {
        let nonce = hex::encode(rand::random::<[u8; 16]>());
        let user_escaped = user.replace("=", "=3D").replace(",", "=2C");
        let client_first_message_bare = format!("n={},r={}", user_escaped, nonce);
        Self {
            password: password.to_string(),
            client_first_message_bare,
        }
    }

    pub fn client_first_message(&self) -> String {
        format!("n,,{}", self.client_first_message_bare)
    }

    pub fn handle_server_first_message(&self, server_first: &str) -> Result<(String, [u8; 32])> {
        let mut r = None;
        let mut s = None;
        let mut i = None;

        for part in server_first.split(',') {
            if part.starts_with("r=") { r = Some(&part[2..]); }
            else if part.starts_with("s=") { s = Some(&part[2..]); }
            else if part.starts_with("i=") { i = Some(&part[2..]); }
        }

        let r = r.ok_or(Error::Authentication("Missing r in SCRAM".into()))?;
        let s = s.ok_or(Error::Authentication("Missing s in SCRAM".into()))?;
        let i = i.ok_or(Error::Authentication("Missing i in SCRAM".into()))?.parse::<u32>().map_err(|_| Error::Authentication("Invalid i".into()))?;

        let salt = BASE64.decode(s).map_err(|_| Error::Authentication("Invalid s base64".into()))?;

        let mut salted_password = [0u8; 32];
        pbkdf2::<Hmac<Sha256>>(self.password.as_bytes(), &salt, i, &mut salted_password).map_err(|e| Error::Authentication(e.to_string()))?;

        let client_key = hmac_sha256(&salted_password, b"Client Key");
        let stored_key = Sha256::digest(&client_key);

        let client_final_message_without_proof = format!("c=biws,r={}", r);
        let auth_message = format!("{},{},{}", self.client_first_message_bare, server_first, client_final_message_without_proof);

        let client_signature = hmac_sha256(&stored_key, auth_message.as_bytes());

        let mut client_proof = [0u8; 32];
        for j in 0..32 {
            client_proof[j] = client_key[j] ^ client_signature[j];
        }

        let proof_base64 = BASE64.encode(client_proof);
        let server_key = hmac_sha256(&salted_password, b"Server Key");
        let server_signature = hmac_sha256(&server_key, auth_message.as_bytes());

        Ok((format!("{},p={}", client_final_message_without_proof, proof_base64), server_signature))
    }
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC should handle any key length");
    mac.update(data);
    mac.finalize().into_bytes().into()
}
