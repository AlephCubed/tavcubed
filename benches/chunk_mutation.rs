use bevy::math::U8Vec3;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tavcubed::realm::chunk::{Chunk, Voxel};

fn place_index(c: &mut Criterion) {
    let chunk = Chunk::default();

    c.bench_function("Place block by index", |b| {
        b.iter_batched(
            || chunk.clone(),
            |mut chunk| black_box(chunk.place(0, Voxel::default())),
            BatchSize::SmallInput,
        )
    });
}

fn place_pos(c: &mut Criterion) {
    let chunk = Chunk::default();

    c.bench_function("Place block by position", |b| {
        b.iter_batched(
            || chunk.clone(),
            |mut chunk| black_box(chunk.place_pos(U8Vec3::default(), Voxel::default())),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, place_index, place_pos);
criterion_main!(benches);
