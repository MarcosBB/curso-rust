# 06 — Features

**Objetivos.** Ao final, você vai saber:

- declarar *features*, escolher as padrão e ligar dependências opcionais com `dep:`;
- compilar código condicionalmente com `#[cfg(feature = "...")]`;
- por que features precisam ser aditivas (a unificação);
- ligar, desligar e inspecionar features na linha de comando.

## O problema

Uma biblioteca de formatos quer suportar JSON, mas nem todo usuário precisa disso, e a dependência de JSON aumenta o tempo de compilação. *Features* resolvem: são opções nomeadas, declaradas no `Cargo.toml`, que ligam partes do código e dependências opcionais. Quem usa a crate escolhe o que quer.

O projeto [`codigo/vendas`](codigo/vendas/) tem três:

```toml
[features]
default = ["moeda"]
# Formata valores como "R$ 12,50".
moeda = []
# Relatório em texto. Precisa da formatação, então liga `moeda` também.
relatorio = ["moeda"]
# Exporta as vendas em JSON. Liga a dependência opcional `serde_json`.
json = ["dep:serde_json"]

[dependencies]
serde_json = { version = "1", optional = true }
```

- Cada feature é um nome e a lista do que ela liga: outras features ou dependências.
- `default` é a lista ligada quando ninguém diz nada.
- `optional = true` faz a dependência só ser baixada e compilada quando alguma feature a liga.

## `#[cfg(feature = "...")]`

Para cada feature ligada, o Cargo passa `--cfg feature="nome"` ao compilador. No código, `#[cfg(...)]` inclui ou remove um item inteiro:

```rust
/// Soma das vendas, em centavos. Existe com qualquer combinação de features.
pub fn total(vendas: &[Venda]) -> u64 {
    vendas.iter().map(|v| v.centavos).sum()
}

/// Formata centavos como `R$ 12,50`.
#[cfg(feature = "moeda")]
pub fn formatar(centavos: u64) -> String {
    format!("R$ {},{:02}", centavos / 100, centavos % 100)
}

/// As vendas como um array JSON.
#[cfg(feature = "json")]
pub fn para_json(vendas: &[Venda]) -> String {
    // ... usa serde_json, que só existe com a feature ligada
}
```

Os testes seguem a mesma regra: `#[cfg(feature = "json")]` em cima de um `#[test]` faz o teste só existir quando a feature está ligada.

Compare com o pré-processador do C++: `#ifdef JSON` trabalha com texto, linha a linha, e um nome digitado errado simplesmente vale "falso". O `cfg` trabalha com itens inteiros, e o Cargo informa ao compilador quais nomes existem. Um erro de digitação vira aviso:

```text
warning: unexpected `cfg` condition value: `relatoro`
  --> src/lib.rs:41:7
   |
41 | #[cfg(feature = "relatoro")]
   |       ^^^^^^^^^^----------
   |                 |
   |                 help: there is a expected value with a similar name: `"relatorio"`
   |
   = note: expected values for `feature` are: `default`, `json`, `moeda`, and `relatorio`
   = help: consider adding `relatoro` as a feature in `Cargo.toml`
...
```

## Usando as features de uma dependência

Quem depende de `vendas` sem pedir nada recebe só a `default`. Usar um item de uma feature desligada dá uma mensagem que diz exatamente o que fazer:

```rust
// ❌ NÃO COMPILA (vendas = { path = "../vendas" }, sem features)
use vendas::Venda;

fn main() {
    let v = [Venda::nova("café", 1250)];
    println!("{}", vendas::relatorio(&v));
}
```

```text
error[E0425]: cannot find function `relatorio` in crate `vendas`
  --> src/main.rs:5:28
   |
 5 |     println!("{}", vendas::relatorio(&v));
   |                            ^^^^^^^^^ not found in `vendas`
   |
note: found an item that was configured out
  --> ../vendas/src/lib.rs:42:8
   |
41 | #[cfg(feature = "relatorio")]
   |       --------------------- the item is gated behind the `relatorio` feature
42 | pub fn relatorio(vendas: &[Venda]) -> String {
   |        ^^^^^^^^^
```

A correção é no manifesto de quem usa:

```toml
[dependencies]
vendas = { path = "../vendas", features = ["relatorio"] }
# ou, sem as padrão:
vendas = { path = "../vendas", default-features = false, features = ["json"] }
```

O caso que você mais vai encontrar na prática é o `serde`, cuja macro `derive` fica atrás de uma feature. Sem `features = ["derive"]`, a mensagem **não** cita a feature:

