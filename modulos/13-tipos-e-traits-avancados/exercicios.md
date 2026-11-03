# Exercícios — Módulo 13

Soluções em [`solucoes/13-tipos-e-traits-avancados`](../../solucoes/13-tipos-e-traits-avancados/).

Em todos os exercícios, crie o projeto com `cargo new --lib` e o **nome indicado** (os doctests usam o nome do crate), acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial em `src/lib.rs`. Nenhum exercício usa crates externas.

O objetivo final é sempre o mesmo: `cargo test` passando (testes **e** doctests) e `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos.

## 1. Dinheiro com operadores (fácil, faça compilar e passe nos testes)

Lições: [01](01-sobrecarga-de-operadores.md) e [02](02-newtype-aliases-never-e-dsts.md).

`cargo new --lib ex1-dinheiro`. `Dinheiro` é um newtype de centavos. Implemente os operadores que os testes usam: `+`, `-`, `-x`, `Dinheiro * i64` **e** `i64 * Dinheiro`, `+=`, e `Sum` para que `iter().copied().sum()` e `iter().sum()` funcionem. Complete o `Display`.

Não implemente `Dinheiro + i64`: somar um número cru a um valor em reais é o erro que o newtype existe para impedir.

```rust
use std::fmt;

/// Valor em centavos de real.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Dinheiro(pub i64);

impl Dinheiro {
    pub fn reais(reais: i64, centavos: i64) -> Self {
        Dinheiro(reais * 100 + centavos)
    }
}

// Implemente aqui: Add, Sub, Neg, Mul<i64>, `i64 * Dinheiro`, AddAssign,
// Sum (de `Dinheiro` e de `&Dinheiro`).

impl fmt::Display for Dinheiro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        todo!("R$ 12,34; negativos como -R$ 0,05")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma_e_subtracao() {
        let a = Dinheiro::reais(10, 50);
        let b = Dinheiro(275);
        assert_eq!(a + b, Dinheiro(1325));
        assert_eq!(a - b, Dinheiro(775));
        assert_eq!(-a, Dinheiro(-1050));
    }

    #[test]
    fn multiplica_dos_dois_lados() {
        let preco = Dinheiro(199);
        assert_eq!(preco * 3, Dinheiro(597));
        assert_eq!(3 * preco, Dinheiro(597));
    }

    #[test]
    fn acumula_no_lugar() {
        let mut total = Dinheiro::default();
        total += Dinheiro(100);
        total += Dinheiro(-30);
        assert_eq!(total, Dinheiro(70));
    }

    #[test]
    fn soma_de_iterador() {
        let itens = [Dinheiro(100), Dinheiro(250), Dinheiro(5)];
        let por_valor: Dinheiro = itens.iter().copied().sum();
        let por_ref: Dinheiro = itens.iter().sum();
        assert_eq!(por_valor, Dinheiro(355));
        assert_eq!(por_ref, Dinheiro(355));
    }

    #[test]
    fn formata_em_reais() {
        assert_eq!(Dinheiro(1234).to_string(), "R$ 12,34");
        assert_eq!(Dinheiro(5).to_string(), "R$ 0,05");
        assert_eq!(Dinheiro(-5).to_string(), "-R$ 0,05");
        assert_eq!(Dinheiro(-100_000).to_string(), "-R$ 1000,00");
    }
}
```

<details>
<summary>Dica</summary>

`iter().sum()` sobre um `&[Dinheiro]` produz itens `&Dinheiro`, então ele precisa de `impl<'a> Sum<&'a Dinheiro> for Dinheiro`, que pode delegar para a outra com `iter.copied().sum()`. No `Display`, trabalhe com `self.0.unsigned_abs()` e escreva o sinal à parte; `{:02}` completa os centavos com zero.

</details>

## 2. Conexão com typestate (médio, faça compilar e passe nos testes)

Lições: [03](03-phantomdata-e-typestate.md) e [04](04-builder-e-idiomas.md).

`cargo new --lib ex2-conexao`. O código inicial é a conexão do [Módulo 04](../04-structs-enums-e-patterns/08-estados-invalidos-irrepresentaveis.md) na versão "estado em tempo de execução": `enviar` numa conexão desconectada compila e devolve `Err`. Reescreva com typestate: `Conexao<Desconectada>`, `Conexao<Conectando>` e `Conexao<Conectada>`, com as transições consumindo `self`.

Os testes usam a API nova, e os **doctests** do topo do arquivo exigem que três usos errados **não compilem** (com o código de erro certo).

```rust
//! Conexão com typestate: cada estado é um tipo.
//!
//! Enviar sem estar conectado não compila:
//!
//! ```compile_fail,E0599
//! use ex2_conexao::Conexao;
//! let mut c = Conexao::new("10.0.0.1:80");
//! c.enviar("oi");
//! ```
//!
//! Nem confirmar uma conexão que já foi confirmada (a transição consome o valor):
//!
//! ```compile_fail,E0382
//! use ex2_conexao::Conexao;
//! let tentando = Conexao::new("10.0.0.1:80").conectar();
//! let a = tentando.confirmar();
//! let b = tentando.confirmar();
//! ```
//!
//! Nem desconectar quem ainda está tentando conectar:
//!
//! ```compile_fail,E0599
//! use ex2_conexao::Conexao;
//! let c = Conexao::new("10.0.0.1:80").conectar().desconectar();
//! ```

