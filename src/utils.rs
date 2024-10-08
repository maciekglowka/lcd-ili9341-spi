pub(crate) fn u16_to_bytes(val: u16) -> (u8, u8) {
    ((val >> 8) as u8, (val & 0xff) as u8)
}

/// Combine RGB channels into 565 RGB format - as u16
pub fn rgb_to_u16(r: u8, g: u8, b: u8) -> u16 {
    let rb = r >> 3;
    let gb = g >> 2;
    let bb = b >> 3;
    (rb as u16) << 11 | (gb as u16) << 5 | bb as u16
}

/// Combine RGB channels into 565 RGB format - as a (u8, u8) tuple
pub fn rgb_to_u8(r: u8, g: u8, b: u8) -> (u8, u8) {
    u16_to_bytes(rgb_to_u16(r, g, b))
}

/// Create a single colored buffer of N/2 pixel length
pub fn color_buffer<const N: usize>(color: u16) -> [u8; N] {
    let (h, l) = u16_to_bytes(color);
    core::array::from_fn(|i| if i % 2 == 0 { h } else { l })
}
