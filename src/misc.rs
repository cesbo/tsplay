use super::ts::TS_PACKET_SIZE;


const MAX_BUF: usize = 7 * TS_PACKET_SIZE;


pub fn offset_calc(start: usize, total: usize) -> (usize, usize) {
    let mut end = start + MAX_BUF;
    if end > total {
        end = total;
    }

    (start, end)
}
