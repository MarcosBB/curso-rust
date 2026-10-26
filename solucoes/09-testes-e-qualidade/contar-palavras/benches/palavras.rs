use contar_palavras::{contar_manual, contar_split};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn comparar(c: &mut Criterion) {
    // Uns 50 KB de texto com acentos, espaços e quebras de linha.
    let texto = "Minha terra tem palmeiras onde canta o sabiá;\nas aves, que aqui gorjeiam, não gorjeiam como lá.\n"
        .repeat(500);

    c.bench_function("split_whitespace", |b| {
        b.iter(|| contar_split(black_box(&texto)));
    });
    c.bench_function("laço manual", |b| {
        b.iter(|| contar_manual(black_box(&texto)));
    });
}

criterion_group!(benches, comparar);
criterion_main!(benches);
