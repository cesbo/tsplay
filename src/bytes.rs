pub trait Bytes {
    fn get_u16(&self) -> u16;
}


impl Bytes for [u8] {
    #[inline]
    fn get_u16(&self) -> u16 {
        debug_assert!(self.len() >= 2);
        (u16::from(self[0]) << 8) | u16::from(self[1])
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_u16() {
        let data: &[u8] = &[0x12, 0x34];
        assert_eq!(data[0 ..].get_u16(), 0x1234);
    }
}
