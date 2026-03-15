pub type VertexPos = [f32; 3];

// Taken from: https://github.com/bevyengine/bevy/blob/main/examples/3d/generate_custom_mesh.rs
pub(super) fn cube(x: f32, y: f32, z: f32, index: u32) -> ([u32; 36], [VertexPos; 24]) {
    (
        [
            // top (+y).
            index + 0,
            index + 3,
            index + 1,
            index + 1,
            index + 3,
            index + 2,
            // bottom (-y)
            index + 4,
            index + 5,
            index + 7,
            index + 5,
            index + 6,
            index + 7,
            // right (+x)
            index + 8,
            index + 11,
            index + 9,
            index + 9,
            index + 11,
            index + 10,
            // left (-x)
            index + 12,
            index + 13,
            index + 15,
            index + 13,
            index + 14,
            index + 15,
            // back (+z)
            index + 16,
            index + 19,
            index + 17,
            index + 17,
            index + 19,
            index + 18,
            // forward (-z)
            index + 20,
            index + 21,
            index + 23,
            index + 21,
            index + 22,
            index + 23,
        ],
        [
            // top (+y)
            [x - 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y + 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z + 0.5],
            // bottom (-y)
            [x - 0.5, y - 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z + 0.5],
            [x - 0.5, y - 0.5, z + 0.5],
            // right (+x)
            [x + 0.5, y - 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z + 0.5],
            [x + 0.5, y + 0.5, z + 0.5],
            [x + 0.5, y + 0.5, z - 0.5],
            // left (-x)
            [x - 0.5, y - 0.5, z - 0.5],
            [x - 0.5, y - 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z - 0.5],
            // back (+z)
            [x - 0.5, y - 0.5, z + 0.5],
            [x - 0.5, y + 0.5, z + 0.5],
            [x + 0.5, y + 0.5, z + 0.5],
            [x + 0.5, y - 0.5, z + 0.5],
            // forward (-z)
            [x - 0.5, y - 0.5, z - 0.5],
            [x - 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y + 0.5, z - 0.5],
            [x + 0.5, y - 0.5, z - 0.5],
        ],
    )
}

type QuadVertexIndices = [u32; 6];
type QuadVertexPos = [VertexPos; 4];

#[doc(alias = "face_y_pos")]
#[inline]
fn face_top(x: f32, y: f32, z: f32) -> QuadVertexPos {
    [
        [x - 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y + 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z + 0.5],
    ]
}

#[doc(alias = "face_y_neg")]
#[inline]
fn face_bottom(x: f32, y: f32, z: f32) -> QuadVertexPos {
    [
        [x - 0.5, y - 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z + 0.5],
        [x - 0.5, y - 0.5, z + 0.5],
    ]
}

#[doc(alias = "face_x_pos")]
#[inline]
fn face_right(x: f32, y: f32, z: f32) -> QuadVertexPos {
    [
        [x + 0.5, y - 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z + 0.5],
        [x + 0.5, y + 0.5, z + 0.5],
        [x + 0.5, y + 0.5, z - 0.5],
    ]
}

#[doc(alias = "face_x_neg")]
#[inline]
fn face_left(x: f32, y: f32, z: f32) -> QuadVertexPos {
    [
        [x - 0.5, y - 0.5, z - 0.5],
        [x - 0.5, y - 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z - 0.5],
    ]
}

#[doc(alias = "face_z_pos")]
#[inline]
fn face_back(x: f32, y: f32, z: f32) -> QuadVertexPos {
    [
        [x - 0.5, y - 0.5, z + 0.5],
        [x - 0.5, y + 0.5, z + 0.5],
        [x + 0.5, y + 0.5, z + 0.5],
        [x + 0.5, y - 0.5, z + 0.5],
    ]
}

#[doc(alias = "face_z_neg")]
#[inline]
fn face_front(x: f32, y: f32, z: f32) -> QuadVertexPos {
    [
        [x - 0.5, y - 0.5, z - 0.5],
        [x - 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y + 0.5, z - 0.5],
        [x + 0.5, y - 0.5, z - 0.5],
    ]
}
