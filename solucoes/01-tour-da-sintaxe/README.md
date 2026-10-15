# Soluções — Módulo 01

Enunciados em [`modulos/01-tour-da-sintaxe/exercicios.md`](../../modulos/01-tour-da-sintaxe/exercicios.md).

Cada solução é um projeto Cargo. Para conferir:

```bash
cd collatz
cargo test
```

## 1. Faça compilar: os cinco clássicos

Projeto: [`cinco-classicos/`](cinco-classicos/src/main.rs). Os erros, na ordem em que o compilador mostra:

| # | Erro | Problema | Correção |
|---|---|---|---|
| 1 | E0308 `expected i32, found ()` | `x * 2;` em `dobro`: o `;` descarta o valor, e a função devolve `()` | tirar o `;` |
| 2 | E0308 `expected bool, found integer` | `if vitorias`: não existe *truthiness* | `if vitorias > 0` |
| 3 | E0061 `takes 2 arguments but 1 argument was supplied` | `resumo("Ana")`: não há argumento padrão | `resumo("Ana", total)` |
| 4 | E0284 `type annotations needed` | `"50".parse()`: nada diz o tipo de destino | `let limite: i32 = ...` |
| 5 | E0384 `cannot assign twice to immutable variable` | `total` muda dentro do laço | `let mut total = 0;` |

Os erros 4 e 5 só aparecem depois que os anteriores são corrigidos, um de cada vez. O compilador trabalha em etapas, e um erro numa etapa pode esconder outros da etapa seguinte. Por isso vale corrigir e recompilar com `cargo check`, em vez de tentar adivinhar tudo de uma vez.

No erro 5, `total = total + dobro(n)` também pode virar `total += dobro(n)`, que é o mais comum.

## 2. Collatz

Projeto: [`collatz/`](collatz/src/lib.rs).

```rust
pub fn tamanho_collatz(mut n: u64) -> u32 {
    let mut termos = 1;
    while n != 1 {
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            3 * n + 1
        };
        termos += 1;
    }
    termos
}
```

- `mut n` no parâmetro torna a variável local mutável. Para quem chama, nada muda: o argumento é copiado, e o valor original não é alterado.
- O `if` é uma expressão, e o resultado vai direto para `n`.
- `termos` começa em 1 porque conta o próprio `n`. O tipo `u32` vem do retorno da função, sem precisar anotar.
- O Clippy sugere `n.is_multiple_of(2)` no lugar de `n % 2 == 0` (lint `manual_is_multiple_of`). As duas formas funcionam, e a sugestão vale para inteiros sem sinal.

Uma alternativa é `loop` com `break termos` quando `n == 1`.

## 3. FizzBuzz por expressão

Projeto: [`fizzbuzz/`](fizzbuzz/src/lib.rs).

```rust
pub fn fizzbuzz(n: u32) -> String {
    match (n % 3, n % 5) {
        (0, 0) => String::from("FizzBuzz"),
        (0, _) => String::from("Fizz"),
        (_, 0) => String::from("Buzz"),
        _ => n.to_string(),
    }
}
```

- O `match` é a última expressão da função, então é o valor de retorno. Não precisa de `return` nem de variável auxiliar.
- A ordem dos braços importa: `(0, 0)` precisa vir antes de `(0, _)`, que também casaria com 15.
- O `_` final cobre todo o resto, e o `match` fica exaustivo.

`sequencia` é um `for n in 1..=ate` com `push`. Com `ate = 0`, o *range* `1..=0` é vazio e o laço não roda. Depois do Módulo 07, você escreveria `(1..=ate).map(fizzbuzz).collect()`.

## 4. Porcentagem sem surpresas

Projeto: [`percentual/`](percentual/src/lib.rs).

```rust
pub fn percentual(parte: u32, total: u32) -> Option<u8> {
    if total == 0 {
        return None;
    }
    match parte.checked_mul(100) {
        Some(vezes_cem) => u8::try_from(vezes_cem / total).ok(),
        None => None,
    }
}
```

- `return None` para o caso especial, logo no começo. A divisão por zero daria *panic* (em debug **e** em release).
- `checked_mul` devolve `None` se `parte * 100` estourar. Com `parte * 100` simples, o teste `multiplicacao_estoura` daria *panic* no perfil `dev`.
- `u8::try_from(...).ok()` converte o `Result` em `Option`.
- **Por que não `as u8`?** Porque `300 as u8` vale 44, sem erro nenhum. O teste `trezentos_por_cento_nao_cabe` passaria a receber `Some(44)`.