```text
error: cannot find derive macro `Serialize` in this scope
   --> src/main.rs:3:10
    |
  3 | #[derive(Serialize)]
    |          ^^^^^^^^^
    |
note: `Serialize` is a trait, not a derive macro
...
```

Quando uma macro ou função "deveria existir" e não existe, olhe a seção *Feature flags* da crate no docs.rs. Daí o `cargo add serde --features derive` da lição anterior.

## `dep:` e as features implícitas

Uma dependência opcional cria, sozinha, uma feature com o mesmo nome. Quando alguma feature usa `dep:serde_json`, essa feature implícita deixa de existir, e o nome da dependência fica escondido. É o que a `vendas` faz, e o Cargo explica se alguém tentar o atalho:

```text
error: package `vendas v0.1.0 (…)` does not have feature `serde_json`

help: an optional dependency with that name exists, but the `features` table includes it with the "dep:" syntax so it does not have an implicit feature with that name
Dependency `serde_json` would be enabled by these features:
	- `json`
```

Com isso, você pode trocar `serde_json` por outra crate no futuro sem mudar o nome da feature, e o nome da feature descreve o que ela faz.

Uma feature também pode ligar a feature de uma dependência: `"serde_json/preserve_order"` liga a dependência e a feature dela. `"serde_json?/preserve_order"` só liga a feature se a dependência já estiver ligada por outro motivo.

## Features precisam ser aditivas

Quando duas crates do grafo dependem de `serde`, uma pedindo `derive` e a outra `std`, o Cargo compila o `serde` **uma vez**, com a **união** das features (*feature unification*). Ninguém recebe "só as suas".

A consequência é a regra mais importante sobre features: **ligar uma feature só pode acrescentar**. Ela nunca pode remover uma função, mudar um comportamento ou ser incompatível com outra feature. Se uma feature `sem_std` tirasse coisas, uma crate que precisasse dessas coisas quebraria só porque outra crate do grafo ligou `sem_std`. Por isso o certo é uma feature `std` que acrescenta o suporte à std, e não o contrário.

Os *extras* do pip (`pip install requests[socks]`) são a mesma ideia e seguem a mesma regra: instalar um extra só acrescenta.

## Linha de comando

```bash
cargo test                          # só as padrão
cargo test --no-default-features    # nenhuma
cargo test --features relatorio     # padrão + relatorio
cargo test --all-features           # todas
cargo run -p app --features vendas/relatorio  # feature de uma dependência
cargo tree -e features              # quem liga o quê
```

Um alvo pode exigir features. O exemplo do relatório declara:

```toml
[[example]]
name = "relatorio"
required-features = ["relatorio"]
```

```text
$ cargo run --example relatorio
error: target `relatorio` in package `vendas` requires the features: `relatorio`
Consider enabling them by passing, e.g., `--features="relatorio"`

$ cargo run --example relatorio --features relatorio
café         R$ 12,50
pão           R$ 3,75
queijo       R$ 29,90
TOTAL        R$ 46,15
```

Com N features, existem 2^N combinações. Teste pelo menos as padrão, nenhuma e todas. A ferramenta `cargo hack --feature-powerset` testa todas (Módulo 09).

## Armadilhas comuns

- **Nome de feature digitado errado** no `cfg`: aviso `unexpected_cfgs`. Não ignore.
- **Item atrás de feature desligada**: E0425 com "gated behind". Ligue a feature no manifesto de quem usa.
- **`serde` sem `derive`**: a mensagem não cita a feature; consulte o docs.rs.
- **Feature que remove ou muda comportamento**: quebra outras crates por causa da unificação. Features só acrescentam.
- **Testar só com as padrão**: o código das outras features pode nem compilar.

## Resumo

- `[features]` declara nomes; `default` lista as ligadas por padrão; `optional = true` + `dep:nome` ligam dependências opcionais.
- `#[cfg(feature = "x")]` inclui ou remove um item; nomes errados geram aviso.
- Quem usa escolhe com `features = [...]` e `default-features = false`.
- O Cargo compila cada crate com a união das features pedidas: features precisam ser aditivas.
- `--features`, `--no-default-features`, `--all-features` e `cargo tree -e features`.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Cargo Book: Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [The Rust Reference: Conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Effective Rust, Item 26: Be wary of feature creep](https://effective-rust.com/features.html)

---

[← Lição anterior](05-dependencias-e-semver.md) · [Índice do módulo](README.md) · [Próxima lição →](07-workspaces.md)
