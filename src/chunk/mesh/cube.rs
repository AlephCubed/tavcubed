//! Based on: https://github.com/bevyengine/bevy/blob/main/examples/3d/generate_custom_mesh.rs

pub const INDICES_PER_FACE: usize = 6;
pub const VERTICES_PER_FACE: usize = 4;

pub type VertexPos = [f32; 3];

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

#[doc(alias = "face_y_pos")]
#[inline]
pub const fn face_top(x: f32, y: f32, z: f32) -> [VertexPos; VERTICES_PER_FACE] {
    [
        [x - 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y + 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z + 0.5],
    ]
}

#[doc(alias = "face_y_neg")]
#[inline]
pub const fn face_bottom(x: f32, y: f32, z: f32) -> [VertexPos; VERTICES_PER_FACE] {
    [
        [x - 0.5, y - 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z + 0.5],
        [x - 0.5, y - 0.5, z + 0.5],
    ]
}

#[doc(alias = "face_x_pos")]
#[inline]
pub const fn face_right(x: f32, y: f32, z: f32) -> [VertexPos; VERTICES_PER_FACE] {
    [
        [x + 0.5, y - 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z + 0.5],
        [x + 0.5, y + 0.5, z + 0.5],
        [x + 0.5, y + 0.5, z - 0.5],
    ]
}

#[doc(alias = "face_x_neg")]
#[inline]
pub const fn face_left(x: f32, y: f32, z: f32) -> [VertexPos; VERTICES_PER_FACE] {
    [
        [x - 0.5, y - 0.5, z - 0.5],
        [x - 0.5, y - 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z - 0.5],
    ]
}

#[doc(alias = "face_z_pos")]
#[inline]
pub const fn face_back(x: f32, y: f32, z: f32) -> [VertexPos; VERTICES_PER_FACE] {
    [
        [x - 0.5, y - 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z + 0.5],
        [x + 0.5, y + 0.5, z + 0.5],
        [x + 0.5, y - 0.5, z + 0.5],
    ]
}

#[doc(alias = "face_z_neg")]
#[inline]
pub const fn face_front(x: f32, y: f32, z: f32) -> [VertexPos; VERTICES_PER_FACE] {
    [
        [x - 0.5, y - 0.5, z - 0.5],
        [x - 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z - 0.5],
    ]
}