Outra forma de eliminar o `if` é `checked_div`, que devolve `None` quando o divisor é zero. No Módulo 05, você vai ver o operador `?`, que encurta tudo para `u8::try_from(parte.checked_mul(100)?.checked_div(total)?).ok()`.

## 5. Closures em ação

Projeto: [`pessoas/`](pessoas/src/lib.rs).

```rust
pub fn ordenar_por_idade(pessoas: &mut [(&str, u32)]) {
    pessoas.sort_by_key(|p| p.1);
}

pub fn maiores_de_idade(pessoas: &[(&str, u32)]) -> usize {
    pessoas.iter().filter(|p| p.1 >= 18).count()
}

pub fn nomes_em_maiusculas(pessoas: &[(&str, u32)]) -> Vec<String> {
    pessoas.iter().map(|p| p.0.to_uppercase()).collect()
}

pub fn manter_a_partir_de(pessoas: &mut Vec<(&str, u32)>, minima: u32) {
    pessoas.retain(|p| p.1 >= minima);
}
```

- `sort_by_key` é estável: pessoas com a mesma idade mantêm a ordem original.
- No `filter`, o `p` é uma referência para uma referência para a tupla, mas o acesso a campo (`p.1`) atravessa as referências sozinho.
- A closure do `retain` usa `minima`, uma variável de fora, sem recebê-la como parâmetro: é a captura do ambiente.
- `ordenar_por_idade` recebe `&mut [...]` (um slice mutável), e `manter_a_partir_de` recebe `&mut Vec<...>`. A diferença: ordenar só troca elementos de lugar, enquanto o `retain` muda o **tamanho**, e um slice não pode mudar de tamanho. O Clippy chega a sugerir trocar `&mut Vec` por `&mut [...]` quando você só usa operações de slice (`ptr_arg`).

## 6. Contador de palavras

Projeto: [`contador-palavras/`](contador-palavras/): [`src/texto.rs`](contador-palavras/src/texto.rs), [`src/lib.rs`](contador-palavras/src/lib.rs) e [`src/main.rs`](contador-palavras/src/main.rs).

**`texto.rs`**

```rust
pub fn normalizar(palavra: &str) -> String {
    palavra
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

pub fn palavras(texto: &str) -> Vec<String> {
    let mut resultado: Vec<String> = texto.split_whitespace().map(normalizar).collect();
    resultado.retain(|p| !p.is_empty());
    resultado
}
```

- `trim_matches` tira das **pontas** os caracteres para os quais a closure devolve `true`. Por isso o apóstrofo de `d'água` fica: ele está no meio.
- `is_alphanumeric` e `to_lowercase` entendem Unicode: `"Olá,"` vira `"olá"`.
- `.map(normalizar)` passa a função direto. Com `.map(|p| normalizar(p))`, o Clippy avisaria `redundant_closure`.
- O `retain` remove as "palavras" que eram só pontuação (`"--"` vira `""`).

**`lib.rs`**

```rust
pub fn contar(texto: &str) -> HashMap<String, usize> {
    let mut contagem = HashMap::new();
    for palavra in texto::palavras(texto) {
        *contagem.entry(palavra).or_insert(0) += 1;
    }
    contagem
}

pub fn em_ordem(contagem: &HashMap<String, usize>) -> Vec<(String, usize)> {
    let mut pares = Vec::new();
    for (palavra, quantidade) in contagem {
        pares.push((palavra.clone(), *quantidade));
    }
    pares.sort();
    pares
}
```

- O `HashMap::new()` não precisa de anotação: o tipo vem do retorno da função.
- Dentro de `contar`, `texto` é o parâmetro e `texto::palavras` é o módulo. Não há conflito, porque o compilador sabe pelo `::` que se trata de um caminho.
- Em `em_ordem`, o mapa chega por referência, então o `for` entrega referências: `palavra.clone()` cria uma `String` nova para o `Vec`, e `*quantidade` copia o número. O porquê do `clone` é assunto do Módulo 02.
- `sort()` em tuplas compara o primeiro elemento e, em caso de empate, o segundo. Aqui as palavras nunca empatam.

**`main.rs`**

```rust
use contador_palavras::{contar, em_ordem};

fn main() {
    let texto = "O rato roeu a roupa do rei de Roma. O rei, irritado, roeu o resto!";
    let contagem = contar(texto);
    for (palavra, quantidade) in em_ordem(&contagem) {
        println!("{palavra:>10} {quantidade}");
    }
}
```

O `main.rs` só usa a API pública da biblioteca, pelo nome da crate com `_`. A função `normalizar` poderia até ser privada, porque o `main` não a usa. Ela é `pub` aqui para que outras crates possam reaproveitá-la.
