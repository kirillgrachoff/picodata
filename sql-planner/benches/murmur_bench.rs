use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[test]
fn murmur3_tikv() {
    let h = mur3::murmurhash3_x86_32(&[1, 0, 0, 0], 0);
    assert_eq!(h, 4226891818);
}

fn murmur_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("MurmurHash3_Range");

    group.bench_function("x86_32_single", |b| {
        b.iter(|| {
            let bytes = 11235_u64.to_le_bytes();
            black_box(mur3::murmurhash3_x86_32(&bytes, 0));
        });
    });

    group.bench_function("x86_32_range_0_300000", |b| {
        b.iter(|| {
            for num in 0..=300_000_u64 {
                let bytes = num.to_le_bytes();
                black_box(mur3::murmurhash3_x86_32(&bytes, 0));
            }
        });
    });

    group.bench_function("x86_32_range_0_3000", |b| {
        b.iter(|| {
            for num in 0..=3000_u64 {
                let bytes = num.to_le_bytes();
                black_box(mur3::murmurhash3_x86_32(&bytes, 0));
            }
        });
    });

    group.bench_function("x86_32_generate_next_id_200000_times", |b| {
        b.iter(|| {
            let mut counter = 0;
            let mut id = 0_u64;
            while counter < 200000 {
                let bytes = id.to_le_bytes();
                let hash = black_box(mur3::murmurhash3_x86_32(&bytes, 0));
                counter += (hash % 3000 == 0) as i64;
                id += 1;
            }
        });
    });

    group.finish();
}

criterion_group!(benches, murmur_bench);
criterion_main!(benches);
