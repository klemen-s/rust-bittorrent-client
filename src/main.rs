pub mod models;
pub mod utils;

use models::torrent::{Info, Torrent};
use models::tracker::{TrackerRequest, TrackerResponse};

use serde_bencode::value::Value;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::Read;
use std::{env, panic};
use utils::bencode::decode_bencoded_value;
use utils::torrent::{generate_peer_id, parse_peers, parse_torrent_file, read_byte_file};

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
        println!("Piece length: {}", info.piece_length);

        println!("Piece Hashes:");
        let piece_iter = info.pieces.chunks_exact(20);
        for piece in piece_iter {
            println!("{}", hex::encode(piece));
        }
    } else if command == "peers" {
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
        let info_hash: [u8; 20] = hasher.finalize().into();

        let peer_id: String = generate_peer_id();
        let tracker_req: TrackerRequest = TrackerRequest::new(
            torrent_file.announce.unwrap(),
            info_hash,
            peer_id,
            info.length.unwrap(),
        );
        let tracker_request = tracker_req.url_encode();

        let mut res =
            reqwest::blocking::get(&tracker_request).expect("No response from tracker server...");
        println!("Sending request to tracker...");

        let mut buffer: Vec<u8> = Vec::new();
        res.read_to_end(&mut buffer)
            .expect("Unable to read response to buffer......");

        println!("Got tracker response!");
        let decoded: HashMap<String, Value> = match serde_bencode::from_bytes(&buffer) {
            Ok(val) => val,
            Err(_) => panic!("Could not decode tracker response"),
        };

        let peers = match decoded.get("peers") {
            Some(Value::Bytes(peers)) => Some(parse_peers(peers)),
            None => None,
            _ => None,
        };
        let interval = match decoded.get("interval") {
            Some(Value::Int(interval)) => *interval,
            None => 0,
            _ => 0,
        };
        let tracker_response = TrackerResponse { interval, peers };
        println!("Tracker response: {:?}", tracker_response);
    } else {
        println!("unknown command: {}", args[1])
    }
}
