use crate::realm::block::data::{BlockFace, VecAxis};
use crate::realm::chunk::{Chunk, DynVoxelGroupRef, VoxelGrid};
use bevy::math::bounding::{Aabb3d, RayCast3d};
use bevy::prelude::*;
use std::ops::Deref;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RaycastHit<'a> {
    pub voxel: DynVoxelGroupRef<'a>,
    pub face: RaycastHitFace,
    pub distance: f32,
}

impl<'a> Deref for RaycastHit<'a> {
    type Target = DynVoxelGroupRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.voxel
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RaycastHitFace {
    Internal,
    Face(BlockFace),
}

impl From<BlockFace> for RaycastHitFace {
    fn from(face: BlockFace) -> Self {
        RaycastHitFace::Face(face)
    }
}

impl Chunk {
    pub const AABB: Aabb3d = Aabb3d {
        min: Vec3A::splat(0.0),
        max: Vec3A::splat(32.0),
    };

    /// Based on [this video](https://www.youtube.com/watch?v=ztkh1r1ioZo&t=787s) by Deadlock.
    pub fn raycast(&'_ self, ray: RayCast3d) -> Option<RaycastHit<'_>> {
        let mut distance = 0.0;

        // Get initial position
        let mut pos = match VoxelGrid::vec_to_pos(ray.origin) {
            // Internal
            Some(pos) => {
                if self.get_ref_pos(pos).is_some() {
                    trace!("Internal hit at {pos}, {distance} from origin.");

                    return Some(RaycastHit {
                        voxel: self.get_ref_pos(pos).into(),
                        face: RaycastHitFace::Internal,
                        distance,
                    });
                }

                pos
            }
            // External
            None => {
                distance += ray.aabb_intersection_at(&Self::AABB)?;
                let point = ray.origin + ray.direction * distance;
                let pos = VoxelGrid::vec_to_pos_clamped(point);

                if self.get_ref_pos(pos).is_some() {
                    trace!("Impact on initial voxel at {pos}, {distance} from origin.");

                    // Todo floating point precision errors.
                    let face = if point.x == 32.0 {
                        BlockFace::Right
                    } else if point.x == 0.0 {
                        BlockFace::Left
                    } else if point.y == 32.0 {
                        BlockFace::Top
                    } else if point.y == 0.0 {
                        BlockFace::Bottom
                    } else if point.z == 32.0 {
                        BlockFace::Back
                    } else if point.z == 0.0 {
                        BlockFace::Front
                    } else {
                        unreachable!()
                    };

                    return Some(RaycastHit {
                        voxel: self.get_ref_pos(pos).into(),
                        face: face.into(),
                        distance,
                    });
                }

                pos
            }
        };

        trace!("Initial voxel found at {pos}, {distance} from origin.");

        let step = ray.direction_recip().map(|a| a.signum()).as_u8vec3();
        let delta = ray.direction_recip().abs();

        let select = ray.direction_recip().map(|a| 0.5 + 0.5 * a.signum());
        let planes = pos.as_vec3a() + select;
        let mut t = (planes - ray.origin) * ray.direction_recip();

        let mut axis = VecAxis::X;

        while distance < ray.max {
            trace!("Testing {pos}. {distance} traveled so far.");

            let voxel = self.voxel_grid.get_ref_pos(pos);

            if voxel.is_some() {
                return Some(RaycastHit {
                    voxel: voxel.into(),
                    face: BlockFace::from_axis(axis, step).flip().into(),
                    distance,
                });
            }

            if t.x < t.y {
                if t.x < t.z {
                    pos.x += step.x;
                    t.x += delta.x;
                    distance += delta.x;
                    axis = VecAxis::X;
                } else {
                    pos.z += step.z;
                    t.z += delta.z;
                    distance += delta.z;
                    axis = VecAxis::Z;
                }
            } else {
                if t.y < t.z {
                    pos.y += step.y;
                    t.y += delta.y;
                    distance += delta.y;
                    axis = VecAxis::Y;
                } else {
                    pos.z += step.z;
                    t.z += delta.z;
                    distance += delta.z;
                    axis = VecAxis::Z;
                }
            }

            if pos.x >= 32 || pos.y >= 32 || pos.z >= 32 {
                return None;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realm::chunk::{Chunk, Voxel};
    use bevy::math::{U8Vec3, u8vec3};

    #[test]
    fn empty() {
        let chunk = Chunk::default();
        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::X, Dir3::X, f32::INFINITY)),
            None
        );
    }

    #[test]
    fn empty_external() {
        let chunk = Chunk::default();
        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::NEG_X, Dir3::X, f32::INFINITY)),
            None
        );
    }

    #[test]
    fn empty_external_non_intersecting() {
        let chunk = Chunk::default();
        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::NEG_X, Dir3::Y, f32::INFINITY)),
            None
        );
    }

    #[test]
    fn full() {
        let chunk = Chunk::full(Voxel::default());
        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::X, Dir3::X, f32::INFINITY)),
            Some(RaycastHit {
                voxel: chunk.get_ref_pos(U8Vec3::X).into(),
                face: RaycastHitFace::Internal,
                distance: 0.0,
            })
        );
    }

    #[test]
    fn full_external() {
        let chunk = Chunk::full(Voxel::default());
        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::NEG_X, Dir3::X, f32::INFINITY)),
            Some(RaycastHit {
                voxel: chunk.get_ref_pos(U8Vec3::default()).into(),
                face: BlockFace::Left.into(),
                distance: 1.0,
            })
        );
    }

    #[test]
    fn full_external_non_intersecting_miss() {
        let chunk = Chunk::full(Voxel::default());
        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::NEG_X, Dir3::Y, f32::INFINITY)),
            None
        );
    }

    #[test]
    fn single() {
        let mut chunk = Chunk::default();
        let pos = u8vec3(1, 31, 0);
        chunk.place_pos(pos, Voxel::default()).unwrap();

        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::X, Dir3::Y, f32::INFINITY)),
            Some(RaycastHit {
                voxel: chunk.voxel_grid.get_ref_pos(pos).into(),
                face: BlockFace::Bottom.into(),
                distance: 31.0,
            })
        );
    }

    #[test]
    fn single_miss() {
        let mut chunk = Chunk::default();
        chunk.place_pos(u8vec3(1, 31, 0), Voxel::default()).unwrap();

        assert_eq!(
            chunk.raycast(RayCast3d::new(Vec3::default(), Dir3::Y, f32::INFINITY)),
            None
        );
    }
}
