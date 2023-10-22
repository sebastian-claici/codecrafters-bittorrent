use bittorrent_starter_rust::torrent::{Keys, Torrent};
use bittorrent_starter_rust::tracker::{TrackerRequest, TrackerResponse};
use bittorrent_starter_rust::decode::decode_bencoded_value;

use anyhow::Context;

use serde_bencode;

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
    Peers {
        torrent: PathBuf,
    },
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
            println!("Info Hash: {}", hex::encode(&d));
            println!("Piece Length: {}", t.info.piece_length);
            println!("Piece Hashes:");
            for hash in t.info.pieces.0 {
                println!("{}", hex::encode(hash));
            }
        }
        Command::Peers { torrent } => {
            let f = std::fs::read(torrent).context("open torrent file")?;
            let t: Torrent = serde_bencode::from_bytes(&f).context("parse torrent file")?;
            let info_hash = t.info_hash()?;
            let length = match t.info.keys {
                Keys::SingleFile { length } => length,
                _ => todo!(),
            };

            let request = TrackerRequest {
                info_hash,
                peer_id: "00112233445566778899".to_string(),
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: length,
                compact: 1,
            };

            let mut url = reqwest::Url::parse(&t.announce)?;
            let mut url_params = serde_urlencoded::to_string(&request)?;
            url_params.push_str(&format!("&info_hash={}", request.info_hash));
            url.set_query(Some(&url_params));

            let response = reqwest::blocking::get(url)?.bytes()?;
            let response: TrackerResponse = serde_bencode::from_bytes(&response)?;
            for peer in response.peers.0 {
                println!("{}", peer);
            }
        }
    }

    Ok(())
}
