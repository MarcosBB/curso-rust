# Exercícios — Módulo 08

Soluções em [`solucoes/08-modulos-e-crates`](../../solucoes/08-modulos-e-crates/).

Em todos os exercícios, use `edition = "2024"` e `rust-version = "1.96"` no `Cargo.toml`. O objetivo é sempre `cargo test` passando, e `cargo clippy --all-targets` sem avisos.

## 1. Conserte a árvore de arquivos (fácil)

Lição: [01](01-a-arvore-de-modulos.md).

Crie o pacote com `cargo new --lib geometria`. Alguém começou a organizar os arquivos e deixou isto:

```text
src/
├── lib.rs
├── circulo.rs
├── geometria.rs
└── geometria/
    ├── mod.rs
    └── quadrado.rs
```

`src/lib.rs`:

```rust
pub mod geometria;

#[cfg(test)]
mod tests {
    use super::geometria::{circulo, quadrado};

    #[test]
    fn areas() {
        assert!((circulo::area(1.0) - std::f64::consts::PI).abs() < 1e-12);
        assert_eq!(quadrado::area(3.0), 9.0);
    }
}
```

`src/geometria.rs`:

```rust
pub mod circulo;
pub mod quadrado;
```

`src/geometria/mod.rs` (sobra de uma versão antiga):

```rust
pub mod circulo;
```

`src/circulo.rs`:

```rust
pub fn area(r: f64) -> f64 {
    std::f64::consts::PI * r * r
}
```

`src/geometria/quadrado.rs`:

```rust
pub fn area(lado: f64) -> f64 {
    lado * lado
}
```

Faça o `cargo test` passar **sem mudar o conteúdo de nenhum arquivo**, só movendo ou apagando arquivos. Rode `cargo build` a cada passo e leia o erro. No fim, desenhe a árvore de módulos (`crate`, `geometria`, ...) ao lado da árvore de arquivos.

<details>
<summary>Dica</summary>

São dois erros, um de cada vez: E0761 (dois arquivos para o mesmo módulo) e depois E0583 (o `help` mostra a pasta onde o arquivo deveria estar).

</details>

## 2. Visibilidade mínima (fácil/médio)

Lição: [02](02-visibilidade.md).

Pacote `cargo new --lib banco`. Tudo está `pub`, e qualquer usuário consegue mudar o saldo direto. Reduza a visibilidade ao mínimo sem quebrar os testes de integração, que usam só a API pública.

`src/lib.rs`:

```rust
pub mod auditoria;
pub mod conta;
pub mod taxas;
```

`src/conta.rs`:

```rust
use crate::taxas;

#[derive(Debug)]
pub struct Conta {
    pub titular: String,
    pub saldo: i64,
    pub historico: Vec<i64>,
}

impl Conta {
    pub fn abrir(titular: &str) -> Conta {
        Conta {
            titular: titular.to_string(),
            saldo: 0,
            historico: Vec::new(),
        }
    }

    pub fn titular(&self) -> &str {
        &self.titular
    }

    pub fn saldo(&self) -> i64 {
        self.saldo
    }

    pub fn depositar(&mut self, valor: i64) {
        self.registrar(valor);
    }

    pub fn sacar(&mut self, valor: i64) -> Result<(), String> {
        let total = valor + taxas::tarifa_saque(valor);
        if total > self.saldo {
            return Err(format!("saldo insuficiente: precisa de {total}"));
        }
        self.registrar(-total);
        Ok(())
    }

    pub fn registrar(&mut self, valor: i64) {
        self.saldo += valor;
        self.historico.push(valor);
    }
}
```

`src/taxas.rs`:

```rust
pub const TARIFA_FIXA: i64 = 2;

pub fn tarifa_saque(valor: i64) -> i64 {
    TARIFA_FIXA + valor / 100
}
```

`src/auditoria.rs`:

```rust
use crate::conta::Conta;

pub fn extrato(conta: &Conta) -> String {
    let mut s = format!("{}\n", conta.titular);
    for v in &conta.historico {
        s.push_str(&format!("{v:+}\n"));
    }
    s.push_str(&format!("saldo: {}\n", conta.saldo));
    s
}
```

`tests/api.rs` (não pode mudar):

