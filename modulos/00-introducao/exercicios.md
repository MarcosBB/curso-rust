# Exercícios — Módulo 00

Soluções em [`solucoes/00-introducao`](../../solucoes/00-introducao/).

## 1. Ambiente pronto

Confira se o ambiente está certo:

```bash
rustc --version
cargo --version
cargo clippy --version
cargo fmt --version
rustup show
```

**Esperado:**

- `rustc` na versão **1.96 ou mais nova** (se não estiver, `rustup update`);
- `clippy` e `rustfmt` instalados;
- em `rustup show`, o `Default host` correto para sua máquina (num Mac Apple Silicon, `aarch64-apple-darwin`).

Depois, abra um projeto no editor e confirme que o `rust-analyzer` está ativo: os tipos inferidos devem aparecer ao lado das variáveis.

## 2. Primeiro projeto

1. Crie um projeto `ola` com `cargo new`.
2. Acrescente `rust-version = "1.96"` no `Cargo.toml`.
3. Troque a mensagem para uma sua.
4. Rode, nesta ordem: `cargo check`, `cargo run`, `cargo build --release`.
5. Encontre os **dois** executáveis gerados dentro de `target/` e rode cada um diretamente, sem o Cargo.

**Esperado:** dois executáveis, um em `target/debug/` e outro em `target/release/`, que imprimem a mesma mensagem.

<details>
<summary>Dica</summary>

No Linux/macOS, rode com `./target/debug/ola`. No Windows, o arquivo termina em `.exe`.

</details>

## 3. Deixe o clippy limpo

Crie um projeto, cole este código no `src/main.rs` e faça o `cargo clippy` terminar **sem nenhum erro nem aviso**, mantendo a mesma saída (os números 1, 2, 3 e o dobro de π):

```rust
fn main() {
    let mut x = 3.1415;
    let v = vec![1, 2, 3];
    for i in 0..v.len() {
        println!("{}", v[i]);
    }
    if v.len() == 0 {
        println!("vazio");
    }
    println!("{}", x * 2.0);
}
```

<details>
<summary>Dica</summary>

Siga o `help:` de cada aviso e rode o `clippy` de novo depois de cada correção. Atenção: uma das correções faz aparecer um aviso **novo**, que não existia antes. Leia com calma o que ele diz.

</details>

## 4. Formate

Cole este código num projeto e faça `cargo fmt --check` passar sem mostrar diferença. Não formate à mão.

```rust
fn area_retangulo(largura:f64,altura:f64)->f64{largura*altura}

fn main(){
let largura=3.0;   let altura = 4.5;
    let area=area_retangulo( largura,altura );
  if area>10.0 { println!("área grande: {area}"); }
        else { println!("área pequena: {area}"); }
}
```

Depois de formatar, compare o antes e o depois. O que o formatador mudou?

## 5. Leia o erro

Para cada mensagem abaixo, responda: (a) qual é o código do erro; (b) em que linha o problema se manifesta; (c) qual é a causa, segundo os rótulos secundários ou as notas; (d) qual correção o compilador sugere. Depois, confira com `rustc --explain`.

**Mensagem A**

```text
error[E0384]: cannot assign twice to immutable variable `total`
 --> src/main.rs:3:5
  |
2 |     let total = 10;
  |         ----- first assignment to `total`
3 |     total = total + 1;
  |     ^^^^^^^^^^^^^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut total = 10;
  |         +++
```

**Mensagem B**

```text
error[E0308]: mismatched types
 --> src/main.rs:1:21
  |
1 | fn dobro(x: i32) -> i32 {
  |    -----            ^^^ expected `i32`, found `()`
  |    |
  |    implicitly returns `()` as its body has no tail or `return` expression
2 |     x * 2;
  |          - help: remove this semicolon to return this value
```

**Mensagem C**

```text
error[E0425]: cannot find value `resultado` in this scope
 --> src/main.rs:7:20
  |
7 |     println!("{}", resultado);
  |                    ^^^^^^^^^ not found in this scope
```

## 6. Debug x release (desafio)

Rode este programa com `cargo run` e depois com `cargo run --release`:

```rust
fn main() {
    let limite: u8 = "255".parse().unwrap();
    let proximo = limite + 1;
    println!("{proximo}");
}
```

O que acontece em cada caso? Por que os resultados são diferentes? (O `"255".parse()` impede o compilador de descobrir o valor durante a compilação. Veremos isso no Módulo 01.)

---

[Índice do módulo](README.md)
