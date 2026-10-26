# 09 — Benchmarks com `criterion`

**Objetivos.** Ao final, você vai saber:

- configurar um benchmark com `criterion` na stable;
- usar `std::hint::black_box` para o otimizador não apagar o que você mede;
- ler `time: [a b c]`, `change:` e os *outliers*, e desconfiar do ruído.

## Por que uma biblioteca

Medir com `Instant::now()` em volta de um laço parece fácil, mas tem três problemas: o código precisa estar otimizado (em `dev`, a medição não vale nada, como avisou a [lição 03 do Módulo 00](../00-introducao/03-cargo-essencial.md)); o otimizador pode apagar um cálculo cujo resultado ninguém usa; e uma medida só não diz nada sem saber quanto ela varia.

O Rust tem um *harness* de benchmark próprio (`#[bench]`), mas ele só funciona na nightly:

```text
error[E0554]: `#![feature]` may not be used on the stable release channel
 --> src/lib.rs:1:1
  |
1 | #![feature(test)]
  | ^^^^^^^^^^^^^^^^^
```

Na stable, o padrão do ecossistema é a crate `criterion`, que faz o aquecimento, coleta amostras, calcula estatísticas e compara com a execução anterior.

## Configurando

```bash
cargo add --dev criterion
```

Os benchmarks ficam em `benches/`, um arquivo por grupo. Cada arquivo precisa de uma seção `[[bench]]` no `Cargo.toml` que desliga o harness padrão, porque o criterion traz o próprio `main`:

```toml
[dev-dependencies]
criterion = "0.8.2"
proptest = "1.12.0"

[[bench]]
name = "linha"     # benches/linha.rs
harness = false
```

O benchmark de [`codigo/linha-csv`](codigo/linha-csv/benches/linha.rs):

```rust
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
```

- `c.bench_function(nome, |b| ...)` registra uma medição.
- `b.iter(|| ...)` roda a *closure* milhões de vezes. O valor que ela devolve é consumido pelo criterion, então o otimizador não pode descartar o resultado.
- `criterion_group!` e `criterion_main!` geram o `main`.

### `black_box`

A entrada é uma constante. Sem cuidado, o otimizador pode calcular `decodificar("a,b,...")` **uma vez, na compilação**, e o benchmark mede o nada. `std::hint::black_box(x)` devolve `x`, mas o compilador é obrigado a tratá-lo como um valor desconhecido.

Use o da **std**. Exemplos antigos (inclusive o livro do criterion) importam `criterion::black_box`, que foi depreciado:

```text
warning: use of deprecated function `criterion::black_box`: use `std::hint::black_box()` instead
 --> benches/linha.rs:4:16
  |
4 | use criterion::black_box;
  |                ^^^^^^^^^
  |
  = note: `#[warn(deprecated)]` on by default
```

## Rodando e lendo

```bash
cargo bench                       # compila em release e roda todos os benchmarks
cargo bench -- codificar          # filtra pelo nome, como cargo test
cargo bench --no-run              # só compila
cargo bench -- --warm-up-time 1 --measurement-time 2   # mais curto, para experimentar
```

Os benchmarks rodam no perfil `bench`, que herda as otimizações do `release`.

```text
     Running benches/linha.rs (target/release/deps/linha-41ede8cdaa4f30d4)
Gnuplot not found, using plotters backend
Benchmarking decodificar simples: Warming up for 1.0000 s
Benchmarking decodificar simples: Collecting 100 samples in estimated 2.0006 s (6.5M iterations)
Benchmarking decodificar simples: Analyzing
decodificar simples     time:   [311.54 ns 317.24 ns 325.37 ns]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) high mild
  10 (10.00%) high severe

