//! Benchmarks for standard Bevy equivalents of effect operations.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tavcubed::realm::block::BlockId;
use tavcubed::realm::block::data::registry::BlockRegistryInner;
use tavcubed::realm::block::data::{Block, BlockTexture};
use tavcubed::realm::chunk::mesh::mesh_chunk;
use tavcubed::realm::chunk::voxel::Voxel;
use tavcubed::realm::chunk::{CHUNK_VOXEL_COUNT, Chunk};

fn get_registry() -> BlockRegistryInner {
    let mut registry = BlockRegistryInner::default();
    registry.register(Block::new(
        BlockId::new("test", "block").unwrap(),
        "block".to_string(),
        BlockTexture::Uniform(0),
    ));
    registry
}

fn mesh_chunk_empty(c: &mut Criterion) {
    let chunk = Chunk::default();
    let registry = get_registry();

    c.bench_function("Mesh chunk empty", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk, &registry)),
            BatchSize::SmallInput,
        )
    });
}

fn mesh_chunk_full(c: &mut Criterion) {
    let chunk = Chunk::new([Some(Voxel::default()); CHUNK_VOXEL_COUNT]);
    let registry = get_registry();

    c.bench_function("Mesh chunk full", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk, &registry)),
            BatchSize::SmallInput,
        )
    });
}

fn mesh_chunk_checkerboard(c: &mut Criterion) {
    let chunk = Chunk::checkerboard();
    let registry = get_registry();

    c.bench_function("Mesh chunk checkerboard", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk, &registry)),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    benches,
    mesh_chunk_empty,
    mesh_chunk_full,
    mesh_chunk_checkerboard,
);
criterion_main!(benches);
