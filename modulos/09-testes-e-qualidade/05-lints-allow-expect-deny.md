# 05 — Lints: `allow`, `expect`, `warn`, `deny`, `forbid`

**Objetivos.** Ao final, você vai saber:

- os cinco níveis de lint e o escopo de cada atributo;
- usar `#[expect]` com `reason`, e quando o `allow` é o certo;
- configurar os lints do projeto inteiro na tabela `[lints]` do `Cargo.toml`.

Um *lint* é uma verificação que gera aviso (ou erro) sobre código que compila, mas provavelmente está errado ou fora do estilo: variável sem uso, código morto, `unsafe`. O próprio `rustc` tem dezenas; o clippy (lição 06) acrescenta centenas.

## Os cinco níveis

| Nível | Efeito |
|---|---|
| `allow` | silencia |
| `expect` | silencia, e **avisa se o lint não disparar** |
| `warn` | aviso (o padrão da maioria) |
| `deny` | erro |
| `forbid` | erro, e ninguém mais abaixo pode relaxar |

O atributo vale para o item em que está (`#[...]` sobre uma função, um `impl`, um `mod`) ou para o módulo inteiro, quando é interno (`#![...]` no topo do arquivo; no `lib.rs` ou `main.rs`, vale para a crate toda).

Qualquer nível aceita um `reason`, que aparece na mensagem:

```rust
#![deny(unused_variables, reason = "variável sem uso costuma ser bug aqui")]

fn main() {
    let total = 3;
}
```

```text
error: unused variable: `total`
 --> src/main.rs:4:9
  |
4 |     let total = 3;
  |         ^^^^^ help: if this is intentional, prefix it with an underscore: `_total`
  |
  = note: variável sem uso costuma ser bug aqui
note: the lint level is defined here
 --> src/main.rs:1:9
  |
1 | #![deny(unused_variables, reason = "variável sem uso costuma ser bug aqui")]
  |         ^^^^^^^^^^^^^^^^
```

E o `forbid` não deixa ninguém voltar atrás:

```rust
// ❌ NÃO COMPILA
#![forbid(unsafe_code)]

#[allow(unsafe_code)]
fn main() {
    println!("ok");
}
```

```text
error[E0453]: allow(unsafe_code) incompatible with previous forbid
 --> src/main.rs:3:9
  |
1 | #![forbid(unsafe_code)]
  |           ----------- `forbid` level set here
2 |
3 | #[allow(unsafe_code)]
  |         ^^^^^^^^^^^ overruled by previous forbid
```

## `#[expect]`: o `allow` que não envelhece

O problema do `allow` é que ele fica para sempre. Alguém silencia um aviso, o código muda, o aviso deixa de existir, e o `allow` continua lá, escondendo o próximo problema que aparecer no mesmo lugar.

O `#[expect]` (estável desde a 1.81) silencia **e** confere. Enquanto a função abaixo não é usada, o build fica limpo:

```rust
#[expect(dead_code, reason = "vai ser usada quando o relatório existir")]
fn formatar_relatorio(total: u32) -> String {
    format!("total: {total}")
}

fn main() {
    println!("ok");
}
```

Quando o `main` passa a chamar `formatar_relatorio(3)`, o atributo pede para ser removido:

```text
warning: this lint expectation is unfulfilled
 --> src/main.rs:1:10
  |
1 | #[expect(dead_code, reason = "vai ser usada quando o relatório existir")]
  |          ^^^^^^^^^
  |
  = note: vai ser usada quando o relatório existir
  = note: `#[warn(unfulfilled_lint_expectations)]` on by default
```

Regra prática: quando você precisa silenciar um lint, use `#[expect(lint, reason = "...")]`.

Duas exceções:

- **Lints de ferramenta** (`clippy::...`) só são conferidos quando a ferramenta roda. Um `#[expect(clippy::cast_possible_truncation)]` não gera nada no `cargo build`; o "não cumprido" só aparece no `cargo clippy`.
- **Código compilado em vários contextos**, em que o lint dispara num e não no outro. O caso clássico é o `tests/common/mod.rs` da [lição 04](04-testes-de-integracao.md): o `expect(dead_code)` se cumpre em `api.rs` e falha em `cli.rs`. Código com `#[cfg(...)]` para várias plataformas tem o mesmo problema. Aí, `allow` com `reason`.

