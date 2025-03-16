use serde_derive::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Write;
use std::net::Ipv4Addr;
use url::Url;
use url_builder::URLBuilder;

#[derive(Debug)]
pub struct TrackerRequest {
    pub tracker_url: String,
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub port: u16,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
    pub compact: u8,
}

impl TrackerRequest {
    pub fn new(
        tracker_url: String,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        left: u32,
    ) -> TrackerRequest {
        TrackerRequest {
            tracker_url,
            info_hash,
            peer_id,
            left,
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            compact: 1,
        }
    }

    fn parse_url(&self) -> (String, String, String) {
        let parsed_url = Url::parse(&self.tracker_url)
            .expect("Could not parse tracker url from torrent file...");
        let protocol = parsed_url.scheme();
        let host = parsed_url
            .host_str()
            .expect("There should be a host in the url, but we could not find it...");
        let path = parsed_url.path();
        (protocol.to_string(), host.to_string(), path.to_string())
    }

    fn url_encode_info_hash(&self) -> String {
        self.info_hash.iter().fold(String::new(), |mut output, b| {
            let _ = write!(output, "%{b:02x}");
            output
        })
    }

    pub fn url_encode(&self) -> String {
        let mut ub = URLBuilder::new();
        let (protocol, host, path) = self.parse_url();
        let url_encoded_info_hash = self.url_encode_info_hash();

        ub.set_protocol(&protocol);
        ub.set_host(format!("{host}/{path}").as_str());
        ub.add_param("info_hash", url_encoded_info_hash.as_str())
            .add_param("peer_id", &String::from_utf8_lossy(&self.peer_id))
            .add_param("port", &self.port.to_string())
            .add_param("uploaded", &self.uploaded.to_string())
            .add_param("downloaded", &self.downloaded.to_string())
            .add_param("left", &self.left.to_string())
            .add_param("compact", &self.compact.to_string());
        ub.build()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrackerResponse {
    pub interval: i64,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Peer {
    pub host: Ipv4Addr,
    pub port: u16,
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}
