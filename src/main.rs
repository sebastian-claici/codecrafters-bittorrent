use anyhow::Context;

use crate::hashes::Hashes;
use serde::Deserialize;
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

#[derive(Debug, Clone, Deserialize)]
struct Torrent {
    announce: String,
    info: Info,
}

#[derive(Debug, Clone, Deserialize)]
struct Info {
    name: String,

    #[serde(rename = "piece length")]
    piece_length: usize,
    pieces: Hashes,

    #[serde(flatten)]
    keys: Keys,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Keys {
    SingleFile { length: usize },
    MultiFile { files: Vec<File> },
}

#[derive(Debug, Clone, Deserialize)]
struct File {
    length: usize,
    path: Vec<String>,
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Decode { value } => {
            // let v: serde_json::Value = serde_bencode::from_str(&value).unwrap();
            // println!("{}", v);
            todo!();
        }
        Command::Info { torrent } => {
            let f = std::fs::read(torrent).context("open torrent file")?;
            let t: Torrent = serde_bencode::from_bytes(&f).context("parse torrent file")?;
            println!("Tracker URL: {}", t.announce);
            println!(
                "Length: {}",
                match t.info.keys {
                    Keys::SingleFile { length } => length,
                    _ => todo!()
                }
            );
        }
    }

    Ok(())
}

mod hashes {
    use serde::de::{self, Visitor};
    use serde::Deserialize;
    use serde::Deserializer;

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
}
