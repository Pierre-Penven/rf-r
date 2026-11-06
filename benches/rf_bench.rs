//! Criterion benchmarks for the hot-path functions.

use criterion::{Criterion, criterion_group, criterion_main};
use rf_r::prelude::*;
use std::hint::black_box;

const BATCH_LEN: usize = 4096;

fn bench_db(c: &mut Criterion) {
    c.bench_function("to_db (ln-based)", |b| b.iter(|| to_db(black_box(123.456))));
    c.bench_function("10*log10 (std reference)", |b| {
        b.iter(|| 10.0 * black_box(123.456_f64).log10());
    });
    c.bench_function("from_db (exp-based)", |b| {
        b.iter(|| from_db(black_box(23.4)));
    });
    c.bench_function("10^(x/10) (std reference)", |b| {
        b.iter(|| 10f64.powf(black_box(23.4_f64) / 10.0));
    });
    c.bench_function("log_sum", |b| {
        b.iter(|| log_sum(black_box(10.0), black_box(7.0)));
    });

    let values: Vec<f64> = (0..BATCH_LEN)
        .map(|i| f64::from(u32::try_from(i % 50).unwrap_or(0)))
        .collect();
    c.bench_function("log_sum_all 4096", |b| {
        b.iter(|| log_sum_all(black_box(&values)));
    });
    c.bench_function("combine_cn_db 4096", |b| {
        b.iter(|| combine_cn_db(black_box(&values)));
    });

    let mut buffer = values.clone();
    c.bench_function("to_db_in_place 4096", |b| {
        b.iter(|| {
            buffer.copy_from_slice(&values);
            to_db_in_place(black_box(&mut buffer));
        });
    });
}

fn bench_fspl(c: &mut Criterion) {
    c.bench_function("fspl_db", |b| {
        b.iter(|| fspl_db(black_box(38_000e3), black_box(12e9)));
    });

    let distances: Vec<f64> = (1..=BATCH_LEN)
        .map(|i| f64::from(u32::try_from(i).unwrap_or(1)) * 1e4)
        .collect();
    let mut out = vec![0.0; BATCH_LEN];
    c.bench_function("fspl_db_batch 4096", |b| {
        b.iter(|| fspl_db_batch(black_box(&distances), black_box(12e9), &mut out));
    });
}

fn bench_link(c: &mut Criterion) {
    let link = Link {
        eirp_dbw: 52.0,
        path_loss_db: 205.5,
        extra_losses_db: 0.5,
        g_over_t_db_k: 20.0,
    };
    c.bench_function("Link::cn_db", |b| {
        b.iter(|| black_box(link).cn_db(black_box(36e6)));
    });
    c.bench_function("slant_range_geo_m", |b| {
        b.iter(|| slant_range_geo_m(black_box(0.5)));
    });
}

criterion_group!(benches, bench_db, bench_fspl, bench_link);
criterion_main!(benches);