// Versão atual: o estado é checado em tempo de execução.
// Reescreva com typestate (`Conexao<Desconectada>`, `Conexao<Conectando>`,
// `Conexao<Conectada>`) para os testes e os doctests acima passarem.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Estado {
    Desconectada,
    Conectando,
    Conectada,
}

#[derive(Debug)]
pub struct Conexao {
    endereco: String,
    estado: Estado,
    tentativas: u32,
    enviadas: Vec<String>,
}

impl Conexao {
    pub fn new(endereco: &str) -> Self {
        Conexao {
            endereco: endereco.to_string(),
            estado: Estado::Desconectada,
            tentativas: 0,
            enviadas: Vec::new(),
        }
    }

    pub fn conectar(&mut self) {
        if self.estado == Estado::Desconectada {
            self.tentativas += 1;
            self.estado = Estado::Conectando;
        }
    }

    pub fn confirmar(&mut self) {
        if self.estado == Estado::Conectando {
            self.estado = Estado::Conectada;
        }
    }

    pub fn falhar(&mut self) {
        if self.estado == Estado::Conectando {
            self.estado = Estado::Desconectada;
        }
    }

    pub fn enviar(&mut self, msg: &str) -> Result<(), String> {
        if self.estado != Estado::Conectada {
            return Err(format!("não conectada ({:?})", self.estado));
        }
        self.enviadas.push(msg.to_string());
        Ok(())
    }

