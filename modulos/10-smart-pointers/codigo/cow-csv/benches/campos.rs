//! `Vec<String>` x `Vec<Cow<str>>` nas mesmas linhas do benchmark do Projeto 2.
//!
//! Rode com `cargo bench -- --warm-up-time 1 --measurement-time 3`.

use std::hint::black_box;

use cow_csv::{campos, campos_string};
use criterion::{Criterion, criterion_group, criterion_main};

/// 10 mil linhas de 3 campos, um deles entre aspas.
fn linhas() -> Vec<String> {
    (0..10_000)
        .map(|i| format!("{i},nome {i},\"texto, com vírgula {i}\""))
        .collect()
}

fn bench(c: &mut Criterion) {
    let linhas = linhas();
    c.bench_function("campos_string", |b| {
        b.iter(|| {
            linhas
                .iter()
                .map(|l| campos_string(black_box(l)).len())
                .sum::<usize>()
        });
    });
    c.bench_function("campos_cow", |b| {
        b.iter(|| {
            linhas
                .iter()
                .map(|l| campos(black_box(l)).len())
                .sum::<usize>()
        });
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
