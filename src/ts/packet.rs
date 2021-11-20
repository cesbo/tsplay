use {
    anyhow::{
        anyhow,
        Result,
    },

    super::is_sync,
};


pub const TS_SYNC_BYTE: u8 = 0x47;
pub const TS_PACKET_SIZE: usize = 188;


pub struct TsPacket<'a> {
    ts: &'a [u8],
}

impl<'a> TsPacket<'a> {
    pub fn new(ts: &'a [u8]) -> Result<Self> {
        if ! is_sync(ts) {
            return Err(
                anyhow!("ts packet must starts with sync byte {:#x}", TS_SYNC_BYTE)
            )
        }

        let ts_len = ts.len();
        if ts_len < TS_PACKET_SIZE {
            return Err(
                anyhow!("ts packet must has {} bytes length, got {}", TS_PACKET_SIZE, ts_len)
            )
        }

        Ok(Self { ts: &ts[ .. TS_PACKET_SIZE] })
    }

    #[inline]
    pub fn get_pid(&self) -> u16 {
        (u16::from(self.ts[1] & 0x1F) << 8) | u16::from(self.ts[2])
    }

    #[inline]
    pub fn is_adaptation(&self) -> bool {
        (self.ts[3] & 0x20) != 0x00
    }

    #[inline]
    pub fn get_adaptation_size(&self) -> u8 {
        self.ts[4]
    }

    #[inline]
    pub fn get_payload_offset(&self) -> u8 {
        if ! self.is_adaptation() {
            4
        } else {
            4 + 1 + self.get_adaptation_size()
        }
    }

    #[inline]
    pub fn get_payload(&self) -> &'a [u8] {
        self.ts[self.get_payload_offset() as usize .. ].as_ref()
    }

    #[inline]
    pub fn get_cc(&self) -> u8 {
        self.ts[3] & 0x0F
    }

    #[inline]
    pub fn is_pes(&self) -> bool {
        let payload = self.get_payload();
        payload.get(0 .. 3) == Some(&[0x00, 0x00, 0x01])
    }
}


#[cfg(test)]
mod test {
    use super::TsPacket;

    // TS Null Packet.
    const NULL_PACKET: &[u8] = &[
        0x47, 0x1F, 0xFF, 0x10, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
    ];

    #[test]
    fn new() {
        let ts = TsPacket::new(NULL_PACKET).unwrap();
    }
}
