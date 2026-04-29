use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tavcubed::realm::block::BlockId;
use tavcubed::realm::block::data::registry::BlockRegistryInner;
use tavcubed::realm::block::data::{Block, VoxelTexture};
use tavcubed::realm::chunk::mesh::{ChunkLOD, mesh_chunk};
use tavcubed::realm::chunk::{Chunk, OCTREE_DEPTH, Voxel};

fn get_registry() -> BlockRegistryInner {
    let mut registry = BlockRegistryInner::default();
    registry.register(Block::new(
        BlockId::new("test", "block").unwrap(),
        "block".to_string(),
        VoxelTexture::Uniform(0),
    ));
    registry
}

fn mesh_chunk_empty(c: &mut Criterion) {
    let chunk = Chunk::default();
    let registry = get_registry();

    for i in 0..=OCTREE_DEPTH {
        c.bench_function(&format!("Mesh chunk empty at depth {i}"), |b| {
            b.iter_batched(
                || chunk.clone(),
                |chunk| black_box(mesh_chunk(chunk, ChunkLOD::default(), &registry)),
                BatchSize::SmallInput,
            )
        });
    }
}

fn mesh_chunk_full(c: &mut Criterion) {
    let chunk = Chunk::full(Voxel::default());
    let registry = get_registry();

    for i in 0..=OCTREE_DEPTH {
        c.bench_function(&format!("Mesh chunk full at depth {i}"), |b| {
            b.iter_batched(
                || chunk.clone(),
                |chunk| black_box(mesh_chunk(chunk, ChunkLOD::default(), &registry)),
                BatchSize::SmallInput,
            )
        });
    }
}

fn mesh_chunk_checkerboard(c: &mut Criterion) {
    let chunk = Chunk::checkerboard(Some(Voxel::default()), None);
    let registry = get_registry();

    for i in 0..=OCTREE_DEPTH {
        c.bench_function(&format!("Mesh chunk checkerboard at depth {i}"), |b| {
            b.iter_batched(
                || chunk.clone(),
                |chunk| black_box(mesh_chunk(chunk, ChunkLOD::default(), &registry)),
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(
    benches,
    mesh_chunk_empty,
    mesh_chunk_full,
    mesh_chunk_checkerboard,
);
criterion_main!(benches);
