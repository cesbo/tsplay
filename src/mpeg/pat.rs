use {
    anyhow::{
        anyhow,
        Result,
    },

    crate::ts::TsPacket,
};


#[derive(Debug, Default)]
pub struct Pat {
    /// This is a 16-bit field which serves as a label to identify this Transport Stream from any other multiplex
    /// within a network. Its value is defined by the user.
    pub transport_stream_id: u16,
    /// This 5-bit field is the version number of the whole Program Association Table. The version number shall be
    /// incremented by 1 modulo 32 whenever the definition of the Program Association Table changes. When the
    /// current_next_indicator is set to '1', then the version_number shall be that of the currently applicable
    /// Program Association Table. When the current_next_indicator is set to '0', then the version_number shall be
    /// that of the next applicable Program Association Table.
    pub version_number: u8,
}

impl Pat {
    pub fn new(ts: &TsPacket) -> Result<Self> {
        let payload = ts.get_payload();

        // TODO: pointer_field?
        match payload.get(2 .. 4).map(
            |value|
            (((value[0] & 0x0F) as usize) << 8) | value[1] as usize
        ) {
            Some(length) => {
                if payload[4 .. ].len() < length {
                    return Err(anyhow!("PAT: part of section is out of ts packet"))
                }
            },
            None => return Err(anyhow!("PAT: section doesn't contain section_length bytes"))
        }

        let transport_stream_id = (payload[4] as u16) << 8 | payload[5] as u16;
        let version_number = (payload[6] & 0x3E) >> 1;

        Ok(Self {
            transport_stream_id,
            version_number,
        })
    }
}
