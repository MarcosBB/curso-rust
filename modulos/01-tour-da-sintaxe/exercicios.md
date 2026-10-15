# Exercícios — Módulo 01

Soluções em [`solucoes/01-tour-da-sintaxe`](../../solucoes/01-tour-da-sintaxe/).

Do exercício 2 em diante, o formato é sempre o mesmo:

1. Crie uma biblioteca com `cargo new --lib nome` e acrescente `rust-version = "1.96"` no `Cargo.toml`.
2. Substitua o conteúdo de `src/lib.rs` pelo código do enunciado.
3. Troque cada `todo!()` pela implementação até `cargo test` passar.

O código inicial compila, mas os testes falham (com `not yet implemented`) até você implementar. Os avisos de parâmetro não usado (`unused_variables`) somem quando o `todo!()` sai. Não altere os testes.

## 1. Faça compilar: os cinco clássicos

Crie um projeto binário (`cargo new cinco-classicos`), cole o código abaixo no `src/main.rs` e faça compilar. Há **cinco** problemas, um de cada lição de 01 a 04. Corrija um de cada vez, lendo a mensagem do compilador, e rode `cargo check` depois de cada correção.

```rust
// ❌ NÃO COMPILA
fn dobro(x: i32) -> i32 {
    x * 2;
}

fn resumo(nome: &str, pontos: i32) -> String {
    format!("{nome} fez {pontos} pontos")
}

fn main() {
    let limite = "50".parse().unwrap();
    let total = 0;
    for parte in "3 10 7".split_whitespace() {
        let n: i32 = parte.parse().unwrap();
        total = total + dobro(n);
    }
    let vitorias = 2;
    if vitorias {
        println!("{}", resumo("Ana"));
    }
    println!("total: {total} (limite: {limite})");
}
```

**Saída esperada** (`cargo run`):

```text
Ana fez 40 pontos
total: 40 (limite: 50)
```

<details>
<summary>Dica</summary>

Na primeira compilação aparecem só três erros. Os outros dois surgem um de cada vez, depois que você corrige os anteriores, porque o compilador trabalha em etapas e alguns erros escondem outros (veja a [lição 05 do Módulo 00](../00-introducao/05-lendo-erros-do-compilador.md)). O `resumo` deve receber o `total`.

</details>

## 2. Collatz

A sequência de Collatz começa num número `n` e segue a regra: se `n` é par, o próximo é `n / 2`; se é ímpar, é `3 * n + 1`. Ela termina ao chegar em 1. Implemente a função que conta quantos termos a sequência tem, incluindo o `n` inicial e o 1 final. Para 11, a sequência é 11, 34, 17, 52, 26, 13, 40, 20, 10, 5, 16, 8, 4, 2, 1: são 15 termos.

```rust
/// Quantidade de termos da sequência de Collatz que começa em `n`,
/// contando o próprio `n` e o 1 final.
pub fn tamanho_collatz(n: u64) -> u32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comeca_em_um() {
        assert_eq!(tamanho_collatz(1), 1);
    }

    #[test]
    fn potencia_de_dois() {
        assert_eq!(tamanho_collatz(8), 4); // 8, 4, 2, 1
    }

    #[test]
    fn onze() {
        assert_eq!(tamanho_collatz(11), 15);
    }

    #[test]
    fn vinte_e_sete() {
        assert_eq!(tamanho_collatz(27), 112);
    }
}
```

<details>
<summary>Dica</summary>

O parâmetro `n` é imutável. Você pode declarar o parâmetro como `mut n: u64` (a assinatura continua a mesma para quem chama) ou fazer um *shadowing* com `let mut n = n;`. O próximo termo cabe numa linha com `if` como expressão. Depois, rode `cargo clippy`: ele tem uma sugestão para o `n % 2 == 0`.

</details>

## 3. FizzBuzz por expressão

Implemente o FizzBuzz **sem usar `return`** e sem `if`: use um `match` sobre a tupla `(n % 3, n % 5)`. Depois, implemente `sequencia`, que devolve os resultados de 1 até `ate`.

```rust
pub fn fizzbuzz(n: u32) -> String {
    todo!()
}

pub fn sequencia(ate: u32) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeros_comuns() {
        assert_eq!(fizzbuzz(1), "1");
        assert_eq!(fizzbuzz(7), "7");
    }

    #[test]
    fn multiplos() {
        assert_eq!(fizzbuzz(3), "Fizz");
        assert_eq!(fizzbuzz(10), "Buzz");
        assert_eq!(fizzbuzz(15), "FizzBuzz");
        assert_eq!(fizzbuzz(0), "FizzBuzz");
    }

    #[test]
    fn sequencia_ate_cinco() {
        assert_eq!(sequencia(5), vec!["1", "2", "Fizz", "4", "Buzz"]);
    }

    #[test]
    fn sequencia_vazia() {
        assert!(sequencia(0).is_empty());
    }
}
```

