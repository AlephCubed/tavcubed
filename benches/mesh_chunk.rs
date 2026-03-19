//! Benchmarks for standard Bevy equivalents of effect operations.

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::hint::black_box;
use tavcubed::chunk::mesh::mesh_chunk;
use tavcubed::chunk::voxel::Voxel;
use tavcubed::chunk::{VoxelBuffer, CHUNK_VOXEL_COUNT};

fn mesh_chunk_empty(c: &mut Criterion) {
    let chunk = VoxelBuffer::default();

    c.bench_function("Mesh chunk empty", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk)),
            BatchSize::SmallInput,
        )
    });
}

fn mesh_chunk_full(c: &mut Criterion) {
    let chunk = VoxelBuffer([Some(Voxel::default()); CHUNK_VOXEL_COUNT]);

    c.bench_function("Mesh chunk full", |b| {
        b.iter_batched(
            || chunk.clone(),
            |chunk| black_box(mesh_chunk(chunk)),
            BatchSize::SmallInput,
        )
    });
}

fn mesh_chunk_checkerboard(c: &mut Criterion) {
    let chunk = VoxelBuffer::checkerboard();

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
