use std::{
    fs::OpenOptions,
    io::{Read, Write},
    net::TcpStream,
};

use bytes::{BufMut, BytesMut};

use crate::{models::peer::PeerMessage, utils::torrent::sha1_hex};

pub fn init_handshake(info_hash: [u8; 20], peer_id: [u8; 20]) -> BytesMut {
    // CLIENT
    // 19 -> 1 byte
    // "BitTorrent protocol" -> 19 bytes
    // 8 bytes (reserved)
    // Download hash (bencoded info) -> 20 bytes
    // Peer ID -> 20 bytes
    // 1 + 19 + 8 + 20 + 20 = 68 bytes

    let mut buf = BytesMut::with_capacity(68);
    buf.put_u8(19);
    buf.put(&b"BitTorrent protocol"[..]);
    buf.put_bytes(0, 8);
    buf.put_slice(&info_hash);
    buf.put_slice(&peer_id);

    buf
}

pub fn read_peer_message(stream: &mut TcpStream) -> PeerMessage {
    let mut length_bytes = [0u8; 4];
    stream
        .read_exact(&mut length_bytes)
        .expect("Should read bytes from peer response buffer...");

    let message_length = u32::from_be_bytes(length_bytes);

    let mut message_id_buf = [0u8; 1];
    stream
        .read_exact(&mut message_id_buf)
        .expect("Should read message id");

    let message_id = message_id_buf[0];

    let payload_size = message_length - 1; // 1B for message_id
    let mut payload = vec![0u8; payload_size as usize];

    stream
        .read_exact(&mut payload)
        .expect("Should get payload data...");

    match message_id {
        1 => PeerMessage {
            length_prefix: message_length,
            id: 1,
            payload: None,
        },
        2 => PeerMessage {
            length_prefix: message_length,
            id: 2,
            payload: None,
        },
        5 => {
            println!("{:?}", payload);
            PeerMessage {
                length_prefix: message_length,
                id: 5,
                payload: Some(payload),
            }
        } // bitfield
        7 => PeerMessage {
            length_prefix: message_length,
            id: 7,
            payload: Some(payload),
        }, // piece
        _ => panic!("No message id found for {message_id}..."),
    }
}

pub fn send_peer_message(
    stream: &mut TcpStream,
    message_id: u8,
    piece_length: u32,
    piece_index: u32,
) {
    match message_id {
        1 => {
            let interested_message: [u8; 5] = [0, 0, 0, 1, 2];
            stream
                .write_all(&interested_message)
                .expect("Should write interested message to peer...");

            read_peer_message(stream);
        }
        6 => {
            let block_size: u32 = 16 * 1024;
            let number_of_blocks = (piece_length as f64 / block_size as f64).ceil() as u32;

            for i in 0..number_of_blocks {
                let mut buf = BytesMut::new();
                let block_offset = i * block_size;
                buf.put_u32(13);
                buf.put_u8(6);

                // payload
                //  index: piece index (zero-based)
                buf.put_u32(piece_index);
                let request_size: u32 = if block_offset + block_size > piece_length {
                    piece_length - block_offset
                } else {
                    block_size
                };
                //  begin: byte offset within the piece (zero-based)
                buf.put_u32(block_offset);
                //
                //  length: the length of the block in bytes
                buf.put_u32(request_size);

                stream
                    .write_all(&buf)
                    .expect("Should write interested message to peer...");

                println!(
                    "Sent request for piece {} block {} (offset: {}, size: {})",
                    piece_index, i, block_offset, request_size
                );
            }

            let mut blocks: Vec<PeerMessage> = Vec::new();
            for _ in 0..number_of_blocks {
                blocks.push(read_peer_message(stream));
            }

            let mut piece: Vec<u8> = Vec::new();
            for block in blocks {
                if let Some(payload) = block.payload {
                    let (_, rest) = payload.split_at(4);
                    let (_, block_data) = rest.split_at(4);
                    piece.extend_from_slice(block_data);
                }
            }

            let mut data_file = OpenOptions::new()
                .append(true)
                .open("data.txt")
                .expect("cannot open file");

            data_file.write_all(&piece).expect("write failed");

            println!("Hashed piece: {}", sha1_hex(piece));
        }
        _ => panic!("Message id for {message_id} not found..."),
    }
}
