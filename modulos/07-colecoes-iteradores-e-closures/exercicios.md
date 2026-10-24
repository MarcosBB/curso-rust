# Exercícios — Módulo 07

Soluções em [`solucoes/07-colecoes-iteradores-e-closures`](../../solucoes/07-colecoes-iteradores-e-closures/).

Os exercícios 1 a 5 são bibliotecas com testes. Para cada um, crie um projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml`, cole o código no `src/lib.rs` e rode `cargo test`. O objetivo é sempre o mesmo: **todos os testes passando**, sem avisos no `cargo clippy`. O exercício 6 parte do código do Projeto 1.

## 1. Faça compilar: o trio das closures (fácil)

Três funções, três erros diferentes. Corrija cada uma com a menor mudança possível e, num comentário acima de cada função, escreva qual das três traits (`Fn`, `FnMut` ou `FnOnce`) a closure implementa **depois** da correção, e por quê.

```rust
// (a) Devolve uma closure que multiplica por `k`.
pub fn multiplicador(k: i32) -> impl Fn(i32) -> i32 {
    |x| x * k
}

// (b) Conta as vogais (sem acento) de um texto.
pub fn conta_vogais(texto: &str) -> usize {
    let mut total = 0;
    let conta = |c: char| {
        if "aeiou".contains(c) {
            total += 1;
        }
    };
    for c in texto.chars() {
        conta(c);
    }
    total
}

// (c) Cumprimenta duas vezes.
fn chamar_duas_vezes<F: Fn() -> String>(f: F) -> String {
    f() + &f()
}

