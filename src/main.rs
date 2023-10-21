use anyhow::Context;
use sha1::{Digest, Sha1};

use crate::hashes::Hashes;
use serde::{Deserialize, Serialize};
use serde_bencode;
use serde_json;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Simple program to greet a person
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Adds files to myapp
    Decode {
        value: String,
    },
    Info {
        torrent: PathBuf,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Torrent {
    announce: String,
    info: Info,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Info {
    name: String,

    #[serde(rename = "piece length")]
    piece_length: usize,
    pieces: Hashes,

    #[serde(flatten)]
    keys: Keys,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct File {
    length: usize,
    path: Vec<String>,
}

fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    // If encoded_value starts with a digit, it's a number
    match encoded_value.chars().next() {
        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                if let Ok(n) = len.parse::<usize>() {
                    return (rest[..n].into(), &rest[n..]);
                }
            }
        }
        Some('i') => {
            if let Some((n, rest)) = encoded_value
                .split_once('i')
                .and_then(|(_, rest)| rest.split_once('e'))
                .and_then(|(n, rest)| {
                    let n = n.parse::<i64>().expect("Expected integer");
                    Some((n, rest))
                })
            {
                return (n.into(), rest);
            }
        }
        Some('l') => {
            let mut values: Vec<serde_json::Value> = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (value, remainder) = decode_bencoded_value(&rest);
                values.push(value);
                rest = &remainder;
            }
            return (values.into(), rest);
        }
        Some('d') => {
            let mut dict = serde_json::Map::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (key, remainder) = decode_bencoded_value(&rest);
                let (value, remainder) = decode_bencoded_value(&remainder);
                let key = match key {
                    serde_json::Value::String(key) => key,
                    _ => panic!("Dict keys must be strings"),
                };
                dict.insert(key, value);
                rest = &remainder;
            }
            return (dict.into(), rest);
        }
        _ => {}
    }

    panic!("unsupported type")
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Decode { value } => {
            let v = decode_bencoded_value(&value).0.to_string();
            println!("{}", v);
        }
        Command::Info { torrent } => {
            let f = std::fs::read(torrent).context("open torrent file")?;
            let t: Torrent = serde_bencode::from_bytes(&f).context("parse torrent file")?;
            let d = serde_bencode::to_bytes(&t.info).context("bencode info dict")?;
            println!("Tracker URL: {}", t.announce);
            println!(
                "Length: {}",
                match t.info.keys {
                    Keys::SingleFile { length } => length,
                    _ => todo!(),
                }
            );

            let mut hasher = Sha1::new();
            hasher.update(d);
            let result = hasher.finalize();
            println!("Info Hash: {}", hex::encode(result));
        }
    }

    Ok(())
}

mod hashes {
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use serde::{Deserialize, Serialize};

    use std::fmt;

    #[derive(Debug, Clone)]
    pub struct Hashes(Vec<[u8; 20]>);
    struct HashesVisitor;

    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a byte string whose length is a multiple of 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if v.len() % 20 != 0 {
                Err(E::custom(
                    "length of hash byte string must be a multiple of 20",
                ))
            } else {
                Ok(Hashes(
                    v.chunks_exact(20)
                        .map(|slice| slice.try_into().expect("guaranteed to be length 20"))
                        .collect(),
                ))
            }
        }
    }

    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Hashes, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let single_slice = self.0.concat();
            serializer.serialize_bytes(&single_slice)
        }
    }
}