<details>
<summary>Dica</summary>

Num padrão de tupla, `_` casa com qualquer valor naquela posição: `(0, _)` quer dizer "divisível por 3, tanto faz o resto". A ordem dos braços importa. Para criar a `String`, use `String::from("Fizz")` ou `n.to_string()`.

</details>

## 4. Porcentagem sem surpresas

Implemente `percentual(parte, total)`, que devolve quanto `parte` representa de `total`, em porcentagem inteira arredondada para baixo. O resultado é um `u8`, e a função devolve `None` em vez de dar *panic* ou devolver lixo quando:

- `total` é zero;
- `parte * 100` não cabe num `u32`;
- o resultado não cabe num `u8` (acima de 255).

```rust
/// Quanto `parte` representa de `total`, em porcentagem inteira (arredondada para baixo).
///
/// Devolve `None` se `total` for zero, se `parte * 100` estourar `u32`
/// ou se o resultado não couber em `u8`.
pub fn percentual(parte: u32, total: u32) -> Option<u8> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metade() {
        assert_eq!(percentual(1, 2), Some(50));
    }

    #[test]
    fn arredonda_para_baixo() {
        assert_eq!(percentual(1, 3), Some(33));
    }

    #[test]
    fn total_zero() {
        assert_eq!(percentual(3, 0), None);
    }

    #[test]
    fn duzentos_por_cento_cabe() {
        assert_eq!(percentual(2, 1), Some(200));
    }

    #[test]
    fn trezentos_por_cento_nao_cabe() {
        assert_eq!(percentual(3, 1), None);
    }

    #[test]
    fn multiplicacao_estoura() {
        assert_eq!(percentual(u32::MAX, u32::MAX), None);
    }
}
```

<details>
<summary>Dica</summary>

`checked_mul` devolve um `Option`: trate com `match` (`Some(v) => ...`, `None => None`). Divisão inteira de `u32` já arredonda para baixo. Para o `u8`, `u8::try_from(x)` devolve um `Result`, e `.ok()` transforma esse `Result` num `Option` (`Ok(v)` vira `Some(v)`, e o erro vira `None`). Não use `as u8`: por quê?

</details>

## 5. Closures em ação

Implemente as quatro funções usando os métodos da [lição 08](08-kit-closures.md): `sort_by_key`, `iter().filter(...).count()`, `iter().map(...).collect()` e `retain`. Cada função cabe em uma linha.

As funções recebem a lista de pessoas como `&[(&str, u32)]` (só leitura) ou `&mut` (para poder alterar). Por enquanto, siga a receita: o `&` e o `&mut` são o assunto dos Módulos 02 e 03.

```rust
/// Ordena da pessoa mais nova para a mais velha.
pub fn ordenar_por_idade(pessoas: &mut [(&str, u32)]) {
    todo!()
}

/// Quantas pessoas têm 18 anos ou mais.
pub fn maiores_de_idade(pessoas: &[(&str, u32)]) -> usize {
    todo!()
}

/// Os nomes, em letras maiúsculas, na ordem em que aparecem.
pub fn nomes_em_maiusculas(pessoas: &[(&str, u32)]) -> Vec<String> {
    todo!()
}

/// Remove as pessoas com idade menor que `minima`.
pub fn manter_a_partir_de(pessoas: &mut Vec<(&str, u32)>, minima: u32) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordena() {
        let mut v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18), ("Duda", 7)];
        ordenar_por_idade(&mut v);
        assert_eq!(v, vec![("Duda", 7), ("Bia", 12), ("Caio", 18), ("Ana", 31)]);
    }

    #[test]
    fn conta_maiores() {
        let v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18), ("Duda", 7)];
        assert_eq!(maiores_de_idade(&v), 2);
        assert_eq!(maiores_de_idade(&[]), 0);
    }

    #[test]
    fn maiusculas() {
        let v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18)];
        assert_eq!(nomes_em_maiusculas(&v), vec!["ANA", "BIA", "CAIO"]);
    }

    #[test]
    fn filtra_no_lugar() {
        let mut v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18), ("Duda", 7)];
        manter_a_partir_de(&mut v, 12);
        assert_eq!(v, vec![("Ana", 31), ("Bia", 12), ("Caio", 18)]);
    }
}
```

