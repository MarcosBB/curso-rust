# 06 — Strings UTF-8

**Objetivos.** Ao final, você vai saber:

- por que `len()` conta bytes e não caracteres;
- as três formas de olhar para um texto: bytes, `char`s e grafemas;
- por que `s[0]` não compila, e o que usar no lugar;
- fatiar uma string sem dar *panic*.

## `len()` conta bytes

```rust
let s = "olá";
println!("{}", s.len());             // 4
println!("{}", s.chars().count());   // 3
```

Toda `String` e toda `&str` guardam texto em **UTF-8**, uma codificação de tamanho variável: cada caractere ocupa de 1 a 4 bytes. Letras sem acento ocupam 1 byte, `á` e `ç` ocupam 2, a maioria dos emojis ocupa 4. `len()` devolve o tamanho em bytes, porque é isso que a `String` sabe sem percorrer o texto (é o campo `len` que você viu na [lição 01](01-quem-libera-a-memoria.md)).

## Três formas de olhar para um texto

O Rust Book descreve três níveis:

1. **Bytes** (`u8`): o que está de fato na memória. `s.bytes()` ou `s.as_bytes()`.
2. **Valores escalares Unicode** (`char`): cada `char` do Rust tem 4 bytes e guarda um *code point* Unicode. `s.chars()`.
3. **Grafemas** (*grapheme clusters*): o que uma pessoa enxerga como "uma letra". A biblioteca padrão não tem um iterador para isso.

O projeto [`codigo/texto`](codigo/texto/) imprime as três visões para quatro textos:

```text
"olá": 4 bytes, 3 chars, bytes = [111, 108, 195, 161], chars = ['o', 'l', 'á']
"é": 2 bytes, 1 chars, bytes = [195, 169], chars = ['é']
"e\u{301}": 3 bytes, 2 chars, bytes = [101, 204, 129], chars = ['e', '\u{301}']
"👍🏽": 8 bytes, 2 chars, bytes = [240, 159, 145, 141, 240, 159, 143, 189], chars = ['👍', '🏽']
```

| Texto | Bytes | `char`s | Grafemas (o que você vê) |
|---|---|---|---|
| `"olá"` | 4 | 3 | 3 |
| `"é"` (pré-composto) | 2 | 1 | 1 |
| `"e\u{301}"` (`e` + acento combinante) | 3 | 2 | 1 (`é`) |
| `"👍🏽"` (polegar + tom de pele) | 8 | 2 | 1 |

As duas formas de `é` aparecem iguais na tela, mas são sequências diferentes: `"é" == "e\u{301}"` é `false`. Comparar textos "como uma pessoa compararia" exige normalização Unicode, que fica fora da biblioteca padrão.