pub fn saudacao_dupla(nome: String) -> String {
    let saudar = move || {
        let n = nome;
        format!("oi, {n}! ")
    };
    chamar_duas_vezes(saudar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_multiplicador() {
        let triplo = multiplicador(3);
        assert_eq!(triplo(4), 12);
        assert_eq!(triplo(-1), -3);
    }

    #[test]
    fn b_vogais() {
        assert_eq!(conta_vogais("rust e legal"), 4);
        assert_eq!(conta_vogais("xyz"), 0);
    }

    #[test]
    fn c_saudacao() {
        assert_eq!(saudacao_dupla(String::from("Ana")), "oi, Ana! oi, Ana! ");
    }
}
```

**Esperado:** antes da correção, os erros E0373, E0596 e E0525; depois, 3 testes passando. Não mude a assinatura de `chamar_duas_vezes`.

<details>
<summary>Dica</summary>

Os três erros estão na lição 04 e na lição 05, com os `help:` correspondentes. No (c), pergunte-se: a closure precisa **entregar** `nome` a alguém, ou só ler?

</details>

## 2. Traduzindo comprehensions (fácil)

Reescreva em Rust estas funções Python, usando iteradores e coleções da std. Os testes definem os tipos exatos.

```python
from collections import Counter

def quadrados_pares(xs):  return [x * x for x in xs if x % 2 == 0]
def tamanhos(palavras):   return {p: len(p) for p in palavras}
def letras_unicas(texto): return {c.lower() for c in texto if c.isalpha()}
def pares(n):             return [(i, j) for i in range(n) for j in range(i + 1, n)]
def top3(texto):          return Counter(texto.lower().split()).most_common(3)
```

```rust
use std::collections::{BTreeSet, HashMap};

/// `[x * x for x in xs if x % 2 == 0]`
pub fn quadrados_pares(xs: &[i32]) -> Vec<i32> {
    todo!()
}

/// `{p: len(p) for p in palavras}` (tamanho em caracteres, não em bytes)
pub fn tamanhos(palavras: &[&str]) -> HashMap<String, usize> {
    todo!()
}

/// `{c.lower() for c in texto if c.isalpha()}`
pub fn letras_unicas(texto: &str) -> BTreeSet<char> {
    todo!()
}

/// `[(i, j) for i in range(n) for j in range(i + 1, n)]`
pub fn pares(n: u32) -> Vec<(u32, u32)> {
    todo!()
}

/// `Counter(texto.lower().split()).most_common(3)`: as 3 palavras mais
/// frequentes, da mais para a menos frequente. Empate: ordem alfabética.
pub fn top3(texto: &str) -> Vec<(String, usize)> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_quadrados_pares() {
        assert_eq!(quadrados_pares(&[1, 2, 3, 4, -6]), [4, 16, 36]);
        assert!(quadrados_pares(&[1, 3]).is_empty());
    }

    #[test]
    fn t_tamanhos() {
        let t = tamanhos(&["sol", "maçã", "sol"]);
        assert_eq!(t.len(), 2);
        assert_eq!(t["maçã"], 4);
        assert_eq!(t.get("sol"), Some(&3));
    }

    #[test]
    fn t_letras_unicas() {
        let l: String = letras_unicas("Ana, ÁGUA 42!").into_iter().collect();
        assert_eq!(l, "agnuá");
    }

    #[test]
    fn t_pares() {
        assert_eq!(pares(3), [(0, 1), (0, 2), (1, 2)]);
        assert_eq!(pares(5).len(), 10);
        assert!(pares(1).is_empty());
    }

    #[test]
    fn t_top3() {
        let texto = "o rato roeu a roupa do rei de Roma o Rato fugiu a";
        assert_eq!(
            top3(texto),
            [
                ("a".to_string(), 2),
                ("o".to_string(), 2),
                ("rato".to_string(), 2)
            ]
        );
        assert_eq!(top3("um"), [("um".to_string(), 1)]);
    }
}
```

**Esperado:** 5 testes passando.

<details>
<summary>Dica</summary>

- `pares`: dois `for` viram `flat_map` com um `map` dentro. A closure interna precisa de `move` (lição 07).
- `letras_unicas`: `char::to_lowercase` devolve um **iterador** (algumas letras viram mais de um caractere em minúsculas), então combina com `flat_map`.
- `top3`: conte com a *Entry API* (lição 02), passe os pares para um `Vec`, ordene com `sort_by` + `then_with` (lição 01) e corte com `truncate`. Sem o desempate, o teste falha de vez em quando: a ordem do `HashMap` muda a cada execução.

</details>

## 3. Índice invertido (médio)

Monte um índice de palavras: para cada palavra (em minúsculas, sem a pontuação das pontas), o conjunto das linhas em que ela aparece. Depois, use o índice para achar as linhas que contêm **todas** as palavras de uma busca.

```rust
use std::collections::{BTreeMap, BTreeSet};

/// Palavra (em minúsculas) → números das linhas (a partir de 1) em que aparece.
/// A pontuação nas pontas da palavra é descartada ("Rei!" vira "rei").
pub fn indice(texto: &str) -> BTreeMap<String, BTreeSet<usize>> {
    todo!()
}

