# Exercícios — Módulo 10

Soluções em [`solucoes/10-smart-pointers`](../../solucoes/10-smart-pointers/).

Em todos os exercícios, crie o projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial em `src/lib.rs`. O objetivo final é sempre o mesmo: `cargo test` passando e `CARGO_BUILD_WARNINGS=deny cargo clippy --all-targets` sem avisos.

## 1. Árvore de busca com `Box` (fácil, faça compilar e passe nos testes)

Lição: [01](01-box-e-tipos-recursivos.md).

`cargo new --lib arvore`. O código não compila: corrija os tipos e depois implemente os quatro métodos. Valores menores vão para a esquerda, maiores para a direita, e repetidos não entram.

```rust
// ❌ NÃO COMPILA
#[derive(Debug, Default)]
pub struct Arvore {
    raiz: Option<No>,
}

#[derive(Debug)]
struct No {
    valor: i32,
    esq: Option<No>,
    dir: Option<No>,
}

impl Arvore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insere `valor`. Devolve `false` se ele já estava na árvore.
    pub fn inserir(&mut self, valor: i32) -> bool {
        todo!()
    }

    pub fn contem(&self, valor: i32) -> bool {
        todo!()
    }

    /// Os valores em ordem crescente.
    pub fn em_ordem(&self) -> Vec<i32> {
        todo!()
    }

    /// Número de nós no caminho mais longo da raiz até uma folha (0 se vazia).
    pub fn altura(&self) -> usize {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vazia() {
        let a = Arvore::new();
        assert!(!a.contem(1));
        assert!(a.em_ordem().is_empty());
        assert_eq!(a.altura(), 0);
    }

    #[test]
    fn insere_e_busca() {
        let mut a = Arvore::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            assert!(a.inserir(v));
        }
        assert!(!a.inserir(40), "repetido não entra");
        assert!(a.contem(60));
        assert!(!a.contem(65));
        assert_eq!(a.em_ordem(), [20, 30, 40, 50, 60, 70, 80]);
        assert_eq!(a.altura(), 3);
    }

    #[test]
    fn degenerada() {
        let mut a = Arvore::new();
        for v in 1..=5 {
            a.inserir(v);
        }
        assert_eq!(a.altura(), 5);
    }
}
```

**Esperado:** 3 testes passando.

<details>
<summary>Dica</summary>

O erro é o E0072 da lição 01, e o `help:` mostra a correção. Para `contem`, `em_ordem` e `altura`, `self.raiz.as_deref()` dá um `Option<&No>`, bom para percorrer com `while let` ou com uma função recursiva interna. Para `inserir`, ande com um cursor `let mut cursor = &mut self.raiz;` até achar um `None`, e escreva `*cursor = Some(Box::new(...))`.

</details>

## 2. Um ponteiro que conta os descartes (fácil, passe nos testes)

Lições: [02](02-deref-e-drop.md) e [04](04-cell-e-refcell.md).

`cargo new --lib rastreado`. Implemente `Deref`, `DerefMut` e `Drop` para `Rastreado`, de forma que ele se comporte como o valor de dentro e some 1 no contador a cada descarte.

```rust
use std::cell::Cell;

/// Envolve um valor e soma 1 em `drops` quando é descartado.
pub struct Rastreado<'a, T> {
    valor: T,
    drops: &'a Cell<u32>,
}

impl<'a, T> Rastreado<'a, T> {
    pub fn new(valor: T, drops: &'a Cell<u32>) -> Self {
        Rastreado { valor, drops }
    }
}

// Implemente aqui: Deref, DerefMut e Drop.

#[cfg(test)]
mod tests {
    use super::*;

    fn tamanho(s: &str) -> usize {
        s.len()
    }

    #[test]
    fn deref_e_coercao() {
        let drops = Cell::new(0);
        let r = Rastreado::new(String::from("oi"), &drops);
        assert_eq!(*r, "oi");
        assert_eq!(tamanho(&r), 2); // &Rastreado<String> -> &String -> &str
        assert!(r.starts_with('o'));
    }

    #[test]
    fn deref_mut() {
        let drops = Cell::new(0);
        let mut v = Rastreado::new(vec![1, 2], &drops);
        v.push(3);
        v[0] = 10;
        assert_eq!(*v, [10, 2, 3]);
    }

    #[test]
    fn conta_os_drops() {
        let drops = Cell::new(0);
        {
            let _a = Rastreado::new(1, &drops);
            let b = Rastreado::new(2, &drops);
            drop(b);
            assert_eq!(drops.get(), 1);
        }
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn mover_nao_descarta() {
        let drops = Cell::new(0);
        let lista: Vec<_> = (0..3).map(|i| Rastreado::new(i, &drops)).collect();
        let soma: i32 = lista.iter().map(|r| **r).sum();
        assert_eq!(soma, 3);
        assert_eq!(drops.get(), 0);
        drop(lista);
        assert_eq!(drops.get(), 3);
    }
}
```

