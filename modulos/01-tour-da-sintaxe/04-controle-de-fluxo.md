# 04 — Controle de fluxo

**Objetivos.** Ao final, você vai saber:

- usar `if` como expressão, no lugar do operador ternário;
- escrever um `match` básico e entender por que ele precisa ser exaustivo;
- usar `loop` (inclusive devolvendo valor), `while` e `for` com *ranges*;
- sair de laços aninhados com rótulos.

## `if` só aceita `bool`

```rust
let temperatura = 31;
if temperatura > 30 {
    println!("calor");
} else if temperatura < 15 {
    println!("frio");
} else {
    println!("agradável");
}
```

Os parênteses em volta da condição são dispensáveis (o compilador avisa com `unused_parens` se você colocar), e as chaves são obrigatórias, mesmo com uma linha só.

A diferença real está na condição: ela precisa ser `bool`. Não existe *truthiness*, ou seja, números, listas e textos não viram `true` ou `false` sozinhos:

```rust
// ❌ NÃO COMPILA
fn main() {
    let itens = 3;
    if itens {
        println!("tem itens");
    }
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:3:8
  |
3 |     if itens {
  |        ^^^^^ expected `bool`, found integer
```

Escreva a comparação que você quer dizer: `if itens > 0`. O `if lista:` do Python vira `if !lista.is_empty()`, e o `if (ptr)` do C++ vira um `Option`, que você verá no [Módulo 04](../04-structs-enums-e-patterns/README.md).

## `if` é expressão

Como blocos têm valor (veja a [lição anterior](03-expressoes-blocos-e-funcoes.md)), o `if` também tem. Por isso Rust não tem o operador ternário:

```rust
let idade = 20;
let categoria = if idade >= 18 { "adulto" } else { "menor" };
```

| C++ | Python | Rust |
|---|---|---|
| `c ? a : b` | `a if c else b` | `if c { a } else { b }` |

Os dois braços precisam ter **o mesmo tipo**, porque a variável só pode ter um:

```rust
// ❌ NÃO COMPILA
fn main() {
    let ok = true;
    let n = if ok { 5 } else { "seis" };
}
```

```text
error[E0308]: `if` and `else` have incompatible types
 --> src/main.rs:3:32
  |
3 |     let n = if ok { 5 } else { "seis" };
  |                     -          ^^^^^^ expected integer, found `&str`
  |                     |
  |                     expected because of this
```

## `match`: um `switch` exaustivo

O `match` compara um valor com uma sequência de padrões e executa o braço do primeiro que casar:

```rust
for n in [0_u8, 1, 4, 7, 42] {
    let nome = match n {
        0 => "nenhum",
        1 | 2 => "um ou dois",   // `|`: um padrão OU outro
        3..=9 => "alguns",       // faixa inclusiva
        _ => "muitos",           // `_`: qualquer outro valor
    };
    println!("{n}: {nome}");
}
```

```text
0: nenhum
1: um ou dois
4: alguns
7: alguns
42: muitos
```

Comparado ao `switch` do C++:

- **Não há *fallthrough*.** Só um braço roda, e não existe `break`.
- **É uma expressão.** O `match` acima produz o valor de `nome`. Os braços precisam ter o mesmo tipo, como no `if`.
- **É exaustivo.** Todo valor possível precisa estar coberto por algum braço. Se faltar um caso, o código não compila.

A exaustividade é o ponto mais importante. Veja o que acontece sem o `_`:

```rust
// ❌ NÃO COMPILA
fn main() {
    let n: u8 = 7;
    let nome = match n {
        0 => "nenhum",
        1 => "um",
        2..=9 => "alguns",
    };
    println!("{nome}");
}
```

```text
error[E0004]: non-exhaustive patterns: `10_u8..=u8::MAX` not covered
 --> src/main.rs:3:22
  |
3 |     let nome = match n {
  |                      ^ pattern `10_u8..=u8::MAX` not covered
  |
  = note: the matched value is of type `u8`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
  |
6 ~         2..=9 => "alguns",
7 ~         10_u8..=u8::MAX => todo!(),
  |
```

O compilador calculou exatamente o que faltava: de 10 até 255. Com inteiros, isso parece só um capricho. O valor real aparece com os `enum`s do [Módulo 04](../04-structs-enums-e-patterns/README.md): quando você acrescenta uma variante nova a um tipo, o compilador aponta **todos** os `match` que precisam tratá-la. Em C++, um `switch` sem um caso compila (no máximo com aviso). No Python 3.10+, um `match` sem caso que case simplesmente não faz nada.

O `match` também desestrutura tuplas, *structs* e *enums*, e aceita condições extras nos braços. Tudo isso fica para o Módulo 04. Por agora, literais, `|`, faixas e `_` já resolvem bastante coisa, inclusive casar uma tupla:

