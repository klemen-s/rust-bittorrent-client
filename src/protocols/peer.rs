use bytes::{BufMut, BytesMut};

use crate::models::peer::PeerResponse;

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

pub fn parse_peer_response(response: [u8; 68]) -> PeerResponse {
    let peer_res = PeerResponse {
        peer_id: response[48..]
            .try_into()
            .expect("Expected 20 bytes for peer_id"),
    };
    println!("Handshake completed!");
    peer_res
}