**Esperado:** 4 testes passando. Depois, responda: por que o contador é um `&Cell<u32>`, e não um `&mut u32`? E no teste `mover_nao_descarta`, por que colocar os valores no `Vec` não conta como descarte?

<details>
<summary>Dica</summary>

`type Target = T;` e `fn deref(&self) -> &T { &self.valor }`. No `drop`, `self.drops.update(|n| n + 1)` (ou `set(get() + 1)`). Os `impl` não precisam de bound em `T`.

</details>

## 3. Conserte os pânicos (médio)

Lição: [04](04-cell-e-refcell.md).

`cargo new --lib banco`. Este código compila, mas dois testes dão *panic*:

```rust
// 💥 PANIC EM RUNTIME
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
pub struct Banco {
    contas: RefCell<HashMap<String, i64>>,
}

impl Banco {
    pub fn depositar(&self, nome: &str, valor: i64) {
        *self
            .contas
            .borrow_mut()
            .entry(nome.to_string())
            .or_insert(0) += valor;
    }

    pub fn saldo(&self, nome: &str) -> Option<i64> {
        self.contas.borrow().get(nome).copied()
    }

    /// Credita `taxa` por cento em todas as contas com saldo positivo.
    pub fn aplicar_juros(&self, taxa: i64) {
        for (nome, saldo) in self.contas.borrow().iter() {
            if *saldo > 0 {
                self.depositar(nome, saldo * taxa / 100);
            }
        }
    }

    /// Devolve o saldo, abrindo a conta com 0 se ela não existir.
    pub fn saldo_ou_abrir(&self, nome: &str) -> i64 {
        match self.contas.borrow().get(nome) {
            Some(s) => *s,
            None => {
                self.depositar(nome, 0);
                0
            }
        }
    }

    /// Abre a conta com `valor`. Devolve `false` se ela já existia.
    pub fn primeiro_deposito(&self, nome: &str, valor: i64) -> bool {
        if let Some(_saldo) = self.contas.borrow().get(nome) {
            false
        } else {
            self.depositar(nome, valor);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn juros() {
        let b = Banco::default();
        b.depositar("ana", 1000);
        b.depositar("bia", -50);
        b.aplicar_juros(10);
        assert_eq!(b.saldo("ana"), Some(1100));
        assert_eq!(b.saldo("bia"), Some(-50));
    }

    #[test]
    fn abre_conta() {
        let b = Banco::default();
        assert_eq!(b.saldo_ou_abrir("caio"), 0);
        assert_eq!(b.saldo("caio"), Some(0));
        b.depositar("caio", 5);
        assert_eq!(b.saldo_ou_abrir("caio"), 5);
    }

    #[test]
    fn primeiro() {
        let b = Banco::default();
        assert!(b.primeiro_deposito("ana", 10));
        assert!(!b.primeiro_deposito("ana", 99));
        assert_eq!(b.saldo("ana"), Some(10));
    }
}
```

```text
thread 'tests::abre_conta' (288933) panicked at src/lib.rs:13:14:
RefCell already borrowed
thread 'tests::juros' (288934) panicked at src/lib.rs:13:14:
RefCell already borrowed
test result: FAILED. 1 passed; 2 failed; ...
```

A linha 13 é o `borrow_mut` de `depositar`, mas o erro não está lá. Conserte `aplicar_juros` e `saldo_ou_abrir` sem mudar `depositar` nem os testes. Depois, explique por que `primeiro_deposito`, que tem a mesma forma de `saldo_ou_abrir`, não dá pânico.

<details>
<summary>Dica</summary>

Nos dois casos, um `Ref` ainda está vivo quando `depositar` pede o `RefMut`. Em `aplicar_juros`, calcule os juros num `Vec` primeiro (o `Ref` morre no fim dessa instrução) e deposite depois. Em `saldo_ou_abrir`, tire o valor do `RefCell` antes de decidir: `self.saldo(nome)` já devolve um `Option<i64>` copiado. A diferença entre `match` e `if let` está na lição 04.

</details>

## 4. Organograma com `Rc` e `Weak` (médio, passe nos testes)

Lição: [05](05-weak-e-ciclos.md).

`cargo new --lib organograma`. Cada funcionário é dono da sua equipe e enxerga o chefe sem mantê-lo vivo.

