const MAX_BUF: usize = 16 * 1024;


pub fn offset_calc(start: usize, total: usize) -> (usize, usize) {
    let mut end = start + MAX_BUF;
    if end > total {
        end = total;
    }

    (start, end)
}
