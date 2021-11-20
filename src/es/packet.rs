use crate::ts::TsPacket;


pub struct PesPacket<'a> {
    pes: &'a [u8]
}

impl<'a> PesPacket<'a> {
    pub fn new(pes: &'a [u8]) -> Self {
        Self { pes }
    }

    #[inline]
    pub fn is_syntax_spec(&self) -> bool {
        match self.pes.get(3) {
            Some(byte) => {
                match byte {
                    0xBC => false,  // program_stream_map
                    0xBE => false,  // padding_stream
                    0xBF => false,  // private_stream_2
                    0xF0 => false,  // ECM
                    0xF1 => false,  // EMM
                    0xF2 => false,  // DSMCC_stream
                    0xF8 => false,  // ITU-T Rec. H.222.1 type E
                    0xFF => false,  // program_stream_directory
                    _ => true,
                }
            },
            None => false
        }
    }

    #[inline]
    pub fn is_pts(&self) -> bool {
        match self.pes.get(7) {
            Some(byte) => { (byte & 0x80) != 0 },
            None => false
        }
    }

    #[inline]
    pub fn get_pts(&self) -> Option<u64> {
        if ! (self.is_pts() & self.is_syntax_spec()) {
            return None
        }

        self.pes.get(9 .. 14).map(
            |pts|
            {
                (u64::from(pts[0] & 0x0E) << 29) |
                (u64::from(pts[1]       ) << 22) |
                (u64::from(pts[2] & 0xFE) << 14) |
                (u64::from(pts[3]       ) <<  7) |
                (u64::from(pts[4]       ) >>  1)
            }
        )
    }
}


impl<'a> From<TsPacket<'a>> for PesPacket<'a> {
    fn from(ts: TsPacket<'a>) -> Self {
        Self::new(ts.get_payload())
    }
}