## A tabela `[lints]` do `Cargo.toml`

Espalhar `#![deny(...)]` no topo de cada crate é chato e fácil de esquecer. Desde a 1.74, o `Cargo.toml` tem uma seção para isso, que vale para todos os alvos do pacote (biblioteca, binários, testes, exemplos). O projeto [`codigo/qualidade`](codigo/qualidade/) usa esta:

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
# O grupo inteiro com prioridade menor, para que as linhas abaixo ganhem dele.
pedantic = { level = "warn", priority = -1 }
must_use_candidate = "allow"
# Um lint de `restriction`, escolhido a dedo (lição 06).
unwrap_used = "warn"
```

O Cargo transforma cada linha numa flag do compilador (`-W clippy::unwrap-used`), e é por isso que a mensagem do aviso diz `requested on the command line`.

### `priority`

`pedantic` é um **grupo** de lints, e `must_use_candidate` é um lint **dentro** dele. A ordem das linhas na tabela não importa (o TOML não garante ordem). Quem decide é a `priority`: as flags de prioridade menor são passadas primeiro, e as seguintes podem sobrescrevê-las. Sem a prioridade, o clippy recusa a configuração:

```text
error: lint group `pedantic` has the same priority (0) as a lint
  --> Cargo.toml:9:1
   |
 9 | pedantic = "warn"
   | ^^^^^^^^   ------ has an implicit priority of 0
10 | must_use_candidate = "allow"
   | ------------------ has the same priority as this lint
   |
   = note: the order of the lints in the table is ignored by Cargo
...
help: to have lints override the group set `pedantic` to a lower priority
   |
 9 - pedantic = "warn"
 9 + pedantic = { level = "warn", priority = -1 }
```

A regra: **grupo com `priority = -1`**, lints individuais com a prioridade padrão (0).

### Workspaces

Num workspace, a tabela vai no `Cargo.toml` da raiz como `[workspace.lints]`, e cada membro herda com:

```toml
[lints]
workspace = true
```

A herança é tudo ou nada: o membro não pode herdar e acrescentar lints próprios na mesma tabela. O [Módulo 08](../08-modulos-e-crates/README.md) mostra workspaces em detalhe.

## Comparação

Em C++, o equivalente são as flags `-Wall -Wextra -Werror` e os `#pragma GCC diagnostic`, que variam de compilador para compilador. Em Python, os linters são ferramentas externas (ruff, pylint) com comentários `# noqa` ou `# pylint: disable=...`. O ruff tem até um aviso para `noqa` que não silencia nada (`RUF100`), a mesma ideia do `#[expect]`.

## Armadilhas comuns

- **`allow` sem motivo.** Daqui a seis meses ninguém sabe por que está ali. Use `expect` e `reason`.
- **`expect` em código compilado em vários contextos.** Ele fica "não cumprido" em algum deles. Use `allow`.
- **Grupo sem `priority = -1`.** O clippy dá erro (`lint_groups_priority`).
- **`forbid` e depois `allow`.** E0453. Se o projeto precisar de exceções, use `deny`.

## Resumo

- Níveis: `allow`, `expect`, `warn`, `deny`, `forbid`. `#[...]` vale para o item; `#![...]`, para o módulo ou a crate.
- `reason = "..."` documenta a decisão e aparece na mensagem.
- `#[expect]` silencia e avisa quando o lint deixa de disparar. Use `allow` só quando o mesmo código é compilado em contextos diferentes.
- `[lints.rust]` e `[lints.clippy]` no `Cargo.toml` configuram o pacote inteiro. Grupos com `priority = -1`.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Reference: atributos de diagnóstico](https://doc.rust-lang.org/reference/attributes/diagnostics.html) (`allow`, `expect`, `reason`)
- [The Cargo Book: a seção `[lints]`](https://doc.rust-lang.org/cargo/reference/manifest.html#the-lints-section)
- [The rustc book: lint levels](https://doc.rust-lang.org/rustc/lints/levels.html)

---

[← Lição anterior](04-testes-de-integracao.md) · [Índice do módulo](README.md) · [Próxima lição →](06-clippy-e-rustfmt-a-fundo.md)
