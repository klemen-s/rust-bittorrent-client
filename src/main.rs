pub mod models;
pub mod utils;

use models::torrent::{Info, Torrent};
use sha1::{Digest, Sha1};
use std::{env, panic};
use utils::bencode::decode_bencoded_value;
use utils::torrent::{parse_torrent_file, read_byte_file};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command: &String = &args[1];

    if command == "decode" {
        let encoded_value: &String = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.0);
    } else if command == "info" {
        let torrent_file_name: &String = &args[2];

        let data = match read_byte_file(torrent_file_name) {
            Ok(data) => data,
            Err(error) => panic!("Could not read file '{torrent_file_name}'. Error: {error}"),
        };

        let torrent_file: Torrent = match parse_torrent_file(&data) {
            Ok(data) => data,
            Err(err) => panic!(
                "Could not parse torrent file ({}). Error: {}",
                torrent_file_name, err
            ),
        };

        let info: Info = torrent_file.info;
        let info_bencoded = match serde_bencode::to_bytes(&info) {
            Ok(data) => data,
            Err(err) => panic!("Error while parsing info field to bencode! {}", err),
        };

        let mut hasher = Sha1::new();
        hasher.update(info_bencoded);
        let info_sha1_hex = hex::encode(hasher.finalize());

        println!("Tracker URL: {}", torrent_file.announce.unwrap());
        println!("Length: {}", info.length.unwrap());
        println!("{}", info_sha1_hex);
    } else {
        println!("unknown command: {}", args[1])
    }
}