> [!NOTE]
> Para trabalhar com grafemas, use a crate [`unicode-segmentation`](https://docs.rs/unicode-segmentation). Na maioria dos programas, `chars()` basta. Grafemas importam quando você mostra, corta ou inverte texto para uma pessoa ler.

## Por que `s[0]` não compila

```rust
// ❌ NÃO COMPILA
fn main() {
    let s = String::from("olá");
    let c = s[0];
    println!("{c}");
}
```

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
   --> src/main.rs:3:15
    |
  3 |     let c = s[0];
    |               ^ string indices are ranges of `usize`
    |
    = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
    = note: you can use `.chars().nth()` or `.bytes().nth()`
            for more information, see chapter 8 in The Book: <https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings>
...
    = note: required for `String` to implement `Index<{integer}>`
```

O compilador já diz o que fazer, e o Rust Book explica o porquê, com dois motivos:

1. **Um índice de byte pode cair no meio de um caractere.** Em `"olá"`, o byte 2 é `195`, a primeira metade do `á`. Devolver `195` como se fosse "o terceiro caractere" seria errado, e devolver `á` exigiria decidir sozinho que você queria caracteres.
2. **Indexar deveria ser O(1).** Achar o n-ésimo `char` exige percorrer o texto desde o começo, porque cada caractere tem um tamanho. O Rust prefere não esconder esse custo atrás de `[]`.

O que usar no lugar depende do que você quer:

```rust
let s = "olá";
println!("{:?}", s.chars().nth(2));   // Some('á'): o 3º char, percorrendo (O(n))
println!("{}", s.as_bytes()[2]);      // 195: o 3º byte (O(1))
let pares: Vec<(usize, char)> = s.char_indices().collect();
println!("{pares:?}");                // [(0, 'o'), (1, 'l'), (2, 'á')]
```

`chars().nth()` devolve um `Option` porque o texto pode ser mais curto que o índice pedido. `char_indices()` dá cada `char` junto com o índice em **bytes** onde ele começa, e é a ponte entre as duas visões: esses índices são sempre seguros para fatiar.

> [!TIP]
> A nota do compilador sugere `.bytes().nth()`, mas o clippy prefere `s.as_bytes()[i]` (ou `.get(i)`), que é direto, sem percorrer um iterador.

## Fatiar com intervalo: pode dar *panic*

Fatiar com um intervalo de bytes compila. O problema aparece em tempo de execução:

```rust
let s = "olá";
println!("{}", &s[0..2]);   // ol
```

```rust
// 💥 PANIC EM RUNTIME
fn main() {
    let s = "olá";
    let fatia = &s[0..3];
    println!("{fatia}");
}
```

```text
thread 'main' (...) panicked at src/main.rs:3:19:
end byte index 3 is not a char boundary; it is inside 'á' (bytes 2..4 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

O byte 3 está no meio do `á` (que ocupa os bytes 2 e 3). Uma `&str` precisa conter UTF-8 válido, então o Rust prefere parar o programa a criar uma fatia com meio caractere. Repare que o compilador não avisa nada, mesmo com índices constantes.

Quando o índice vem de fora (do usuário, de uma conta), use as versões que não dão *panic*:

```rust
let s = "olá";
println!("{:?}", s.get(0..3));            // None
println!("{:?}", s.get(0..4));            // Some("olá")
println!("{}", s.is_char_boundary(3));    // false
```

`get` devolve `None` em vez de parar o programa. O tratamento de `Option` com calma é o Módulo 04.

## Métodos que dispensam índices

Na prática, você raramente precisa de índices. A biblioteca padrão tem métodos para a maioria das tarefas:

```rust
println!("{}", "olá".starts_with("ol"));              // true
println!("{}", "olá".contains('á'));                   // true
println!("{:?}", "  oi  ".trim());                     // "oi"
for p in "  um  dois três ".split_whitespace() {
    print!("[{p}]");                                   // [um][dois][três]
}
println!();
println!("{:?}", "pão de queijo".find("de"));          // Some(5)
println!("{}", "straße".to_uppercase());               // STRASSE
```

Dois cuidados:

- `find` devolve o índice em **bytes**: `Some(5)`, porque o `ã` de "pão" ocupa 2 bytes. Esse índice é seguro para fatiar (`&s[5..]`), mas não é a posição em caracteres (que seria 4).
- `to_uppercase()` devolve uma `String` nova, e ela pode ter outro número de caracteres: `"straße"` tem 6 e `"STRASSE"` tem 7.

## Comparando com Python e C++

| | `str` (Python 3) | `std::string` (C++) | `String` / `&str` (Rust) |
|---|---|---|---|
| O que guarda | *code points* | bytes, sem codificação definida | bytes UTF-8, sempre válidos |
| tamanho de `"olá"` | `len(s)` = 3 | `s.size()` = 4 | `s.len()` = 4; `s.chars().count()` = 3 |
| `s[2]` | `'á'` | o byte 195 | não compila (E0277) |
| cortar no meio de um caractere | impossível | `s.substr(0, 3)` gera UTF-8 inválido, em silêncio | *panic*, ou `None` com `get` |
| tamanho de `"👍🏽"` | 2 | 8 | 8 bytes, 2 `char`s |
| pode alterar? | não (imutável) | sim | `String` sim, `&str` não |

**Python** indexa por *code point* porque guarda o texto internamente com largura fixa (1, 2 ou 4 bytes por caractere, conforme o maior caractere do texto, pela PEP 393). Por isso `s[2]` é O(1). Ainda assim, Python também conta `"👍🏽"` como 2: grafemas são difíceis em qualquer linguagem.

**C++** trata `std::string` como uma sequência de bytes. `s[2]` devolve meio `á` e `substr` corta onde você mandar, sem erro nenhum. Rust tem a mesma representação (bytes UTF-8), mas garante que uma `&str` nunca contém UTF-8 inválido.

## Armadilhas comuns

### Usar `len()` como número de caracteres

Para limitar um texto a "N caracteres", `len()` dá o número errado e `&s[..n]` pode dar *panic*. Use `chars().count()` para contar, e `char_indices()` ou `get` para cortar. O exercício 5 pede exatamente isso.

### Inverter por `char` e quebrar o emoji

```rust
let inv: String = "👍🏽".chars().rev().collect();
println!("{:?}", inv.chars().collect::<Vec<char>>());   // ['🏽', '👍']
```

O modificador de tom de pele vem primeiro e o emoji se desmancha. Inverter "por letra" de verdade exige grafemas (`unicode-segmentation`).

### Misturar índices de bytes e de `char`s

`find` e `char_indices` falam em bytes. `chars().nth()` e `chars().position()` falam em `char`s. Usar um índice de uma visão na outra funciona com texto ASCII e quebra no primeiro acento, o que costuma passar nos testes e falhar em produção. Teste sempre com acentos e emojis.

### Comparar textos que parecem iguais

`"é" == "e\u{301}"` é `false`. Se os textos vêm de fontes diferentes (teclados, sistemas operacionais, bancos de dados), pode ser preciso normalizar antes de comparar (crate `unicode-normalization`).

## Resumo

- `String` e `&str` são sempre UTF-8 válido. Um caractere ocupa de 1 a 4 bytes.
- `len()` conta bytes. `chars().count()` conta valores escalares Unicode. Grafemas exigem uma crate.
- `s[0]` não compila: um índice de byte pode cair no meio de um caractere, e achar o n-ésimo `char` é O(n).
- Use `chars().nth()`, `as_bytes()[i]` ou `char_indices()`, conforme a visão que você precisa.
- Fatiar com intervalo fora de uma fronteira de caractere dá *panic*. `get` e `is_char_boundary` são as versões seguras.
- `find` devolve índices em bytes. Não misture com posições em `char`s.

## Exercícios

Veja os exercícios 5 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 8.2 Storing UTF-8 Encoded Text with Strings](https://doc.rust-lang.org/stable/book/ch08-02-strings.html) (seções "Indexing into Strings", "Slicing Strings" e "Methods for Iterating Over Strings")
- Documentação de [`str`](https://doc.rust-lang.org/std/primitive.str.html) (veja `char_indices`, `get`, `is_char_boundary`)
- [`unicode-segmentation`](https://docs.rs/unicode-segmentation) para grafemas
- [PEP 393: Flexible String Representation](https://peps.python.org/pep-0393/), para entender o lado do Python

---

[← Lição anterior](05-string-e-str.md) · [Índice do módulo](README.md)
