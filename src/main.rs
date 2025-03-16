pub mod models;
pub mod protocols;
pub mod utils;

use crate::protocols::peer;
use models::torrent::{Info, Torrent};
use models::tracker::{Peer, TrackerRequest};

use protocols::peer::{read_peer_message, send_peer_message};
use reqwest::blocking::Response;
use serde_bencode::value::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::net::TcpStream;
use std::{env, panic};
use utils::bencode::decode_bencoded_value;
use utils::torrent::{
    generate_peer_id, parse_peers, parse_torrent_file, read_byte_file, sha1_bytes, sha1_hex,
};

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

        let info_sha1_hex = sha1_hex(info_bencoded);

        println!("Tracker URL: {}", torrent_file.announce.unwrap());
        println!("Length: {}", info.length);
        println!("{}", info_sha1_hex);
        println!("Piece length: {}", info.piece_length);

        println!("Piece Hashes:");
        let piece_iter = info.pieces.chunks_exact(20);
        for piece in piece_iter {
            println!("{}", hex::encode(piece));
        }
    } else if command == "download" {
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
        println!("Successfully parsed torrent file!");

        let info: Info = torrent_file.info;
        let info_bencoded = match serde_bencode::to_bytes(&info) {
            Ok(data) => data,
            Err(err) => panic!("Error while parsing info field to bencode! {}", err),
        };

        let info_hash: [u8; 20] = sha1_bytes(info_bencoded);
        let peer_id: [u8; 20] = generate_peer_id();
        let tracker_req: TrackerRequest = TrackerRequest::new(
            torrent_file.announce.unwrap(),
            info_hash,
            peer_id,
            info.length,
        );
        let tracker_request: String = tracker_req.url_encode();

        let mut res: Response =
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

        let peers: Vec<Peer> = match decoded.get("peers") {
            Some(Value::Bytes(peers)) => Some(parse_peers(peers)),
            None => None,
            _ => None,
        }
        .expect("Should have peers...");

        let peer = peers.first().expect("Expected a peer...");
        let handshake_bytes = peer::init_handshake(info_hash, peer_id);
        let mut peer_res = [0u8; 68];

        println!("Connecting to peer {:?} over TCP...", format!("{peer}"));
        let mut stream = TcpStream::connect(format!("{peer}"))
            .expect("Wanted to connect to peer but could not...");

        stream
            .write_all(&handshake_bytes)
            .expect("Should be able to write to TCP connection");
        println!("Reading data from TCP stream...");
        stream
            .read_exact(&mut peer_res)
            .expect("Should be able to read trom TCP connection");

        // Creata file for torrent data
        File::create("data.txt").expect("creation failed");

        // Parse peer response
        let peer_id: [u8; 20] = peer_res[48..68].try_into().expect("");
        println!("Peer response -> Peer ID: {:?}", hex::encode(peer_id));

        let bitfield = read_peer_message(&mut stream).payload.unwrap()[0] as u32;
        send_peer_message(&mut stream, 1, 0, 0);

        let mut available_pieces = Vec::new();
        for i in 0..8 {
            if (bitfield & (1 << (7 - i))) != 0 {
                available_pieces.push(i);
            }
        }

        let last_piece_length = if info.length % info.piece_length > 0 {
            info.length % info.piece_length
        } else {
            info.piece_length
        };
        for (index, piece_index) in available_pieces.iter().enumerate() {
            if index == available_pieces.len() - 1 {
                send_peer_message(&mut stream, 6, last_piece_length, *piece_index);
            } else {
                send_peer_message(&mut stream, 6, info.piece_length, *piece_index);
            }
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}
