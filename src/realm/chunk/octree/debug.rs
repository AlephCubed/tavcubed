use crate::player::PlayerChunk;
use crate::realm::chunk::{Chunk, ChunkPos, DIAMETER, OCTREE_DEPTH, OctreeNodeFlag, VoxelGroupRef};
use bevy::color::palettes::css::*;
use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use std::fmt::{Debug, Formatter};

pub struct OctreeDebugPlugin;

impl Plugin for OctreeDebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OctreeDebug>()
            .add_systems(PostUpdate, render_octree_debug);
    }
}

#[derive(Resource, Default)]
pub struct OctreeDebug {
    enabled: u8,
}

impl OctreeDebug {
    pub fn set(&mut self, depth: u32) {
        if depth <= OCTREE_DEPTH as u32 {
            self.enabled = 1 << depth;
        }
    }

    pub fn add(&mut self, depth: u32) {
        if depth <= OCTREE_DEPTH as u32 {
            self.enabled |= 1 << depth;
        }
    }

    pub fn reset(&mut self) {
        self.enabled = 0;
    }
}

impl Debug for OctreeDebug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OctreeDebug({:0>8b})", self.enabled)
    }
}

const COLORS: [Srgba; 6] = [WHITE, YELLOW, ORANGE, GREEN, BLUE, PURPLE];

fn render_octree_debug(
    mut gizmos: Gizmos,
    debug: Res<OctreeDebug>,
    player_chunk: Res<PlayerChunk>,
    chunks: Query<(&Chunk, &ChunkPos)>,
) {
    if debug.enabled.count_ones() == 0 {
        return;
    }

    for (chunk, chunk_pos) in &chunks {
        if player_chunk.pos.distance_squared(chunk_pos.0) > 2 {
            continue;
        }

        for depth in 0..=OCTREE_DEPTH {
            if debug.enabled & 1 << depth == 0 {
                continue;
            }

            for r in chunk.octree.iter_depth(depth) {
                if r.voxel.is_none() {
                    continue;
                }

                let pos = Vec3::new(
                    chunk_pos.0.x as f32,
                    chunk_pos.0.y as f32,
                    chunk_pos.0.z as f32,
                ) * Vec3::splat(DIAMETER as f32)
                    + Vec3::new(r.pos().x as f32, r.pos().y as f32, r.pos().z as f32);

                let scale =
                    Vec3::splat(r.size() as f32 + (depth as f32 / OCTREE_DEPTH as f32) / 100.0);

                let mut color = COLORS[depth];

                if r.flags().contains(OctreeNodeFlag::MINORITY) {
                    color = color.darker(0.1);
                } else if r.flags().contains(OctreeNodeFlag::UNIFORM) {
                    color = color.lighter(0.1);
                }

                color.alpha = 1.0;

                gizmos.aabb_3d(
                    Aabb3d::new(pos + scale / 2.0, scale / 2.0),
                    Transform::IDENTITY,
                    color,
                )
            }
        }
    }
}
