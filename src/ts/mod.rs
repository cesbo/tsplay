mod packet;
pub use packet::{
    TsPacket,
    TS_PACKET_SIZE,
};


#[inline]
pub fn is_sync(ts: &[u8]) -> bool {
    if let Some(byte) = ts.first() {
        return *byte == packet::TS_SYNC_BYTE
    }

    false
}
