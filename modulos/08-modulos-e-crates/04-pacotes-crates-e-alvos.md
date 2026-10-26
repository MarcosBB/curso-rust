# 04 — Pacotes, crates e alvos

**Objetivos.** Ao final, você vai saber:

- os alvos que um pacote pode ter e onde o Cargo os encontra sozinho;
- rodar um binário ou exemplo específico, e resolver o "qual binário?";
- por que a crate inteira é a unidade de compilação, e o que isso muda em relação ao C++.

## Revisão rápida

A [M01-09](../01-tour-da-sintaxe/09-kit-modulos-lib-e-main.md) definiu: **pacote** é uma pasta com `Cargo.toml`; **crate** é o que o compilador compila de uma vez. Um pacote tem no máximo uma biblioteca e quantos binários quiser. Cada crate que o Cargo compila a partir de um pacote é um **alvo** (*target*).

## Os alvos automáticos

O Cargo descobre os alvos pela posição dos arquivos (*target auto-discovery*). O pacote [`codigo/palavras`](codigo/palavras/) tem um de cada tipo:

```text
palavras/
├── Cargo.toml
├── src/
│   ├── lib.rs          # a biblioteca `palavras`
│   ├── main.rs         # o binário `palavras`
│   └── bin/
│       └── gritar.rs   # o binário `gritar`
├── examples/
│   └── frase.rs        # o exemplo `frase`
└── tests/
    └── api.rs          # um teste de integração
```

| Arquivo | Alvo | Como rodar |
|---|---|---|
| `src/lib.rs` | biblioteca | usada pelos outros alvos |
| `src/main.rs` | binário com o nome do pacote | `cargo run` |
| `src/bin/*.rs` (ou `src/bin/nome/main.rs`) | um binário por arquivo | `cargo run --bin nome` |
| `examples/*.rs` | exemplo | `cargo run --example nome` |
| `tests/*.rs` | teste de integração | `cargo test` ([Módulo 09](../09-testes-e-qualidade/README.md)) |
| `benches/*.rs` | benchmark | `cargo bench` (Módulo 09) |

Todos os alvos usam a biblioteca como uma dependência externa, pelo nome: `palavras::contar(...)`. Cada um é uma crate separada, com a própria raiz.

```bash
cargo run --bin gritar -- o rato roeu
cargo run --example frase
```

```text
     Running `target/debug/gritar o rato roeu`
O RATO ROEU (3 palavras)
...
     Running `target/debug/examples/frase`
"o rato roeu a roupa do rei de Roma"
palavras: 9
mais longa: Some("roupa")
```

Exemplos são o lugar certo para "como usar a biblioteca": eles são compilados pelo `cargo test` (então não apodrecem), mas não pelo `cargo build`, e não são instalados. Em Python, seria a pasta `examples/` que ninguém roda; aqui, ela quebra o build de testes se a API mudar.

### "Qual binário?"

Com mais de um binário, o `cargo run` sem argumentos não sabe qual executar:

```text
error: `cargo run` could not determine which binary to run. Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: gritar, palavras
```

A mensagem dá as duas saídas: `--bin`, ou escolher um padrão no `Cargo.toml`:

```toml
[package]
name = "palavras"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
default-run = "palavras"
```

### Quando escrever `[[bin]]` e `[[example]]`

Só quando a convenção não basta: um nome diferente do arquivo, um caminho fora do padrão, um alvo que precisa de uma *feature* (lição 06) ou um benchmark sem o *harness* padrão (`harness = false`, usado pelo `criterion` no Módulo 09):

```toml
[[bin]]
name = "palavras-cli"
path = "src/cli.rs"
```

### O que o `cargo test` testa

O `cargo test` compila e roda cada alvo com testes. A saída do `palavras` mostra um bloco por alvo, na ordem:

