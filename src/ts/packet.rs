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

    /// transport_error_indicator
    ///
    /// ISO/IEC 13818-1
    ///
    /// The transport_error_indicator is a 1-bit flag. When set to '1' it indicates that at least 1 uncorrectable
    /// bit error exists in the associated Transport Stream packet. This bit may be set to '1' by entities
    /// external to the transport layer. When set to '1' this bit shall not be reset to '0' unless the bit
    /// value(s) in error have been corrected.
    #[inline]
    pub fn is_error(&self) -> bool {
        (self.ts[1] & 0x80) != 0x00
    }

    /// payload_unit_start_indicator
    ///
    /// ISO/IEC 13818-1
    ///
    /// The payload_unit_start_indicator is a 1-bit flag which has normative meaning for Transport Stream packets
    /// that carry PES packets (refer to 2.4.3.6) or PSI data (refer to 2.4.4).
    ///
    /// When the payload of the Transport Stream packet contains PES packet data, the payload_unit_start_indicator
    /// has the following significance: a '1' indicates that the payload of this Transport Stream packet will
    /// commence with the first byte of a PES packet and a '0' indicates no PES packet shall start in this
    /// Transport Stream packet. If the payload_unit_start_indicator is set to '1', then one and only one PES
    /// packet starts in this Transport Stream packet. This also applies to private streams of stream_type 6
    /// (refer to Table 2-29).
    ///
    /// When the payload of the Transport Stream packet contains PSI data, the payload_unit_start_indicator has
    /// the following significance: if the Transport Stream packet carries the first byte of a PSI section, the
    /// payload_unit_start_indicator value shall be '1', indicating that the first byte of the payload of this
    /// Transport Stream packet carries the pointer_field. If the Transport Stream packet does not carry the first
    /// byte of a PSI section, the payload_unit_start_indicator value shall be '0', indicating that there is no
    /// pointer_field in the payload. Refer to 2.4.4.1 and 2.4.4.2. This also applies to private streams of
    /// stream_type 5 (refer to Table 2-29).
    ///
    /// For null packets the payload_unit_start_indicator shall be set to '0'.
    ///
    /// The meaning of this bit for Transport Stream packets carrying only private data is not defined in this
    /// Specification.
    #[inline]
    pub fn is_pusi(&self) -> bool {
        (self.ts[1] & 0x40) != 0x00
    }

    /// PID
    ///
    /// ISO/IEC 13818-1
    ///
    /// The PID is a 13-bit field, indicating the type of the data stored in the packet payload. PID value 0x0000
    /// is reserved for the Program Association Table (see Table 2-25). PID value 0x0001 is reserved for the
    /// Conditional Access Table (see Table 2-27). PID values 0x0002 – 0x000F are reserved. PID value 0x1FFF is
    /// reserved for null packets (see Table 2-3).
    #[inline]
    pub fn get_pid(&self) -> u16 {
        (u16::from(self.ts[1] & 0x1F) << 8) | u16::from(self.ts[2])
    }

    /// adaptation_field_control
    ///
    /// ISO/IEC 13818-1
    ///
    /// This 2-bit field indicates whether this Transport Stream packet header is followed by an adaptation field
    /// and/or payload (see Table 2-5).
    ///
    /// ITU-T Rec. H.222.0 | ISO/IEC 13818-1 decoders shall discard Transport Stream packets with the
    /// adaptation_field_control field set to a value of '00'. In the case of a null packet the value of the
    /// adaptation_field_control shall be set to '01'.
    #[inline]
    pub fn is_adaptation(&self) -> bool {
        (self.ts[3] & 0x20) != 0x00
    }

    #[inline]
    pub fn is_payload(&self) -> bool {
        (self.ts[3] & 0x10) != 0x00
    }

    /// continuity_counter
    ///
    /// ISO/IEC 13818-1
    ///
    /// The continuity_counter is a 4-bit field incrementing with each Transport Stream packet with the same PID.
    /// The continuity_counter wraps around to 0 after its maximum value. The continuity_counter shall not be
    /// incremented when the adaptation_field_control of the packet equals '00' or '10'.
    ///
    /// In Transport Streams, duplicate packets may be sent as two, and only two, consecutive Transport Stream
    /// packets of the same PID. The duplicate packets shall have the same continuity_counter value as the
    /// original packet and the adaptation_field_control field shall be equal to '01' or '11'. In duplicate
    /// packets each byte of the original packet shall be duplicated, with the exception that in the program clock
    /// reference fields, if present, a valid value shall be encoded.
    ///
    /// The continuity_counter in a particular Transport Stream packet is continuous when it differs by a positive
    /// value of one from the continuity_counter value in the previous Transport Stream packet of the same PID, or
    /// when either of the non- incrementing conditions (adaptation_field_control set to '00' or '10', or
    /// duplicate packets as described above) are met. The continuity counter may be discontinuous when the
    /// discontinuity_indicator is set to '1' (refer to 2.4.3.4). In the case of a null packet the value of the
    /// continuity_counter is undefined.
    #[inline]
    pub fn get_cc(&self) -> u8 {
        self.ts[3] & 0x0F
    }

    /// adaptation_field_length
    ///
    /// ISO/IEC 13818-1
    ///
    /// The adaptation_field_length is an 8-bit field specifying the number of bytes in the adaptation_field
    /// immediately following the adaptation_field_length. The value 0 is for inserting a single stuffing byte in
    /// a Transport Stream packet. When the adaptation_field_control value is '11', the value of the
    /// adaptation_field_length shall be in the range 0 to 182. When the adaptation_field_control value is '10',
    /// the value of the adaptation_field_length shall be 183. For Transport Stream packets carrying PES packets,
    /// stuffing is needed when there is insufficient PES packet data to completely fill the Transport Stream
    /// packet payload bytes. Stuffing is accomplished by defining an adaptation field longer than the sum of the
    /// lengths of the data elements in it, so that the payload bytes remaining after the adaptation field exactly
    /// accommodates the available PES packet data. The extra space in the adaptation field is filled with
    /// stuffing bytes.
    ///
    /// This is the only method of stuffing allowed for Transport Stream packets carrying PES packets. For
    /// Transport Stream packets carrying PSI, an alternative stuffing method is described in 2.4.4.
    #[inline]
    pub fn get_adaptation_size(&self) -> u8 {
        self.ts[4]
    }

    /// Packet payload offset calculation.
    #[inline]
    pub fn get_payload_offset(&self) -> u8 {
        if ! self.is_adaptation() {
            4
        } else {
            4 + 1 + self.get_adaptation_size()
        }
    }

    /// Packet payload getting.
    #[inline]
    pub fn get_payload(&self) -> &'a [u8] {
        self.ts[self.get_payload_offset() as usize .. ].as_ref()
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
