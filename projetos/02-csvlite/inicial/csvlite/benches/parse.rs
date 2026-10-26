//! Benchmark: ler registro a registro (`Reader`) x montar a tabela (`Table`).
//!
//! Rode com `cargo bench -p csvlite` (ou, mais rápido,
//! `cargo bench -p csvlite -- --warm-up-time 1 --measurement-time 3`).

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use csvlite::{Options, Reader, Table};

/// 10 mil registros de 3 campos, um deles entre aspas.
fn input() -> String {
    let rows = (0..10_000).map(|i| format!("{i},nome {i},\"texto, com vírgula {i}\"\n"));
    std::iter::once(String::from("id,nome,descricao\n"))
        .chain(rows)
        .collect()
}

fn bench(c: &mut Criterion) {
    let text = input();
    c.bench_function("reader_iter", |b| {
        b.iter(|| {
            Reader::new(black_box(&text), Options::default())
                .map(|record| record.unwrap().len())
                .sum::<usize>()
        });
    });
    c.bench_function("table_parse", |b| {
        b.iter(|| {
            Table::parse(black_box(&text), Options::default())
                .unwrap()
                .rows()
                .len()
        });
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