/// Linhas que contêm **todas** as palavras pedidas (sem diferenciar
/// maiúsculas). Sem palavras, ou com uma palavra fora do índice: vazio.
pub fn linhas_com_todas(
    idx: &BTreeMap<String, BTreeSet<usize>>,
    palavras: &[&str],
) -> BTreeSet<usize> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXTO: &str = "\
O rato roeu a roupa.
A roupa era do Rei!
O rei, de Roma, riu do rato.";

    fn linhas(v: &[usize]) -> BTreeSet<usize> {
        v.iter().copied().collect()
    }

    #[test]
    fn monta_o_indice() {
        let idx = indice(TEXTO);
        assert_eq!(idx["rato"], linhas(&[1, 3]));
        assert_eq!(idx["rei"], linhas(&[2, 3]));
        assert_eq!(idx["roupa"], linhas(&[1, 2]));
        assert!(!idx.contains_key("Rei"));
        assert!(!idx.contains_key("rei!"));
    }

    #[test]
    fn chaves_em_ordem() {
        let idx = indice("b a\nc a");
        let chaves: Vec<&str> = idx.keys().map(|k| k.as_str()).collect();
        assert_eq!(chaves, ["a", "b", "c"]);
        assert_eq!(idx["a"], linhas(&[1, 2]));
    }

    #[test]
    fn ignora_pontuacao_solta() {
        let idx = indice("oi - tchau");
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn busca_com_todas() {
        let idx = indice(TEXTO);
        assert_eq!(linhas_com_todas(&idx, &["rato", "Rei"]), linhas(&[3]));
        assert_eq!(linhas_com_todas(&idx, &["roupa"]), linhas(&[1, 2]));
        assert!(linhas_com_todas(&idx, &["rato", "dragão"]).is_empty());
        assert!(linhas_com_todas(&idx, &[]).is_empty());
    }
}
```

**Esperado:** 4 testes passando.

<details>
<summary>Dica</summary>

`lines().enumerate()` dá o número da linha. Para limpar a palavra: `trim_matches(|c: char| !c.is_alphanumeric())` e depois `to_lowercase()`. Uma palavra que vira `""` (um travessão solto) deve ser pulada. Para inserir: `idx.entry(palavra).or_default().insert(linha)`. Na busca, comece com uma cópia do conjunto da primeira palavra e vá mantendo (`retain`) só as linhas que estão nos conjuntos das outras.

</details>

## 4. Implemente dois iteradores (médio)

Implemente `Iterator` para `Collatz` (a sequência que, a partir de `n`, divide por 2 os pares e faz `3n + 1` com os ímpares, até chegar a 1) e para `Pares`, que devolve pares de elementos consecutivos de um slice, **sem copiar** os elementos. Você escolhe os campos das structs.

```rust
pub struct Collatz {
    // seus campos
}

/// Sequência de Collatz a partir de `n`, terminando em 1 (inclusive).
/// Para `n == 0`, a sequência é vazia.
pub fn collatz(n: u64) -> Collatz {
    todo!()
}

// impl Iterator for Collatz { ... }

pub struct Pares<'a, T> {
    // seus campos
}

/// Pares de elementos consecutivos: (v[0], v[1]), (v[1], v[2]), ...
pub fn pares<T>(v: &[T]) -> Pares<'_, T> {
    todo!()
}

// impl<'a, T> Iterator for Pares<'a, T> { ... }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collatz_de_6() {
        assert_eq!(collatz(6).collect::<Vec<_>>(), [6, 3, 10, 5, 16, 8, 4, 2, 1]);
    }

    #[test]
    fn collatz_casos_de_borda() {
        assert_eq!(collatz(1).collect::<Vec<_>>(), [1]);
        assert_eq!(collatz(0).count(), 0);
    }

    #[test]
    fn collatz_de_27() {
        assert_eq!(collatz(27).count(), 112);
        assert_eq!(collatz(27).max(), Some(9232));
    }

    #[test]
    fn pares_consecutivos() {
        let v = [1, 4, 9, 16];
        let p: Vec<(&i32, &i32)> = pares(&v).collect();
        assert_eq!(p, [(&1, &4), (&4, &9), (&9, &16)]);
        let difs: Vec<i32> = pares(&v).map(|(a, b)| b - a).collect();
        assert_eq!(difs, [3, 5, 7]);
    }

    #[test]
    fn pares_curtos() {
        assert_eq!(pares(&[42]).count(), 0);
        assert_eq!(pares::<i32>(&[]).count(), 0);
    }

    #[test]
    fn pares_com_strings_nao_copia() {
        let nomes = vec![String::from("ana"), String::from("bia"), String::from("caio")];
        let maior = pares(&nomes).max_by_key(|(a, b)| a.len() + b.len());
        assert_eq!(maior, Some((&nomes[1], &nomes[2])));
        assert!(std::ptr::eq(maior.unwrap().0, &nomes[1]));
    }
}
```

**Esperado:** 6 testes passando. **Bônus:** escreva `collatz_successors(n: u64) -> impl Iterator<Item = u64>` com `iter::successors` e um teste que compare as duas versões para `n` de 0 a 49 (o método `eq` de iteradores compara item a item).

<details>
<summary>Dica</summary>

- `Collatz`: guarde o **próximo** valor como `Option<u64>`. No `next`, `let atual = self.proximo?;` encerra a sequência quando for `None`, e o 1 é o caso em que o próximo vira `None`.
- `Pares`: guarde um `&'a [T]` com o que falta percorrer, e use `type Item = (&'a T, &'a T)`. Um padrão de slice (`let [a, b, ..] = self.resto else { return None; };`) pega os dois primeiros; depois, avance com `self.resto = &self.resto[1..]`. É a lição 08, seção "Um iterador que empresta".

