//! Benchmarks for standard Bevy equivalents of effect operations.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tavcubed::chunk::mesh::mesh_chunk;
use tavcubed::chunk::voxel::Voxel;
use tavcubed::chunk::{CHUNK_VOXEL_COUNT, Chunk};

fn mesh_chunk_empty(c: &mut Criterion) {
    let chunk = Chunk::default();

    c.bench_function("Mesh chunk empty", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk)),
            BatchSize::SmallInput,
        )
    });
}

fn mesh_chunk_full(c: &mut Criterion) {
    let chunk = Chunk::new([Some(Voxel::default()); CHUNK_VOXEL_COUNT]);

    c.bench_function("Mesh chunk full", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk)),
            BatchSize::SmallInput,
        )
    });
}

fn mesh_chunk_checkerboard(c: &mut Criterion) {
    let chunk = Chunk::checkerboard();

    c.bench_function("Mesh chunk checkerboard", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk)),
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
