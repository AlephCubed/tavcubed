pub const INDICES_PER_FACE: usize = 6;
pub const VERTICES_PER_FACE: usize = 4;

#[inline]
pub const fn get_indices_pos(index: u32) -> [u32; INDICES_PER_FACE] {
    [
        index + 0,
        index + 3,
        index + 1,
        index + 1,
        index + 3,
        index + 2,
    ]
}

#[inline]
pub const fn get_indices_neg(index: u32) -> [u32; INDICES_PER_FACE] {
    [
        index + 0,
        index + 1,
        index + 3,
        index + 1,
        index + 2,
        index + 3,
    ]
}