</details>

## 5. Closures guardadas (médio)

Implemente duas structs que guardam closures:

- `Pipeline`: uma lista de transformações `i32 -> i32`, aplicadas em ordem. Cada etapa pode ser uma closure diferente.
- `Filtro<P>`: um filtro de linhas com um único predicado, escolhido na criação (adaptado do exercício *Log Filter* do Comprehensive Rust).

```rust
#[derive(Default)]
pub struct Pipeline {
    // seus campos
}

impl Pipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta uma etapa no fim e devolve o pipeline (para encadear).
    pub fn etapa(mut self, f: impl Fn(i32) -> i32 + 'static) -> Self {
        todo!()
    }

    pub fn aplicar(&self, x: i32) -> i32 {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

pub struct Filtro<P> {
    pred: P,
}

impl<P: Fn(&str) -> bool> Filtro<P> {
    pub fn new(pred: P) -> Self {
        Filtro { pred }
    }

    pub fn filtrar<'a>(&self, linhas: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_vazio_e_identidade() {
        let p = Pipeline::new();
        assert!(p.is_empty());
        assert_eq!(p.aplicar(7), 7);
    }

    #[test]
    fn pipeline_aplica_em_ordem() {
        let p = Pipeline::new().etapa(|x| x + 1).etapa(|x| x * 10);
        assert_eq!(p.aplicar(2), 30); // (2 + 1) * 10, e não 2 * 10 + 1
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn pipeline_com_captura() {
        let deslocamento = 100;
        let limite = 150;
        let p = Pipeline::new()
            .etapa(move |x| x + deslocamento)
            .etapa(move |x| x.min(limite));
        assert_eq!(p.aplicar(20), 120);
        assert_eq!(p.aplicar(90), 150);
    }

    #[test]
    fn filtro_de_log() {
        let nivel = String::from("ERRO");
        let f = Filtro::new(|l: &str| l.starts_with(&nivel));
        let log = "ERRO disco cheio\nINFO ok\nERRO rede caiu";
        assert_eq!(f.filtrar(log.lines()), ["ERRO disco cheio", "ERRO rede caiu"]);
        // `nivel` foi só emprestado: continua aqui.
        assert_eq!(nivel, "ERRO");
    }

    #[test]
    fn filtro_aceita_vec_e_array() {
        let curtas = Filtro::new(|l: &str| l.len() <= 3);
        assert_eq!(curtas.filtrar(vec!["ab", "abcd"]), ["ab"]);
        assert_eq!(curtas.filtrar(["xyz", "", "long"]), ["xyz", ""]);
    }
}
```

**Esperado:** 5 testes passando. Depois, responda: por que `Pipeline` guarda `Box<dyn Fn>` e `Filtro` usa um genérico `P`? O que aconteceria se `Pipeline` fosse `Pipeline<F>` com um `Vec<F>`? E por que o bound de `Filtro` é `Fn`, e não `FnMut`?

<details>
<summary>Dica</summary>

Para o campo de `Pipeline`, veja "Devolver uma closure" na lição 05 (um `Vec` com closures diferentes). `aplicar` é um `fold` sobre as etapas, começando em `x`. Em `filtrar`, lembre que a closure guardada num campo se chama com parênteses: `(self.pred)(l)`.