```rust
use banco::auditoria::extrato;
use banco::conta::Conta;

#[test]
fn deposito_e_saque_com_tarifa() {
    let mut c = Conta::abrir("Ana");
    c.depositar(1000);
    assert_eq!(c.sacar(500), Ok(()));
    assert_eq!(c.saldo(), 1000 - 500 - 7);
    assert_eq!(c.titular(), "Ana");
}

#[test]
fn saque_sem_saldo_falha_e_nao_muda_nada() {
    let mut c = Conta::abrir("Bruno");
    c.depositar(100);
    assert!(c.sacar(100).is_err());
    assert_eq!(c.saldo(), 100);
}

#[test]
fn extrato_lista_os_lancamentos() {
    let mut c = Conta::abrir("Carla");
    c.depositar(300);
    c.sacar(100).unwrap();
    assert_eq!(extrato(&c), "Carla\n+300\n-103\nsaldo: 197\n");
}
```

Ao final, de **fora** da crate, nada disto pode compilar: `c.saldo = 1_000_000`, `c.historico`, `banco::taxas::tarifa_saque(100)` e `c.registrar(50)`. Para cada `pub` que sobrar (ou `pub(crate)` que você criar), escreva uma frase justificando.

<details>
<summary>Dica</summary>

A auditoria precisa ler o histórico, mas ela está em outro módulo. Um campo `pub(crate)` resolve sem expor o campo a quem usa a crate. E o `taxas` não precisa ser um módulo público. Bônus: transforme cada linha proibida num doctest ` ```compile_fail ` (lição 09) no `lib.rs`.

</details>

## 3. Fachada com `pub use` (médio)

Lições: [01](01-a-arvore-de-modulos.md) e [03](03-caminhos-use-e-reexports.md).

Pacote `cargo new --lib formas`. Todo o código está num arquivo só, em módulos públicos:

```rust
// src/lib.rs
pub mod area {
    pub trait Area {
        fn area(&self) -> f64;
    }

    pub fn area_total(formas: &[&dyn Area]) -> f64 {
        formas.iter().map(|f| f.area()).sum()
    }
}

pub mod circulo {
    use crate::area::Area;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Circulo {
        pub raio: f64,
    }

    impl Area for Circulo {
        fn area(&self) -> f64 {
            std::f64::consts::PI * self.raio * self.raio
        }
    }
}

pub mod retangulo {
    use crate::area::Area;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Retangulo {
        pub largura: f64,
        pub altura: f64,
    }

    impl Area for Retangulo {
        fn area(&self) -> f64 {
            self.largura * self.altura
        }
    }
}
```

Os testes em `tests/fachada.rs` usam os nomes curtos e, por isso, ainda não compilam:

```rust
use formas::{Area, Circulo, Retangulo, area_total};

#[test]
fn areas_individuais() {
    assert_eq!(Retangulo { largura: 2.0, altura: 5.0 }.area(), 10.0);
    let c = Circulo { raio: 2.0 };
    assert!((c.area() - 4.0 * std::f64::consts::PI).abs() < 1e-12);
}

#[test]
fn soma_formas_diferentes() {
    let c = Circulo { raio: 1.0 };
    let r = Retangulo { largura: 1.0, altura: 1.0 };
    let total = area_total(&[&c, &r]);
    assert!((total - (std::f64::consts::PI + 1.0)).abs() < 1e-12);
}
```

Faça o seguinte:

1. Mova cada módulo para o próprio arquivo (`src/area.rs`, `src/circulo.rs`, `src/retangulo.rs`).
2. Torne os três módulos **privados** e reexporte na raiz só o que os testes usam.
3. Confirme que `formas::circulo::Circulo` não compila mais de fora da crate.

Bônus: rode `cargo doc --open` e veja onde os itens aparecem.

<details>
<summary>Dica</summary>

O `lib.rs` final tem três `mod` e três `pub use` (um deles com chaves). Os `use crate::area::Area;` dentro dos módulos continuam funcionando: a privacidade de `area` não afeta os irmãos, que são descendentes da raiz.

</details>

## 4. Workspace: biblioteca + CLI (médio)

Lições: [04](04-pacotes-crates-e-alvos.md), [05](05-dependencias-e-semver.md) e [07](07-workspaces.md).

Transforme o projeto [`boletim-escolar`](../01-tour-da-sintaxe/codigo/boletim-escolar/) do Módulo 01 num workspace:

```text
boletim/
├── Cargo.toml          # manifesto virtual
└── crates/
    ├── boletim/        # biblioteca: o lib.rs e o notas.rs de lá
    └── boletim-cli/    # binário: o main.rs de lá
