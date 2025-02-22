use crate::models::torrent::Torrent;

use rand::distr::Alphanumeric;
use rand::Rng;
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