```rust
let (x, y) = (0, 5);
let onde = match (x, y) {
    (0, 0) => "na origem",
    (0, _) => "no eixo y",
    (_, 0) => "no eixo x",
    _ => "fora dos eixos",
};
println!("{onde}"); // no eixo y
```

## `loop`, `while` e `for`

### `loop`: repetir até um `break`

`loop` é o `while (true)` do C++ ou o `while True:` do Python, com um extra: o `break` pode **devolver um valor**, e o `loop` inteiro vira uma expressão.

```rust
let mut tentativas = 0;
let resultado = loop {
    tentativas += 1;
    if tentativas * tentativas > 50 {
        break tentativas;
    }
};
println!("resultado: {resultado}"); // resultado: 8
```

Prefira `loop` a `while true`: o compilador entende que o `loop` só termina por um `break` e usa isso na verificação de tipos e de inicialização.

### `while`

```rust
let mut n = 10;
while n > 0 {
    n /= 3;
}
println!("n = {n}"); // n = 0
```

### `for` e *ranges*

O `for` de Rust é o `for ... in` do Python: ele percorre qualquer coisa iterável. Não existe o `for (int i = 0; i < n; i++)` do C. Para contar, use um *range*:

```rust
for i in 0..3 { print!("{i} "); }              // 0 1 2
for i in 1..=3 { print!("{i} "); }             // 1 2 3
for i in (1..=3).rev() { print!("{i} "); }     // 3 2 1
for i in (0..10).step_by(3) { print!("{i} "); } // 0 3 6 9
```

| Python | Rust |
|---|---|
| `range(5)` | `0..5` |
| `range(1, 11)` | `1..=10` |
| `range(0, 10, 2)` | `(0..10).step_by(2)` |
| `reversed(range(5))` | `(0..5).rev()` |

`a..b` exclui o fim, como no Python, e `a..=b` inclui.

Para percorrer uma coleção, percorra a coleção, e não os índices:

```rust
let notas = [7.5, 8.0, 9.5];
for nota in notas {
    println!("{nota}");
}
```

Indexar dentro de um `for i in 0..v.len()` funciona, mas é mais propenso a erro e paga a verificação de limites a cada volta. O Clippy avisa com `needless_range_loop`, como você viu no exercício 3 do Módulo 00.

### Rótulos: sair de laços aninhados

Um laço pode ter um rótulo começando com `'`. `break` e `continue` aceitam o rótulo para agir sobre o laço de fora:

```rust
'externo: for i in 1..=3 {
    for j in 1..=3 {
        if i * j == 4 {
            println!("achei: {i} x {j}"); // achei: 2 x 2
            break 'externo;
        }
    }
}
```

Em C++, isso exigiria um `goto` ou uma variável de controle. O Python não tem nada equivalente.

## Armadilhas comuns

- **Condição que não é `bool`** (E0308 `expected bool, found integer`): escreva a comparação (`> 0`, `!= 0`, `!v.is_empty()`).
- **Braços com tipos diferentes** num `if` ou `match` usado como valor (E0308 `incompatible types`).
- **`match` sem cobrir todos os casos** (E0004): o erro diz o que falta. Use `_` só quando "todo o resto" é mesmo um caso único. Com `enum`s, listar as variantes uma a uma costuma ser melhor, porque o compilador avisa quando surgir uma nova.
- **`for i in 0..v.len()` para ler `v[i]`**: percorra `v` diretamente.
- **`n % 2 == 0` em inteiros sem sinal**: compila, mas o Clippy sugere `n.is_multiple_of(2)` (`manual_is_multiple_of`).

## Resumo

- `if` exige `bool` e é uma expressão: substitui o ternário. Os braços precisam ter o mesmo tipo.
- `match` não tem *fallthrough*, produz valor e é exaustivo. Por enquanto: literais, `|`, faixas `a..=b` e `_`.
- `loop` repete até um `break`, que pode devolver um valor.
- `for x in coleção` e `for i in a..b` (ou `a..=b`), com `.rev()` e `.step_by(n)`.
- Rótulos (`'externo:`) permitem `break` e `continue` em laços de fora.

## Exercícios

Veja os exercícios 2 e 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 3.5 Control Flow](https://doc.rust-lang.org/stable/book/ch03-05-control-flow.html)
- [The Rust Programming Language, 6.2 The match Control Flow Construct](https://doc.rust-lang.org/stable/book/ch06-02-match.html) (usa `enum`s, que vêm no Módulo 04)
- [Comprehensive Rust: Control Flow Basics](https://google.github.io/comprehensive-rust/control-flow-basics.html)

---

[← Lição anterior](03-expressoes-blocos-e-funcoes.md) · [Índice do módulo](README.md) · [Próxima lição →](05-tuplas-arrays-e-slices.md)
