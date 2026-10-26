//! Benchmarks da lição 09: `cargo bench` (ou `cargo bench --no-run`, só para compilar).

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn decodificacao(c: &mut Criterion) {
    let simples = "a,b,c,d,e,f,g,h";
    let com_aspas = r#"a,"b,c",d,"e ""f"" g",h"#;

    c.bench_function("decodificar simples", |b| {
        b.iter(|| linha_csv::decodificar(black_box(simples)));
    });
    c.bench_function("decodificar com aspas", |b| {
        b.iter(|| linha_csv::decodificar(black_box(com_aspas)));
    });
}

fn codificacao(c: &mut Criterion) {
    let campos = ["nome", "São Paulo, SP", "diz \"oi\"", "42"];
    c.bench_function("codificar", |b| {
        b.iter(|| linha_csv::codificar(black_box(&campos)));
    });
}

criterion_group!(benches, decodificacao, codificacao);
criterion_main!(benches);