```rust
use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub struct Funcionario {
    pub nome: String,
    chefe: RefCell<Weak<Funcionario>>,
    equipe: RefCell<Vec<Rc<Funcionario>>>,
}

impl Funcionario {
    pub fn novo(nome: &str) -> Rc<Funcionario> {
        Rc::new(Funcionario {
            nome: nome.to_string(),
            chefe: RefCell::new(Weak::new()),
            equipe: RefCell::new(Vec::new()),
        })
    }

    /// Cria um funcionário na equipe de `chefe`.
    pub fn contratar(chefe: &Rc<Funcionario>, nome: &str) -> Rc<Funcionario> {
        todo!()
    }

    /// O chefe direto, se ainda existir.
    pub fn chefe(&self) -> Option<Rc<Funcionario>> {
        todo!()
    }

    /// Os nomes do próprio funcionário até o topo: `["dev", "gerente", "ceo"]`.
    pub fn cadeia_de_comando(&self) -> Vec<String> {
        todo!()
    }

    /// Quantas pessoas estão abaixo, direta ou indiretamente.
    pub fn tamanho_da_equipe(&self) -> usize {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadeia() {
        let ceo = Funcionario::novo("ceo");
        let gerente = Funcionario::contratar(&ceo, "gerente");
        let dev = Funcionario::contratar(&gerente, "dev");
        assert_eq!(dev.cadeia_de_comando(), ["dev", "gerente", "ceo"]);
        assert_eq!(ceo.cadeia_de_comando(), ["ceo"]);
    }

    #[test]
    fn equipe() {
        let ceo = Funcionario::novo("ceo");
        let g1 = Funcionario::contratar(&ceo, "g1");
        let g2 = Funcionario::contratar(&ceo, "g2");
        Funcionario::contratar(&g1, "a");
        Funcionario::contratar(&g1, "b");
        Funcionario::contratar(&g2, "c");
        assert_eq!(ceo.tamanho_da_equipe(), 5);
        assert_eq!(g1.tamanho_da_equipe(), 2);
    }

    #[test]
    fn contadores() {
        let ceo = Funcionario::novo("ceo");
        let g = Funcionario::contratar(&ceo, "g");
        Funcionario::contratar(&g, "a");
        // `g`: a variável e a equipe do ceo; observado pelo chefe de `a`.
        assert_eq!(Rc::strong_count(&g), 2);
        assert_eq!(Rc::weak_count(&g), 1);
        // O ceo só tem a variável como dona.
        assert_eq!(Rc::strong_count(&ceo), 1);
    }

    #[test]
    fn sem_vazamento() {
        let ceo = Funcionario::novo("ceo");
        let dev = Funcionario::contratar(&ceo, "dev");
        let observador = Rc::downgrade(&ceo);
        drop(ceo);
        assert!(observador.upgrade().is_none(), "o ceo foi liberado");
        assert!(dev.chefe().is_none());
        assert_eq!(dev.cadeia_de_comando(), ["dev"]);
    }
}
```

**Esperado:** 4 testes passando. O teste `sem_vazamento` falha se o chefe for guardado com `Rc` em vez de `Weak`. Experimente trocar e veja.

<details>
<summary>Dica</summary>

`contratar` segue o `criar_em` da lição 05. `chefe` é um `upgrade()`. Em `cadeia_de_comando`, um `while let Some(f) = atual` subindo com `atual = f.chefe();`. Em `tamanho_da_equipe`, some `1 + f.tamanho_da_equipe()` para cada membro.

</details>

## 5. Normalizar espaços sem alocar à toa (médio, passe nos testes)

Lição: [06](06-cow.md).

`cargo new --lib espacos`. Implemente `normalizar_espacos`: tira os espaços das pontas e troca cada sequência de espaços em branco (espaço, `\t`, `\n`...) por um único `' '`. Se não houver nada a trocar no meio, a função não pode alocar.

```rust
use std::borrow::Cow;

/// Tira os espaços das pontas e troca cada sequência de espaços em branco
/// internos por um único `' '`. Só aloca se o texto mudar.
pub fn normalizar_espacos(s: &str) -> Cow<'_, str> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ja_normal_nao_aloca() {
        let r = normalizar_espacos("bom dia");
        assert_eq!(r, "bom dia");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn so_pontas_nao_aloca() {
        let r = normalizar_espacos("  bom dia \n");
        assert_eq!(r, "bom dia");
        assert!(matches!(r, Cow::Borrowed(_)));
    }

    #[test]
    fn espacos_repetidos_alocam() {
        let r = normalizar_espacos("bom   dia,\t\tmundo");
        assert_eq!(r, "bom dia, mundo");
        assert!(matches!(r, Cow::Owned(_)));
    }

    #[test]
    fn tab_sozinho_tambem_e_trocado() {
        let r = normalizar_espacos("a\tb");
        assert_eq!(r, "a b");
        assert!(matches!(r, Cow::Owned(_)));
    }

    #[test]
    fn vazio() {
        assert_eq!(normalizar_espacos("   "), "");
        assert_eq!(normalizar_espacos(""), "");
    }

    #[test]
    fn devolve_string_quando_precisa() {
        let dono: String = normalizar_espacos(" x  y ").into_owned();
        assert_eq!(dono, "x y");
    }
}
```

**Esperado:** 6 testes passando. Repare no teste `so_pontas_nao_aloca`: tirar só as pontas ainda é um empréstimo. Por quê?

<details>
<summary>Dica</summary>

`s.trim()` devolve uma fatia de `s`. Escreva uma função auxiliar que diz se a fatia já está normalizada (todo espaço em branco é um `' '` sozinho). Se estiver, `Cow::Borrowed`; senão, junte `split_whitespace()` com `' '` numa `String` nova (`Cow::Owned`).

</details>

---

[← Lição 06](06-cow.md) · [Índice do módulo](README.md) · [Soluções →](../../solucoes/10-smart-pointers/README.md)
