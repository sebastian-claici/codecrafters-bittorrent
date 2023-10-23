use crate::hashes::Hashes;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    pub announce: String,
    pub info: Info,
}

impl Torrent {
    pub fn info_hash(&self) -> anyhow::Result<[u8; 20]> {
        let info_dict = serde_bencode::to_bytes(&self.info).context("bencode info dict")?;
        let mut hasher = Sha1::new();
        hasher.update(&info_dict);
        let info_hash: [u8; 20] = hasher
            .finalize()
            .try_into()
            .expect("should only have 20 bytes");

        Ok(info_hash)
    }

    pub fn encode_hash(&self, info_hash: [u8; 20]) -> anyhow::Result<String> {
        // Encode info_hash into url encoded string
        let mut encoded = String::with_capacity(3 * info_hash.len());
        for &byte in &info_hash {
            encoded.push('%');
            encoded.push_str(&hex::encode(&[byte]));
        }

        Ok(encoded)

    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub name: String,

    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,

    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub length: usize,
    pub path: Vec<String>,
}
