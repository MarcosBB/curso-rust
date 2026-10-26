# Soluções — Módulo 08

Enunciados em [`modulos/08-modulos-e-crates/exercicios.md`](../../modulos/08-modulos-e-crates/exercicios.md).

Cada solução é um projeto Cargo (os exercícios 4 e 5 são workspaces). Para conferir:

```bash
cd nome-do-projeto
cargo test
cargo clippy --all-targets
```

## 1. Conserte a árvore de arquivos

Projeto: [`ex1-geometria/`](ex1-geometria/).

Dois passos:

1. Apagar `src/geometria/mod.rs`. Com ele e `src/geometria.rs`, o módulo `geometria` tinha dois arquivos (E0761). O `geometria.rs` é o que declara os dois filhos, então é ele que fica.
2. Mover `src/circulo.rs` para `src/geometria/circulo.rs`. O `mod circulo;` está em `geometria.rs`, então o arquivo é procurado na pasta `geometria/` (E0583, com o caminho certo no `help`).

```text
árvore de módulos          árvore de arquivos
crate                      src/lib.rs
├── geometria              src/geometria.rs
│   ├── circulo            src/geometria/circulo.rs
│   └── quadrado           src/geometria/quadrado.rs
└── tests (inline)         (dentro de src/lib.rs)
```

## 2. Visibilidade mínima

Projeto: [`ex2-banco/`](ex2-banco/).

| Item | Antes | Depois | Por quê |
|---|---|---|---|
| `mod taxas` | `pub` | privado | só a `conta` usa; não é API |
| `taxas::tarifa_saque` | `pub` | `pub(crate)` | usada por um módulo irmão (`conta`) |
| `taxas::TARIFA_FIXA` | `pub` | privada | só usada dentro de `taxas` |
| `Conta::titular`, `saldo` (campos) | `pub` | privados | lidos pelos métodos `titular()` e `saldo()`; escrever direto quebraria o saldo |
| `Conta::historico` | `pub` | `pub(crate)` | a `auditoria`, outro módulo da crate, precisa ler |
| `Conta::registrar` | `pub` | privado | detalhe interno; deixaria alterar o saldo sem regra |
| `abrir`, `titular()`, `saldo()`, `depositar`, `sacar`, `extrato` | `pub` | `pub` | é a API que os testes usam |

A `auditoria` passou a usar `conta.titular()` e `conta.saldo()` em vez dos campos, e só o histórico ficou `pub(crate)`. Com `pub(crate)` em vez de `pub`, mudar o formato do histórico depois não quebra ninguém fora da crate.

O `lib.rs` da solução traz o bônus: as quatro linhas proibidas como doctests `compile_fail`, que o `cargo test` confere.

## 3. Fachada com `pub use`

Projeto: [`ex3-formas/`](ex3-formas/src/lib.rs).

```rust
mod area;
mod circulo;
mod retangulo;

pub use area::{Area, area_total};
pub use circulo::Circulo;
pub use retangulo::Retangulo;
```

Os módulos viraram privados, e a API é a lista de `pub use`. O `cargo doc` mostra `Area`, `Circulo`, `Retangulo` e `area_total` direto na página da crate, sem módulos. Daqui para frente, dá para dividir `area.rs` ou renomear arquivos sem que ninguém fora da crate perceba. Um doctest `compile_fail` no `lib.rs` confirma que `formas::circulo::Circulo` não é mais acessível.

Os `impl Area for ...` continuam funcionando sem mudança: `circulo` e `retangulo` importam `crate::area::Area`, e um módulo privado da raiz é visível para todos os descendentes da raiz.

## 4. Workspace: biblioteca + CLI

Projeto: [`ex4-boletim/`](ex4-boletim/).

O `Cargo.toml` raiz:

```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
boletim = { path = "crates/boletim" }

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
```

Os membros herdam tudo com `version.workspace = true` (e os outros campos), `boletim.workspace = true` e `[lints] workspace = true`. Três ajustes no código:

- no `main.rs`, `boletim_escolar::` virou `boletim::`, o nome da nova crate;
- o `missing_docs` exigiu um `//!` no `notas.rs` (o módulo é `pub`) e no `main.rs` (a crate binária também é cobrada);
- o `lib.rs` já tinha `//!` e `///` em tudo.

O `Cargo.lock` e o `target/` ficam só na raiz.

## 5. Uma feature aditiva

Projeto: [`ex5-boletim-json/`](ex5-boletim-json/).

Na biblioteca:

```toml
[features]
json = []
```

```rust
#[cfg(feature = "json")]
pub fn para_json(turma: &HashMap<String, Vec<f64>>) -> String {
    let campos: Vec<String> = nomes_ordenados(turma)
        .iter()
        .map(|nome| {
            let notas: Vec<String> = turma[nome].iter().map(|n| n.to_string()).collect();
            format!("\"{nome}\":[{}]", notas.join(","))
        })
        .collect();
    format!("{{{}}}", campos.join(","))
}
```

Na CLI: `boletim = { workspace = true, features = ["json"] }` e um `if` que procura `--json` nos argumentos.

A feature é aditiva: só acrescenta uma função, e o resto da biblioteca não muda com ela ligada ou desligada. Por isso a CLI pode ligá-la sem afetar outro usuário da biblioteca.

Por que `cargo test` na raiz roda 8 testes? Porque o comando seleciona todos os membros, inclusive a CLI, que pede `json`. O resolver unifica as features dos pacotes selecionados, e a biblioteca é compilada com `json` ligada. Com `-p boletim`, a CLI fica de fora e ninguém liga a feature: 6 testes.

## 6. Doctests que não mentem

Projeto: [`ex6-medidas/`](ex6-medidas/src/lib.rs).

O `cargo test --doc` roda 7 doctests: o da crate, os dois de `parse_metros` e `dividir`, o `should_panic`, o `compile_fail`, e os de `Percentual::new` e `aplicar`. Pontos que valem atenção:

- O exemplo de `parse_metros` usa `?` e termina com `# Ok::<(), std::num::ParseFloatError>(())`. A linha não aparece no `cargo doc`, e o leitor vê um exemplo sem `unwrap`.
- O `# Panics` de `dividir` documenta a condição e prova com ` ```should_panic `. O lint `missing_panics_doc` detecta o `assert!` e cobra a seção.
- O `compile_fail` de `Percentual` mostra a garantia do tipo. Ao trocar temporariamente por um bloco comum, o erro é `E0603: tuple struct constructor Percentual is private`, que é o motivo certo. Um erro de digitação também faria o teste passar, então vale conferir.
- ``[`Percentual::new`]`` e ``[`ParseFloatError`]`` são links verificados pelo `cargo doc`.