<details>
<summary>Dica</summary>

Dentro das closures, `p.0` é o nome e `p.1` é a idade. Para as maiúsculas, `p.0.to_uppercase()` já devolve uma `String`.

</details>

## 6. Contador de palavras (integrador do kit)

Este exercício junta o kit inteiro: `Vec`, `HashMap`, `entry`, closures, módulos e testes.

Crie o pacote com `cargo new --lib contador-palavras` e monte três arquivos:

```text
contador-palavras/
└── src/
    ├── lib.rs      # contar e em_ordem
    ├── texto.rs    # módulo `texto`: normalizar e palavras
    └── main.rs     # o programa (crie este arquivo)
```

**`src/texto.rs`:**

```rust
/// Tira a pontuação das pontas e passa para minúsculas.
/// "(Rust)" vira "rust", e "--" vira "".
pub fn normalizar(palavra: &str) -> String {
    todo!()
}

/// Separa o texto em palavras normalizadas, descartando as que ficam vazias.
pub fn palavras(texto: &str) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza() {
        assert_eq!(normalizar("Olá,"), "olá");
        assert_eq!(normalizar("(Rust)"), "rust");
        assert_eq!(normalizar("d'água"), "d'água");
        assert_eq!(normalizar("--"), "");
    }

    #[test]
    fn separa_palavras() {
        assert_eq!(palavras("O rato, o RATO!"), vec!["o", "rato", "o", "rato"]);
    }

    #[test]
    fn descarta_vazias() {
        assert!(palavras("  -- ... ").is_empty());
    }
}
```

**`src/lib.rs`:**

```rust
pub mod texto;

use std::collections::HashMap;

/// Conta quantas vezes cada palavra aparece, sem diferenciar maiúsculas
/// e ignorando a pontuação nas pontas.
pub fn contar(texto: &str) -> HashMap<String, usize> {
    todo!()
}

/// Os pares (palavra, quantidade) em ordem alfabética.
pub fn em_ordem(contagem: &HashMap<String, usize>) -> Vec<(String, usize)> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conta_repetidas() {
        let c = contar("a b a");
        assert_eq!(c.len(), 2);
        assert_eq!(c.get("a"), Some(&2));
        assert_eq!(c.get("b"), Some(&1));
        assert_eq!(c.get("z"), None);
    }

    #[test]
    fn ignora_maiusculas_e_pontuacao() {
        let c = contar("Oi, oi! OI?");
        assert_eq!(c.len(), 1, "deveria virar uma palavra só: {c:?}");
        assert_eq!(c["oi"], 3);
    }

    #[test]
    fn texto_vazio() {
        assert!(contar("").is_empty());
        assert!(contar("  ... --- ").is_empty());
    }

    #[test]
    fn ordem_alfabetica() {
        let c = contar("b a b");
        assert_eq!(
            em_ordem(&c),
            vec![(String::from("a"), 1), (String::from("b"), 2)]
        );
    }
}
```

Por fim, escreva o `src/main.rs` usando a biblioteca para contar as palavras de `"O rato roeu a roupa do rei de Roma. O rei, irritado, roeu o resto!"` e imprimi-las em ordem, com a palavra alinhada à direita em 10 colunas.

**Esperado:** `cargo test` com os 7 testes passando, e `cargo run` imprimindo:

```text
         a 1
        de 1
        do 1
  irritado 1
         o 3
      rato 1
       rei 2
     resto 1
      roeu 2
      roma 1
     roupa 1
```

<details>
<summary>Dica: <code>texto.rs</code></summary>

`trim_matches` aceita uma closure que diz quais caracteres tirar das pontas: `palavra.trim_matches(|c: char| !c.is_alphanumeric())`. Aqui, a anotação `c: char` é necessária. Depois, `.to_lowercase()` devolve a `String`. Em `palavras`, use `split_whitespace()`, `map` e `collect`, e depois um `retain` para descartar as vazias. Você pode passar a função direto: `.map(normalizar)`.

</details>

<details>
<summary>Dica: <code>lib.rs</code> e <code>main.rs</code></summary>

`contar` é a receita da [lição 07](07-kit-vec-e-hashmap.md), percorrendo `texto::palavras(texto)`. Em `em_ordem`, o `for (palavra, quantidade) in contagem` entrega referências: guarde `(palavra.clone(), *quantidade)` num `Vec` e chame `sort()`. No `main.rs`, importe pelo nome da crate, com `_`: `use contador_palavras::{contar, em_ordem};`. Para o alinhamento, veja a [lição 06](06-macros-e-formatacao.md).

</details>

---

[Índice do módulo](README.md)
