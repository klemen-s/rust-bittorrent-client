use crate::models::{torrent::Torrent, tracker::Peer};

use rand::distr::Alphanumeric;
use rand::Rng;
use std::net::Ipv4Addr;
use std::{fs, io};

pub fn read_byte_file(file_name: &String) -> io::Result<Vec<u8>> {
    let data: Vec<u8> = fs::read(file_name)?;
    Ok(data)
}

pub fn parse_torrent_file(data: &[u8]) -> Result<Torrent, serde_bencode::Error> {
    let torrent_file: Torrent = serde_bencode::from_bytes(data)?;
    Ok(torrent_file)
}

pub fn generate_peer_id() -> String {
    let mut rng = rand::rng();
    let peer_id: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(20)
        .collect();
    println!("Generated peer id: {peer_id}");

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