</details>

## 6. Minigrep, retomada (médio)

O [Projeto 1](../../projetos/01-minigrep/README.md) prometeu: "No Módulo 07 você vai reescrever a função consumindo um iterador, movendo as `String` em vez de copiá-las". Chegou a hora. Parta da sua versão do minigrep, ou copie a [solução do Projeto 1](../../solucoes/projeto-01-minigrep/) (se você já fez o exercício do minigrep do Módulo 06, parta dela: as mudanças não se cruzam).

**(a) `search` com adaptadores.** Reescreva `search` (em `src/search.rs`) como uma cadeia `lines().enumerate().filter(...).map(...).collect()`, sem `Vec` mutável nem `push`. A assinatura não muda, e todos os testes das etapas 1 e 2 precisam continuar passando, inclusive `results_borrow_from_contents`. Aproveite e reescreva `render` (em `src/lib.rs`) da mesma forma.

**(b) `from_args` consumindo um iterador.** Troque a assinatura de `Config::from_args` por:

```rust
pub fn from_args(
    args: impl IntoIterator<Item = String>,
    ignore_case_env: bool,
) -> Result<Config, ArgsError>
```

e reescreva o corpo para **mover** as `String` para dentro do `Config`, sem nenhum `to_string`, `clone` ou `to_owned` de argumento. Depois:

- no `main`, passe `env::args().skip(1)` direto, sem o `Vec` intermediário;
- nos testes da etapa 3, troque `from_args(&args(...), ...)` por `from_args(args(...), ...)`, e reescreva o helper `args` com `map` e `collect`;
- acrescente estes testes no fim de `src/config.rs`:

```rust
#[cfg(test)]
mod m07_tests {
    use super::*;

    #[test]
    fn accepts_any_into_iterator() {
        // um array de String
        let cfg = Config::from_args(["foo", "a.txt"].map(String::from), false).unwrap();
        assert_eq!(cfg.inputs, vec![Input::File("a.txt".into())]);

        // um iterador, como o de `env::args().skip(1)`
        let all = vec!["minigrep".to_string(), "-n".to_string(), "bar".to_string()];
        let cfg = Config::from_args(all.into_iter().skip(1), false).unwrap();
        assert_eq!(cfg.pattern, "bar");
        assert!(cfg.output.line_numbers);
    }

    #[test]
    fn pattern_is_moved_not_copied() {
        let list = vec!["um padrão qualquer".to_string()];
        let ptr = list[0].as_ptr();
        let cfg = Config::from_args(list, false).unwrap();
        // Mesmo buffer: a `String` foi movida para o `Config`.
        assert!(std::ptr::eq(cfg.pattern.as_ptr(), ptr));
    }
}
```

**(c) Para ir além.** Escreva `search_iter`, uma versão preguiçosa de `search` que devolve `impl Iterator<Item = Match<'a>>` em vez de um `Vec`, e um teste que confira que as duas versões dão o mesmo resultado. Por que `search_iter` precisa que `pattern` também viva tanto quanto o iterador, e `search` não?

**Esperado:** todos os testes unitários e de integração passando (`cargo test`), sem avisos no `cargo clippy`, e o programa se comportando como antes (`cargo run -- -n -i how poem.txt`).

<details>
<summary>Dica</summary>

- O laço de `from_args` pode continuar um `for arg in args`, agora com `arg: String`. Guarde os posicionais num `Vec<String>`. No braço da opção desconhecida, `UnknownFlag(arg)` funciona mesmo com o `match arg.as_str()`: o empréstimo do `as_str()` já terminou quando o braço executa.
- Para separar o padrão dos arquivos sem copiar: `let mut positional = positional.into_iter();`, depois `positional.next()` dá o padrão (por valor), e o resto vira `Vec<Input>` com `map` + `collect`. `PathBuf::from(String)` também não copia.
- Em `search_iter`, as closures precisam de `move` (lições 04 e 08).

</details>
