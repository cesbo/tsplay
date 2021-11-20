mod packet;
pub use packet::{
    TsPacket,
    TS_PACKET_SIZE,
};


#[inline]
pub fn is_sync(ts: &[u8]) -> bool {
    match ts.first() {
        Some(byte) => { byte == &packet::TS_SYNC_BYTE },
        None => false
    }
}