```text
     Running unittests src/lib.rs (target/debug/deps/palavras-7b3363050fc6bf0e)
test tests::conta_palavras ... ok
     Running unittests src/bin/gritar.rs (target/debug/deps/gritar-7a61a1949175e16f)
     Running unittests src/main.rs (target/debug/deps/palavras-5c633ffbf7fa12d0)
     Running tests/api.rs (target/debug/deps/api-2189a676447d1fc2)
test mais_longa_pega_a_primeira_no_empate ... ok
   Doc-tests palavras
```

## A crate é a unidade de compilação

Em C++, a unidade de compilação é o arquivo `.cpp` (a *translation unit*): cada um é compilado sozinho, enxergando o resto só pelos headers, e o *linker* junta tudo no fim. Em Rust, o compilador recebe a **crate inteira** de uma vez. Consequências:

- **Sem headers.** A interface de uma crate é o que é `pub`, e o compilador lê essa informação dos metadados da dependência já compilada.
- **Sem ODR entre arquivos.** Cada item tem um lugar só na árvore e é compilado uma vez por crate. Não existe "a mesma função definida de dois jeitos em dois arquivos".
- **Generics na crate que usa.** Uma função genérica de uma biblioteca é monomorfizada ([Módulo 06](../06-traits-e-generics/README.md)) na crate que a chama com tipos concretos, como um template do C++.
- **Paralelismo por crate.** O Cargo compila crates independentes em paralelo, e o compilador divide uma crate grande internamente. Mesmo assim, dividir um projeto grande em várias crates (lição 07) costuma acelerar o build incremental.

Depois do build, a biblioteca aparece como `target/debug/libpalavras.rlib`. O `.rlib` é o formato de biblioteca **estática** do Rust, ligado dentro de cada executável. Rust não tem uma ABI estável entre crates (a forma binária muda entre versões do compilador), então "biblioteca dinâmica Rust para Rust" não é o caminho normal. Para gerar `.so`/`.dll`/`.a` para C ou outras linguagens, existe a chave `crate-type = ["cdylib"]` ou `["staticlib"]`, assunto do Módulo 15.

| | C++ | Python | Rust |
|---|---|---|---|
| unidade de compilação | arquivo `.cpp` | módulo (arquivo) | crate |
| interface | header `.h` | — | itens `pub` |
| quem entra no build | o que está no `CMakeLists.txt` | o que é importado | o que é alcançado por `mod` a partir da raiz |
| biblioteca | `.a`, `.so`, `.dll` | pacote, *wheel* | `.rlib` estática; `cdylib`/`staticlib` para FFI |

## Armadilhas comuns

- **`cargo run` com vários binários** sem `--bin` nem `default-run`.
- **`crate::` num binário de `src/bin/`** para chegar à biblioteca: como no `main.rs`, use o nome do pacote (`palavras::...`).
- **Achar que `cargo build` compila os exemplos.** Não compila: use `cargo build --examples`, ou o `cargo test`.
- **Teste de integração num pacote só com binário.** `tests/` só enxerga a biblioteca: sem `lib.rs`, não há o que importar (Módulo 09).

## Resumo

- Um pacote tem alvos: uma biblioteca, binários (`src/main.rs`, `src/bin/`), exemplos, testes e benchmarks, descobertos pela posição dos arquivos.
- `cargo run --bin x`, `cargo run --example y`; `default-run` escolhe o padrão.
- Seções `[[bin]]`/`[[example]]` só quando a convenção não serve.
- A crate inteira é a unidade de compilação: sem headers e sem ODR; a biblioteca vira um `.rlib` estático.

## Exercícios

Os exercícios 4 e 5 em [`exercicios.md`](exercicios.md) usam uma biblioteca e um binário, já num workspace.

## Leitura complementar

- [The Cargo Book: Cargo Targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html)
- [The Cargo Book: Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [The Rust Reference: Linkage](https://doc.rust-lang.org/reference/linkage.html)

---

[← Lição anterior](03-caminhos-use-e-reexports.md) · [Índice do módulo](README.md) · [Próxima lição →](05-dependencias-e-semver.md)
