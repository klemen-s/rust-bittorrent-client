use crate::models::{torrent::Torrent, tracker::Peer};

use std::net::Ipv4Addr;
use std::{fs, io};

use sha1::{Digest, Sha1};

pub fn read_byte_file(file_name: &String) -> io::Result<Vec<u8>> {
    let data: Vec<u8> = fs::read(file_name)?;
    Ok(data)
}

pub fn parse_torrent_file(data: &[u8]) -> Result<Torrent, serde_bencode::Error> {
    let torrent_file: Torrent = serde_bencode::from_bytes(data)?;
    Ok(torrent_file)
}

pub fn sha1_hex(data: Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn sha1_bytes(data: Vec<u8>) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}
pub fn generate_peer_id() -> [u8; 20] {
    let peer_id: [u8; 20] = *b"KS001122334455667788";
    println!("Generated peer id: KS0011223344556677");

    peer_id
}
pub fn parse_peers(peers: &[u8]) -> Vec<Peer> {
    let mut peer_list = Vec::new();
    for peer in peers.chunks_exact(6) {
        let host = Ipv4Addr::new(peer[0], peer[1], peer[2], peer[3]);
        let port = u16::from_be_bytes([peer[4], peer[5]]);
        peer_list.push(Peer { host, port });
    }

    peer_list
}
