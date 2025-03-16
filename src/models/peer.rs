#[derive(Debug)]
pub struct PeerMessage {
    pub length_prefix: u32,
    pub id: u8,
    pub payload: Option<Vec<u8>>,
}