Benchmarking decodificar com aspas: ...
decodificar com aspas   time:   [226.78 ns 227.20 ns 227.65 ns]
...
codificar               time:   [237.54 ns 241.47 ns 246.79 ns]
```

O que cada parte diz:

- **`time: [311.54 ns 317.24 ns 325.37 ns]`**: o número do meio é a estimativa do tempo de uma chamada; os dos lados são os limites do intervalo de confiança (o criterion tem 95% de confiança de que o tempo real está ali). Intervalo largo = medição instável.
- **outliers**: amostras muito fora da curva, quase sempre interferência da máquina (outro programa, o sistema operacional). Alguns por cento é normal num computador de uso geral.
- **`Gnuplot not found`**: só um aviso. Os gráficos e o relatório HTML ficam em `target/criterion/` de qualquer jeito.

Um resultado que surpreende: a linha **com aspas** (5 campos) é mais rápida que a **simples** (8 campos). O custo não está em tratar as aspas, e sim em alocar uma `String` por campo. Sem medir, o palpite seria o contrário. (O Projeto 2 volta a isso, e o [Módulo 10](../10-smart-pointers/README.md) mostra como evitar a cópia com `Cow`.)

### `change:` e o ruído

Na segunda execução, o criterion compara com a anterior, guardada em `target/criterion/`:

```text
decodificar simples     time:   [307.93 ns 312.64 ns 321.03 ns]
                        change: [−6.5835% −1.8791% +3.3541%] (p = 0.50 > 0.05)
                        No change in performance detected.
...
codificar               time:   [257.03 ns 265.11 ns 273.64 ns]
                        change: [+10.826% +18.195% +29.998%] (p = 0.00 < 0.05)
                        Performance has regressed.
```

Nenhuma linha de código mudou entre as duas execuções, e mesmo assim o `codificar` "piorou 18%". Isso é ruído: a máquina estava fazendo outra coisa. Lições:

- compare versões **na mesma máquina, em sequência, sem nada pesado rodando**;
- rode de novo antes de acreditar numa diferença pequena;
- benchmark em CI compartilhada é quase só ruído; serve no máximo para pegar regressões grandes.

## Comparação

| | C++ | Python | Rust |
|---|---|---|---|
| biblioteca | Google Benchmark | `timeit`, `pytest-benchmark` | `criterion` (nightly: `#[bench]`) |
| impedir otimização | `benchmark::DoNotOptimize(x)` | não precisa (interpretado) | `std::hint::black_box(x)` |
| estatística | média, desvio, repetições | `timeit` repete; `pytest-benchmark` compara | intervalo de confiança, outliers, `change:` |

O `DoNotOptimize` do Google Benchmark e o `black_box` resolvem o mesmo problema: compiladores otimizadores apagam trabalho que não tem efeito visível. A crate `divan` é uma alternativa mais nova ao criterion (Módulo 17).

## Armadilhas comuns

- **Medir em `dev`.** O `cargo bench` já usa o perfil otimizado; um `Instant::now()` num `cargo run` sem `--release`, não.
- **Esquecer o `black_box`.** O compilador pode calcular o resultado de antemão, e você mede zero.
- **Esquecer o `harness = false`.** Não dá erro nenhum: o arquivo é compilado com o harness padrão, que procura funções `#[bench]`, não acha nenhuma e termina feliz:

  ```text
       Running benches/linha.rs (target/release/deps/linha-41ede8cdaa4f30d4)

  running 0 tests

  test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
  ```

  Se o `cargo bench` terminar em segundos sem nenhum `time:`, confira o `Cargo.toml`.
- **Acreditar numa diferença de 5%.** Com ruído de 18% sem mudar nada, 5% não diz nada.

## Resumo

- Na stable, use `criterion` em `benches/`, com `[[bench]]` + `harness = false`.
- `c.bench_function` + `b.iter`; a entrada passa por `std::hint::black_box` (o `criterion::black_box` está depreciado).
- `time: [baixo estimativa alto]` é um intervalo de confiança; `change:` compara com a execução anterior.
- Outliers e regressões fantasmas são ruído da máquina. Repita antes de concluir.
- Medir derruba palpites: na linha CSV, o custo estava nas alocações, não nas aspas.

## Exercícios

Veja o exercício 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação do `criterion`](https://docs.rs/criterion/latest/criterion/) (o [livro](https://bheisler.github.io/criterion.rs/book/getting_started.html) explica bem a estatística, mas os exemplos são de versões antigas)
- [Documentação de `std::hint::black_box`](https://doc.rust-lang.org/std/hint/fn.black_box.html)
- [The Rust Performance Book: Benchmarking](https://nnethercote.github.io/perf-book/benchmarking.html)

---

[← Lição anterior](08-property-testing-com-proptest.md) · [Índice do módulo](README.md) · [Próxima lição →](10-outras-ferramentas-de-teste.md)