```

Requisitos:

- `resolver = "3"` e `members = ["crates/*"]`;
- `version`, `edition`, `rust-version` e `license` em `[workspace.package]`, herdados pelos dois membros;
- a dependência da CLI na biblioteca declarada em `[workspace.dependencies]`;
- `[workspace.lints.rust]` com `unsafe_code = "forbid"` e `missing_docs = "warn"`, herdado pelos dois membros.

Critérios: `cargo test` na raiz roda os 6 testes do Módulo 01; `cargo run -p boletim-cli` imprime o mesmo boletim de antes; `cargo clippy --all-targets` e `cargo build` sem nenhum aviso.

<details>
<summary>Dica</summary>

Crie o `Cargo.toml` raiz primeiro e depois os membros com `cargo new` dentro dele. O `main.rs` usava `boletim_escolar::...`: o nome da crate mudou. O `missing_docs` vai cobrar documentação de módulos e da crate binária: um `//!` no topo de cada arquivo resolve.

</details>

## 5. Uma feature aditiva (médio)

Lição: [06](06-features.md).

No workspace do exercício 4, acrescente à biblioteca uma feature `json` (desligada por padrão) com a função:

```rust
/// A turma em JSON, com os alunos em ordem alfabética: `{"Ana":[9.5,8],"Bruno":[6]}`.
pub fn para_json(turma: &HashMap<String, Vec<f64>>) -> String
```

Escreva o JSON à mão, com `format!` (sem `serde`). As notas saem com o `Display` padrão do `f64`, que imprime `8` para `8.0`. Testes, para o `mod tests` do `lib.rs`:

```rust
#[cfg(feature = "json")]
#[test]
fn exporta_json_em_ordem() {
    let turma = agrupar(vec![("Bruno", 6.0), ("Ana", 9.5), ("Ana", 8.0)]);
    assert_eq!(para_json(&turma), r#"{"Ana":[9.5,8],"Bruno":[6]}"#);
}

#[cfg(feature = "json")]
#[test]
fn turma_vazia_vira_objeto_vazio() {
    assert_eq!(para_json(&HashMap::new()), "{}");
}
```

Depois, faça a CLI ligar a feature e aceitar `--json`:

```text
$ cargo run -p boletim-cli -- --json
{"Ana":[9.5,8.5],"Bruno":[6,7.5],"Carla":[4,5.5]}
```

Critérios: `cargo test -p boletim` roda 6 testes, `cargo test -p boletim --features json` roda 8, e `cargo clippy --all-targets --all-features` fica sem avisos. Explique por que `cargo test` na raiz também roda 8.

<details>
<summary>Dica</summary>

A feature fica em `[features]` no `Cargo.toml` da biblioteca, e a CLI pede `features = ["json"]` na dependência. Para a ordem, reaproveite `nomes_ordenados`. No `format!`, chaves literais se escrevem `{{` e `}}`.

</details>

## 6. Doctests que não mentem (médio)

Lição: [09](09-documentacao-e-doctests.md).

Pacote `cargo new --lib medidas`, com estes lints no `Cargo.toml`:

```toml
[lints.rust]
missing_docs = "warn"

[lints.clippy]
missing_errors_doc = "warn"
missing_panics_doc = "warn"
```

E este código, ainda sem documentação:

```rust
use std::num::ParseFloatError;

pub fn parse_metros(s: &str) -> Result<f64, ParseFloatError> {
    let s = s.trim();
    s.strip_suffix('m').unwrap_or(s).trim().parse()
}

pub fn dividir(total: u32, partes: u32) -> Vec<u32> {
    assert!(partes > 0, "não dá para dividir em zero partes");
    let base = total / partes;
    let sobra = total % partes;
    (0..partes).map(|i| base + u32::from(i < sobra)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Percentual(u8);

impl Percentual {
    pub fn new(valor: u8) -> Option<Percentual> {
        (valor <= 100).then_some(Percentual(valor))
    }

    pub fn valor(self) -> u8 {
        self.0
    }

    pub fn aplicar(self, x: f64) -> f64 {
        x * f64::from(self.0) / 100.0
    }
}
```

Documente tudo. Requisitos:

- um `//!` para a crate, com um exemplo que usa `dividir` e `Percentual`;
- `parse_metros` com `# Errors` e um exemplo que usa `?` (sem `unwrap`);
- `dividir` com `# Panics` e um exemplo `should_panic`;
- `Percentual` com um exemplo `compile_fail` mostrando que `Percentual(150)` não compila de fora;
- pelo menos um link intradoc.

Critérios: `cargo test --doc` passa, com pelo menos 6 doctests; `cargo clippy` e `cargo doc --no-deps` sem avisos.

<details>
<summary>Dica</summary>

Para o `?`, termine o exemplo com `# Ok::<(), std::num::ParseFloatError>(())`. Para conferir o `compile_fail`, troque-o por um bloco comum e veja o erro que aparece: ele precisa ser o E0603, e não um erro de digitação.

</details>

---

[Índice do módulo](README.md)
