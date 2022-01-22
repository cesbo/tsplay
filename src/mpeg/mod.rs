mod pat;
pub use pat::Pat;

pub mod pid;

mod pmt;
pub use pmt::Pmt;


/// MPEG-TS Elementary Stream Types
#[derive(Debug)]
pub enum StreamType {
    /// Video stream:
    ///
    /// Pmt stream_type:
    ///   0x01 - ISO/IEC 11172 Video
    ///   0x02 - ISO/IEC 13818-2 Video
    ///   0x10 - ISO/IEC 14496-2 Visual
    ///   0x1B - ISO/IEC 14496-10 Video | H.264
    ///   0x24 - ISO/IEC 23008-2 Video | H.265
    Video,
    /// Audio stream:
    ///
    /// Pmt stream_type:
    ///   0x03 - ISO/IEC 11172 Audio
    ///   0x04 - ISO/IEC 13818-3 Audio
    ///   0x0F - ISO/IEC 13818-7 Audio (ADTS)
    ///   0x11 - ISO/IEC 14496-3 Audio (LATM)
    Audio,
    /// Private data
    Data,
}
