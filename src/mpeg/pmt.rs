use {
    anyhow::{
        anyhow,
        Result,
    },

    super::StreamType,
    crate::{
        ts::TsPacket,
        bytes::*,
    },
};


#[derive(Debug)]
pub struct PmtItem {
    /// ISO/IEC 13818-1
    ///
    /// This is an 8-bit field specifying the type of program element carried within the packets with the PID
    /// whose value is specified by the elementary_PID. The values of stream_type are specified in Table 2-29.
    ///
    /// NOTE – An ITU-T Rec. H.222.0 | ISO/IEC 13818-1 auxiliary stream is available for data types defined by
    /// this Specification, other than audio, video, and DSM CC, such as Program Stream Directory and Program
    /// Stream Map.
    pub stream_type: u8,
    /// ISO/IEC 13818-1
    ///
    /// This is a 13-bit field specifying the PID of the Transport Stream packets which carry the associated
    /// program element.
    pub elementary_pid: u16,
}

impl PmtItem {
    pub fn get_stream_type(&self) -> StreamType {
        match self.stream_type {
            0x01 | 0x02 | 0x10 | 0x1B | 0x24 => StreamType::Video,
            0x03 | 0x04 | 0x0F | 0x11 => StreamType::Audio,
            _ => StreamType::Data,
        }
    }
}


#[derive(Debug)]
pub struct Pmt {
    /// ISO/IEC 13818-1
    ///
    /// program_number is a 16-bit field. It specifies the program to which the program_map_PID is applicable. One
    /// program definition shall be carried within only one TS_program_map_section. This implies that a program
    /// definition is never longer than 1016 (0x3F8). See Informative Annex C for ways to deal with the cases when
    /// that length is not sufficient. The program_number may be used as a designation for a broadcast channel,
    /// for example. By describing the different program elements belonging to a program, data from different
    /// sources (e.g. sequential events) can be concatenated together to form a continuous set of streams using a
    /// program_number. For examples of applications refer to Annex C.
    pub program_number: u16,
    /// ISO/IEC 13818-1
    ///
    /// This 5-bit field is the version number of the TS_program_map_section. The version number shall be
    /// incremented by 1 modulo 32 when a change in the information carried within the section occurs. Version
    /// number refers to the definition of a single program, and therefore to a single section. When the
    /// current_next_indicator is set to '1', then the version_number shall be that of the currently applicable
    /// TS_program_map_section. When the current_next_indicator is set to '0', then the version_number shall be
    /// that of the next applicable TS_program_map_section.
    pub version_number: u8,
    /// List of PmtItem.
    pub items: Vec<PmtItem>,
}

impl Pmt {
    pub fn new(ts: &TsPacket) -> Result<Self> {
        let payload = ts.get_payload();

        // TODO: pointer_field?
        match payload.get(2 .. 4).map(
            |value|
                usize::from(value.get_u16() & 0x0FFF)
        ) {
            Some(length) => {
                if payload[4 .. ].len() < length {
                    return Err(anyhow!("PMT: part of section is out of ts packet"))
                }

                let program_number = payload[4 .. ].get_u16();
                let version_number = (payload[6] & 0x3E) >> 1;
                let program_info_length = (payload[11 .. ].get_u16() & 0x0FFF) as usize;

                let mut items = Vec::new();
                // Skip descriptor section;
                let mut ptr = 13 + program_info_length;
                // PMT items section parsing.
                while ptr < length - 9 {
                    items.push(
                        PmtItem {
                            stream_type: payload[ptr],
                            elementary_pid: payload[ptr + 1 .. ].get_u16() & 0x1FFF
                        }
                    );

                    let es_info_length = (payload[ptr + 3 .. ].get_u16() & 0x0FFF) as usize;
                    ptr += 5 + es_info_length;
                }

                Ok(Self {
                    program_number,
                    version_number,
                    items,
                })
            },
            None => Err(anyhow!("PMT: section doesn't contain section_length bytes"))
        }
    }
}
