use {
    anyhow::{
        anyhow,
        Result,
    },

    crate::{
        ts::TsPacket,
        bytes::*,
    },
};


#[derive(Debug)]
pub struct PatItem {
    /// ISO/IEC 13818-1
    ///
    /// Program_number is a 16-bit field. It specifies the program to which the program_map_PID is applicable.
    /// When set to 0x0000, then the following PID reference shall be the network PID. For all other cases the
    /// value of this field is user defined. This field shall not take any single value more than once within one
    /// version of the Program Association Table.
    ///
    /// NOTE – The program_number may be used as a designation for a broadcast channel, for example.
    pub program_number: u16,
    /// ISO/IEC 13818-1
    ///
    /// The program_map_PID is a 13-bit field specifying the PID of the Transport Stream packets which shall
    /// contain the program_map_section applicable for the program as specified by the program_number. No
    /// program_number shall have more than one program_map_PID assignment. The value of the program_map_PID is
    /// defined by the user, but shall only take values as specified in Table 2-3.
    pub program_map_pid: u16,
}


#[derive(Debug)]
pub struct Pat {
    /// ISO/IEC 13818-1
    ///
    /// This is a 16-bit field which serves as a label to identify this Transport Stream from any other multiplex
    /// within a network. Its value is defined by the user.
    pub transport_stream_id: u16,
    /// ISO/IEC 13818-1
    ///
    /// This 5-bit field is the version number of the whole Program Association Table. The version number shall be
    /// incremented by 1 modulo 32 whenever the definition of the Program Association Table changes. When the
    /// current_next_indicator is set to '1', then the version_number shall be that of the currently applicable
    /// Program Association Table. When the current_next_indicator is set to '0', then the version_number shall be
    /// that of the next applicable Program Association Table.
    pub version_number: u8,
    /// List of PatItems.
    pub items: Vec<PatItem>,
}

impl Pat {
    pub fn new(ts: &TsPacket) -> Result<Self> {
        let payload = ts.get_payload();

        // TODO: pointer_field?
        match payload.get(2 .. 4).map(
            |value|
            usize::from(value.get_u16() & 0x0FFF)
        ) {
            Some(length) => {
                if payload[4 .. ].len() < length {
                    return Err(anyhow!("PAT: part of section is out of ts packet"))
                }

                let transport_stream_id = payload[4 .. ].get_u16();
                let version_number = (payload[6] & 0x3E) >> 1;
                let items = payload[9 .. length].chunks_exact(4).map(
                    |data|
                    PatItem {
                        program_number: data.get_u16(),
                        program_map_pid: data[2 .. ].get_u16() & 0x1FFF,
                    }
                ).collect();

                Ok(Self {
                    transport_stream_id,
                    version_number,
                    items,
                })
            },
            None => Err(anyhow!("PAT: section doesn't contain section_length bytes"))
        }
    }
}
