use crate::realm::chunk::{Chunk, ChunkPos, RaycastHit};
use bevy::ecs::system::SystemParam;
use bevy::math::bounding::RayCast3d;
use bevy::prelude::*;

#[derive(SystemParam)]
pub struct VoxelQuery<'w, 's> {
    chunks: Query<'w, 's, (&'static Chunk, &'static ChunkPos)>,
}

impl<'w, 's> VoxelQuery<'w, 's> {
    pub fn cast_ray(&'_ self, mut ray: RayCast3d) -> Option<RaycastHit<'_>> {
        let chunk_pos = ChunkPos::vec_to_chunk_pos(ray.origin);

        println!("Looking for chunk at {chunk_pos}");

        let (chunk, pos) = self
            .chunks
            .iter()
            .filter(|(_, pos)| pos.0 == chunk_pos)
            .next()?;

        ray.origin -= pos.0.as_vec3a();

        let hit = chunk.raycast(ray);

        info!("Ray hit: {:?}", hit);

        hit
    }
}
