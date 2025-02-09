use crate::models::torrent::Torrent;

use std::{fs, io};

pub fn read_byte_file(file_name: &String) -> io::Result<Vec<u8>> {
    let data: Vec<u8> = fs::read(file_name)?;
    Ok(data)
}

pub fn parse_torrent_file(data: &[u8]) -> Result<Torrent, serde_bencode::Error> {
    let torrent_file: Torrent = serde_bencode::from_bytes(data)?;
    Ok(torrent_file)
}