    pub fn desconectar(&mut self) {
        if self.estado == Estado::Conectada {
            self.estado = Estado::Desconectada;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluxo_feliz() {
        let c = Conexao::new("10.0.0.1:80");
        assert_eq!(c.estado(), "desconectada");
        let c = c.conectar();
        assert_eq!(c.estado(), "conectando");
        let mut c = c.confirmar();
        assert_eq!(c.estado(), "conectada");
        c.enviar("oi");
        c.enviar("tchau");
        assert_eq!(c.enviadas(), ["oi", "tchau"]);
        let c = c.desconectar();
        assert_eq!(c.estado(), "desconectada");
        assert_eq!(c.endereco(), "10.0.0.1:80");
    }

    #[test]
    fn falha_volta_a_desconectada_e_conta_tentativas() {
        let c = Conexao::new(String::from("host:1")).conectar().falhar();
        assert_eq!(c.estado(), "desconectada");
        let c = c.conectar().falhar().conectar();
        assert_eq!(c.tentativas(), 3);
        let c = c.confirmar();
        assert_eq!(c.estado(), "conectada");
    }

    #[test]
    fn mensagens_sobrevivem_a_reconexao() {
        let mut c = Conexao::new("h").conectar().confirmar();
        c.enviar("a");
        let mut c = c.desconectar().conectar().confirmar();
        c.enviar("b");
        assert_eq!(c.enviadas(), ["a", "b"]);
    }
}
```

<details>
<summary>Dica</summary>

Uma trait `Estado { const NOME: &'static str; }` dá o `estado()` num `impl<E: Estado> Conexao<E>`. Para não repetir `Conexao { endereco: self.endereco, ... }` em cada transição, escreva um método privado genérico `fn para<F>(self) -> Conexao<F>` num `impl<E> Conexao<E>`. `new` aceita `&str` e `String`: use `impl Into<String>`.

**Desafio extra:** um `ConexaoBuilder` com `tempo_limite` e `tentativas_maximas` opcionais, e o endereço obrigatório verificado na compilação.

</details>

## 3. Extension traits e uma trait selada (médio, passe nos testes)

Lição: [05](05-blanket-extension-e-sealed.md).

`cargo new --lib ex3-extensoes`. Crie as extension traits do módulo `ext` e **sele** a trait `Moeda`, para que o doctest `compile_fail` passe. Implemente `formatar`.

```rust
//! Extension traits e uma trait selada.
//!
//! Outro crate não consegue criar uma moeda nova:
//!
//! ```compile_fail,E0277
//! use ex3_extensoes::moeda::Moeda;
//! struct Iene;
//! impl Moeda for Iene {
//!     const CODIGO: &'static str = "JPY";
//!     const CASAS: u32 = 0;
//! }
//! ```

pub mod ext {
    // Crie as traits `SliceExt<T>` (para `[T]`) e `OptionExt<T>` (para `Option<T>`):
    //
    // - `segundo_maior(&self) -> Option<&T>`, o segundo maior valor DISTINTO (exige `T: Ord`);
    // - `contagem_de(&self, x: &T) -> usize` (exige só `T: PartialEq`);
    // - `ou_erro(self, msg: &str) -> Result<T, String>`.
}

pub mod moeda {
    // Sele esta trait: só este crate pode implementá-la.
    pub trait Moeda {
        const CODIGO: &'static str;
        /// Casas decimais da unidade menor (centavos: 2).
        const CASAS: u32;
    }

    pub struct Real;
    pub struct Dolar;
    pub struct Iene;

    impl Moeda for Real {
        const CODIGO: &'static str = "BRL";
        const CASAS: u32 = 2;
    }
    impl Moeda for Dolar {
        const CODIGO: &'static str = "USD";
        const CASAS: u32 = 2;
    }
    impl Moeda for Iene {
        const CODIGO: &'static str = "JPY";
        const CASAS: u32 = 0;
    }

    /// `formatar::<Real>(1234)` -> "12.34 BRL"; `formatar::<Iene>(500)` -> "500 JPY".
    pub fn formatar<M: Moeda>(unidades_menores: i64) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::ext::{OptionExt, SliceExt};
    use super::moeda::{Dolar, Iene, Real, formatar};

    #[test]
    fn segundo_maior() {
        assert_eq!([3, 9, 4, 9].segundo_maior(), Some(&4));
        assert_eq!(["b", "a"].segundo_maior(), Some(&"a"));
        assert_eq!([7, 7].segundo_maior(), None);
        assert_eq!(Vec::<u8>::new().segundo_maior(), None);
    }

    #[test]
    fn contagem() {
        assert_eq!([1, 2, 1, 1].contagem_de(&1), 3);
        let nomes = [String::from("ana"), String::from("bia")];
        assert_eq!(nomes.contagem_de(&String::from("bia")), 1);
        // f64 é PartialEq: funciona
        assert_eq!([0.5, 0.5].contagem_de(&0.5), 2);
    }

    #[test]
    fn option_ou_erro() {
        assert_eq!(Some(3).ou_erro("faltou"), Ok(3));
        assert_eq!(None::<i32>.ou_erro("faltou"), Err(String::from("faltou")));
    }

    #[test]
    fn moedas() {
        assert_eq!(formatar::<Real>(1234), "12.34 BRL");
        assert_eq!(formatar::<Dolar>(-5), "-0.05 USD");
        assert_eq!(formatar::<Iene>(500), "500 JPY");
    }
}
```

<details>
<summary>Dica</summary>

Para `segundo_maior` aceitar `[T]` com qualquer `T` e só exigir `Ord` quando chamado, ponha o bound no método: `fn segundo_maior(&self) -> Option<&T> where T: Ord;`. Implemente para `[T]`, e `Vec` e arrays ganham o método pela coerção. O selo é um `mod selo { pub trait Selado {} }` privado e `pub trait Moeda: selo::Selado`. Em `formatar`, `10_i64.pow(M::CASAS)` dá a base, e `{:0largura$}` completa com zeros.

</details>

## 4. Quatro erros de lifetime (médio/difícil, faça compilar)

Lições: [07](07-variancia-e-lifetimes-avancados.md) e [08](08-hrtb-e-captura-em-impl-trait.md).

`cargo new --lib ex4-lifetimes`. Cada função tem um erro de compilação diferente, todos vistos no módulo. Conserte **as assinaturas** (os corpos e os testes ficam como estão).

```rust
use std::fmt::Debug;

/// a) Os índices do miolo: sem a primeira e a última posição.
pub fn indices_do_meio(v: &[u8]) -> impl Iterator<Item = usize> {
    1..v.len().saturating_sub(1)
}

/// b) Guarda qualquer valor `Debug` numa lista de caixas.
pub fn guardar<'a, T: Debug>(caixas: &mut Vec<Box<dyn Debug + 'a>>, x: T) {
    caixas.push(Box::new(x));
}

/// c) O maior e o menor de qualquer coleção que itera `&i32`.
pub fn extremos<'a, C>(c: C) -> Option<(i32, i32)>
where
    &'a C: IntoIterator<Item = &'a i32>,
{
    let maior = (&c).into_iter().max()?;
    let menor = (&c).into_iter().min()?;
    Some((*maior, *menor))
}

/// d) Acrescenta um evento ao log.
pub fn registrar(log: &mut Vec<&'static str>, evento: &str) {
    log.push(evento);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn indices_nao_prendem_o_vetor() {
        let mut v = vec![1, 2, 3, 4, 5];
        let it = indices_do_meio(&v);
        v.clear();
        assert_eq!(it.collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn guarda_valores_e_emprestimos() {
        let texto = String::from("emprestado");
        let mut caixas = Vec::new();
        guardar(&mut caixas, 42);
        guardar(&mut caixas, &texto);
        assert_eq!(format!("{caixas:?}"), r#"[42, "emprestado"]"#);
    }

    #[test]
    fn extremos_de_varias_colecoes() {
        assert_eq!(extremos(vec![3, 1, 2]), Some((3, 1)));
        assert_eq!(extremos([7]), Some((7, 7)));
        assert_eq!(extremos(BTreeSet::from([5, -2])), Some((5, -2)));
        assert_eq!(extremos(Vec::new()), None);
    }

    #[test]
    fn registra_eventos_locais() {
        let mut log = vec!["inicio"];
        let evento = format!("login de {}", "ana");
        registrar(&mut log, &evento);
        assert_eq!(log, ["inicio", "login de ana"]);
    }
}
```

<details>
<summary>Dica</summary>

- (a) O iterador não guarda referência a `v`, mas a edition 2024 supõe que sim. Leia a nota do E0502.
- (b) `Box<dyn Debug + 'a>` promete que o conteúdo vive `'a`. Leia o `help:` do E0309.
- (c) O empréstimo é criado **dentro** da função. O `help:` sugere `C: 'a`, que não resolve.
- (d) Leia a nota *"mutable references are invariant over their type parameter"*. O teste passa uma `String` local.

</details>

## 5. Um emprestador com GAT (difícil, passe nos testes)

Lição: [06](06-tipos-associados-avancados-e-gats.md).

`cargo new --lib ex5-emprestador`. Implemente `Emprestador` para `JanelasMut` (janelas **sobrepostas** de tamanho `n`) e para `Palavras` (cada palavra normalizada num buffer reaproveitado, sem alocar uma `String` por palavra). Depois, `contar_com_prefixo` e `propagar` usando os dois.

```rust
/// Um "iterador" cujos itens podem emprestar do próprio emprestador.
pub trait Emprestador {
    type Item<'a>
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<Self::Item<'_>>;
}

/// Janelas mutáveis e sobrepostas de tamanho `n`.
pub struct JanelasMut<'d, T> {
    dados: &'d mut [T],
    n: usize,
    pos: usize,
}

impl<'d, T> JanelasMut<'d, T> {
    pub fn new(dados: &'d mut [T], n: usize) -> Self {
        assert!(n > 0, "janela de tamanho zero");
        JanelasMut { dados, n, pos: 0 }
    }
}

// impl Emprestador for JanelasMut ...

/// Palavras normalizadas (minúsculas, só letras e dígitos), num buffer reaproveitado.
/// "Palavras" que eram só pontuação (como "--") são puladas.
pub struct Palavras<'t> {
    resto: std::str::SplitWhitespace<'t>,
    buf: String,
}

impl<'t> Palavras<'t> {
    pub fn new(texto: &'t str) -> Self {
        Palavras {
            resto: texto.split_whitespace(),
            buf: String::new(),
        }
    }
}

// impl Emprestador for Palavras ...

/// Quantas palavras (normalizadas) começam com `prefixo`.
pub fn contar_com_prefixo(texto: &str, prefixo: &str) -> usize {
    todo!()
}

/// Em cada janela de tamanho `n`, soma o primeiro elemento ao último.
/// Com `n = 2`, é a soma acumulada.
pub fn propagar(v: &mut [i64], n: usize) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn janelas_de_tamanho_n() {
        let mut v = [1, 2, 3, 4, 5];
        let mut j = JanelasMut::new(&mut v, 3);
        let mut vistas = Vec::new();
        while let Some(w) = j.proximo() {
            vistas.push(w.to_vec());
            w[0] *= 10;
        }
        assert_eq!(vistas, [vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
        assert_eq!(v, [10, 20, 30, 4, 5]);
    }

    #[test]
    fn janela_maior_que_os_dados() {
        let mut v = [1, 2];
        assert!(JanelasMut::new(&mut v, 3).proximo().is_none());
    }

    #[test]
    fn propagar_soma_acumulada() {
        let mut v = [1, 2, 3, 4];
        propagar(&mut v, 2);
        assert_eq!(v, [1, 3, 6, 10]);
        let mut w = [1, 0, 0, 0, 0];
        propagar(&mut w, 3);
        assert_eq!(w, [1, 0, 1, 0, 1]);
    }

    #[test]
    fn palavras_normalizadas() {
        let mut p = Palavras::new("O Rato, -- roeu!");
        assert_eq!(p.proximo(), Some("o"));
        assert_eq!(p.proximo(), Some("rato"));
        assert_eq!(p.proximo(), Some("roeu"));
        assert_eq!(p.proximo(), None);
    }

    #[test]
    fn conta_por_prefixo() {
        let texto = "O rato roeu a roupa do Rei de Roma.";
        assert_eq!(contar_com_prefixo(texto, "r"), 5);
        assert_eq!(contar_com_prefixo("-- ... !", "a"), 0);
    }
}
```

<details>
<summary>Dica</summary>

`self.dados.get_mut(self.pos..self.pos + self.n)` devolve `None` quando a janela passa do fim, e o `?` encerra. Em `Palavras`, um `loop` com `self.resto.next()?` pula as palavras que ficam vazias depois do filtro. Em `propagar`, copie `janela[0]` para uma variável antes de pegar `janela.last_mut()`, ou o *borrow checker* reclama de dois empréstimos.

Por que não uma função genérica `contar_se<E: Emprestador>(e, f)` com uma closure? A lição 06 mostra a limitação atual que aparece aí.

</details>

---

[← Última lição](08-hrtb-e-captura-em-impl-trait.md) · [Índice do módulo](README.md) · [Soluções →](../../solucoes/13-tipos-e-traits-avancados/README.md)
