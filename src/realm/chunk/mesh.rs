//! Mesh generation for chunks.

mod material;
mod packed_data;

use crate::realm::block::data::BlockFace;
use crate::realm::block::data::registry::{BlockRegistry, BlockRegistryInner};
use crate::realm::chunk::mesh::material::ChunkMaterial;
use crate::realm::chunk::mesh::packed_data::{VoxelData, pack};
use crate::realm::chunk::{Chunk, ChunkPos, OCTREE_DEPTH, VoxelGroupRef};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::bundle::InsertMode;
use bevy::ecs::system::entity_command::insert;
use bevy::math::U8Vec2;
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexFormat};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam_channel::{Receiver, Sender};

pub struct ChunkMeshPlugin;

impl Plugin for ChunkMeshPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = crossbeam_channel::unbounded();
        app.insert_resource(ChunkMeshChannel { sender, receiver })
            .add_systems(Update, (mesh_changed_chunks, mesh_finished).chain())
            .add_plugins(MaterialPlugin::<ChunkMaterial>::default());
    }
}

pub const INDICES_PER_FACE: usize = 6;
pub const VERTICES_PER_FACE: usize = 4;

#[derive(Component, Default, Debug)]
pub struct ChunkMesh(pub Handle<Mesh>);

impl ChunkMesh {
    pub const ATTRIBUTE_PACKED_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("packed_data", 806567756968, VertexFormat::Uint32x2);
}

#[doc(alias = "ChunkLevelOfDetail")]
#[derive(Component, Deref, Debug, Eq, PartialEq, Clone, Copy)]
pub struct ChunkLOD(u32);

impl Default for ChunkLOD {
    fn default() -> Self {
        Self(OCTREE_DEPTH as u32 + 1)
    }
}

impl ChunkLOD {
    pub fn new(lod: u32) -> Self {
        assert!(
            lod <= OCTREE_DEPTH as u32 + 1,
            "LOD must be less than or equal to {}, got {}",
            OCTREE_DEPTH + 1,
            lod
        );
        Self(lod)
    }

    pub fn set(&mut self, lod: u32) {
        assert!(
            lod <= OCTREE_DEPTH as u32 + 1,
            "LOD must be less than or equal to {}, got {}",
            OCTREE_DEPTH + 1,
            lod
        );
        self.0 = lod;
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

/// A channel to send finished meshes through to be applied.
#[derive(Resource)]
struct ChunkMeshChannel {
    sender: Sender<ChunkMeshFinished>,
    receiver: Receiver<ChunkMeshFinished>,
}

/// A message to send down the [`ChunkMeshChannel`].
struct ChunkMeshFinished {
    chunk: Entity,
    mesh: Mesh,
}

/// Spins up meshing tasks for changed chunks.
fn mesh_changed_chunks(
    channel: Res<ChunkMeshChannel>,
    registry: Res<BlockRegistry>,
    chunks: Query<(Entity, &Chunk, &ChunkLOD), Or<(Changed<Chunk>, Changed<ChunkLOD>)>>,
) {
    let pool = AsyncComputeTaskPool::get();

    for (entity, chunk, lod) in &chunks {
        let sender = channel.sender.clone();

        trace!("Meshing {entity}");

        let chunk = *chunk;
        let lod = *lod;
        let registry = registry.clone();

        pool.spawn(async move {
            _ = sender.send(ChunkMeshFinished {
                chunk: entity,
                mesh: mesh_chunk(chunk, lod, &registry),
            });
        })
        .detach();
    }
}

/// Creates a mesh from a chunk.
pub fn mesh_chunk(chunk: Chunk, lod: ChunkLOD, registry: &BlockRegistryInner) -> Mesh {
    let face_estimate = chunk.len() * 3; // Estimate half faces.

    let mut indices = Vec::with_capacity(face_estimate * INDICES_PER_FACE);
    let mut packed: Vec<[u32; 2]> = Vec::with_capacity(face_estimate * VERTICES_PER_FACE);

    for group in chunk.iter_depth(lod.0) {
        if group.voxel().is_none() {
            continue;
        }

        let data = VoxelData {
            position: group.pos(),
            size: U8Vec2::splat(group.size()),
            texture: registry[group.voxel().unwrap().id].texture,
        };

        if !group.right_is_some() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(data, BlockFace::Right); 4]);
        }

        if !group.left_is_some() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(data, BlockFace::Left); 4]);
        }

        if !group.up_is_some() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(data, BlockFace::Top); 4]);
        }

        if !group.down_is_some() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(data, BlockFace::Bottom); 4]);
        }

        if !group.backward_is_some() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(data, BlockFace::Back); 4]);
        }

        if !group.forward_is_some() {
            indices.extend(get_indices(packed.len() as u32));
            packed.extend([pack(data, BlockFace::Front); 4]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_indices(Indices::U32(indices))
    .with_inserted_attribute(ChunkMesh::ATTRIBUTE_PACKED_DATA, packed)
}

/// Applies finished meshes to changed chunks.
fn mesh_finished(
    mut commands: Commands,
    channel: Res<ChunkMeshChannel>,
    registry: Res<BlockRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ChunkMaterial>>,
    mut chunk_meshes: Query<(&mut ChunkMesh, &ChunkPos)>,
) {
    for msg in channel.receiver.try_iter() {
        let Ok((mut mesh, pos)) = chunk_meshes.get_mut(msg.chunk) else {
            warn!("Mesh finished for deleted chunk with ID {}", msg.chunk);
            continue;
        };

        trace!("Finished meshing chunk at {} with ID {}", pos.0, msg.chunk);

        let handle = meshes.add(msg.mesh);
        mesh.0 = handle.clone();
        commands.entity(msg.chunk).queue_handled(
            insert(
                (
                    Mesh3d(handle),
                    MeshMaterial3d(materials.add(ChunkMaterial {
                        chunk_pos: pos.0,
                        texture_array: registry.textures.clone().unwrap(),
                    })),
                ),
                InsertMode::Replace,
            ),
            |_error, _ctx| {
                error!("Unable to insert new mesh!");
            },
        );
    }
}

#[inline]
#[allow(clippy::identity_op)]
pub const fn get_indices(index: u32) -> [u32; INDICES_PER_FACE] {
    [
        index + 0,
        index + 3,
        index + 1,
        index + 1,
        index + 3,
        index + 2,
    ]
}
