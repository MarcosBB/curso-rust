# Soluções — Módulo 00

Enunciados em [`modulos/00-introducao/exercicios.md`](../../modulos/00-introducao/exercicios.md).

## 1. Ambiente pronto

Não há código. A saída de `rustup show` deve ser parecida com esta (num Mac Apple Silicon):

```text
Default host: aarch64-apple-darwin
...
active toolchain
----------------
name: stable-aarch64-apple-darwin
```

## 2. Primeiro projeto

```bash
cargo new ola
cd ola
# edite o Cargo.toml e o src/main.rs
cargo check
cargo run
cargo build --release
./target/debug/ola
./target/release/ola
```

O projeto pronto está em [`modulos/00-introducao/codigo/ola`](../../modulos/00-introducao/codigo/ola/).

## 3. Deixe o clippy limpo

Projeto: [`clippy-limpo/`](clippy-limpo/src/main.rs). As correções, na ordem:

1. `3.1415` vira `std::f64::consts::PI` (o `approx_constant` é **erro**), e sai o `mut`, que não era usado.
2. `vec![1, 2, 3]` vira o array `[1, 2, 3]`: não há motivo para alocar no heap.
3. `for i in 0..v.len()` com `v[i]` vira `for item in &v`.
4. `v.len() == 0` vira `v.is_empty()`...
5. ...e aí aparece o aviso novo `const_is_empty`: como `v` agora é um array de tamanho fixo, `is_empty()` é **sempre** falso. O `if` inteiro era código morto, e o certo é removê-lo.

O passo 5 mostra por que vale rodar o clippy de novo depois de cada correção: uma mudança pode revelar outro problema.

## 4. Formate

Projeto: [`formatado/`](formatado/src/main.rs). O `cargo fmt`:

- coloca espaços em volta de `:`, `->`, `=`, `>` e depois de vírgulas;
- quebra o corpo da função em linhas e indenta com 4 espaços;
- coloca uma instrução por linha;
- junta o `else` na mesma linha do `}` que fecha o `if`;
- tira os espaços de dentro dos parênteses.

## 5. Leia o erro

| | (a) código | (b) linha | (c) causa | (d) correção sugerida |
|---|---|---|---|---|
| A | E0384 | 3 | `total` foi declarada na linha 2 sem `mut`, e variáveis são imutáveis por padrão | `let mut total = 10;` |
| B | E0308 | 1 (tipo de retorno) | o `;` em `x * 2;` faz o corpo não ter expressão final, então a função devolve `()` em vez de `i32` | remover o ponto-e-vírgula |
| C | E0425 | 7 | não existe nenhuma variável `resultado` visível nesse escopo | (nenhuma sugestão: declarar a variável ou corrigir o nome) |

## 6. Debug x release

Com `cargo run` (perfil `dev`):

```text
thread 'main' panicked at src/main.rs:3:19:
attempt to add with overflow
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Com `cargo run --release`:

```text
0
```

`u8` vai de 0 a 255. No perfil `dev`, o Rust verifica overflow em toda operação aritmética e encerra o programa com *panic*. No `release`, essa verificação é desligada por desempenho, e o valor "dá a volta" (*wrapping*): 255 + 1 = 0. Nos dois casos o comportamento é **definido**, ao contrário do overflow de inteiro com sinal em C++, que é comportamento indefinido. No Módulo 01 veremos como deixar a intenção explícita com `checked_add`, `wrapping_add` e `saturating_add`.

Se o valor fosse escrito direto no código (`let limite: u8 = 255;`), o compilador detectaria o overflow **durante a compilação** e recusaria o programa. O `"255".parse()` esconde o valor até a execução.
