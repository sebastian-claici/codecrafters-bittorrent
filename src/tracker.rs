use std::fmt;
use std::net::{Ipv4Addr, SocketAddrV4};

use serde::de::{self, Visitor};
use serde::Deserializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TrackerRequest {
    #[serde(skip_serializing)]
    pub info_hash: String,

    pub peer_id: String,
    pub port: u16,
    pub uploaded: usize,
    pub downloaded: usize,
    pub left: usize,
    pub compact: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerResponse {
    pub interval: usize,
    pub peers: Peers,
    // pub complete: usize,
    // pub incomplete: usize,
    // #[serde(rename = "min interval")]
    // pub min_interval: usize,
}

#[derive(Debug, Clone)]
pub struct Peers(pub Vec<SocketAddrV4>);
struct PeersVisitor;

impl<'de> Visitor<'de> for PeersVisitor {
    type Value = Peers;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte string whose length is a multiple of 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 6 != 0 {
            Err(E::custom(
                "length of hash byte string must be a multiple of 20",
            ))
        } else {
            Ok(Peers(
                v.chunks_exact(6)
                    .map(|bytes| {
                        SocketAddrV4::new(
                            Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]),
                            u16::from_be_bytes([bytes[4], bytes[5]]),
                        )
                    })
                    .collect(),
            ))
        }
    }
}

impl<'de> Deserialize<'de> for Peers {
    fn deserialize<D>(deserializer: D) -> Result<Peers, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(PeersVisitor)
    }
}

