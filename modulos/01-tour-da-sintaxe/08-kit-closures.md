# 08 — Kit: closures

> [!NOTE]
> Lição do **kit mínimo**: só a sintaxe e os usos mais comuns. Como as closures capturam variáveis, o `move` e as traits `Fn`, `FnMut` e `FnOnce` ficam para o Módulo 07.

**Objetivos.** Ao final, você vai saber:

- escrever uma closure, com e sem anotação de tipos;
- por que uma closure não pode ser chamada com tipos diferentes;
- passar closures para `map`, `filter`, `sort_by_key`, `retain` e `unwrap_or_else`.

## A sintaxe

Uma *closure* é uma função anônima que pode usar variáveis do lugar onde foi criada. É o `lambda` do Python e a lambda `[](){}` do C++:

```cpp
auto mais_um = [](int x) { return x + 1; };
```

```python
mais_um = lambda x: x + 1
```

```rust
let mais_um = |x: i32| x + 1;
println!("{}", mais_um(1)); // 2
```

Os parâmetros ficam entre barras `| |`. Comparando com uma função comum, do mais explícito ao mais enxuto:

```text
fn  mais_um_v1   (x: u32) -> u32 { x + 1 }
let mais_um_v2 = |x: u32| -> u32 { x + 1 };
let mais_um_v3 = |x|             { x + 1 };
let mais_um_v4 = |x|               x + 1  ;
```

Ao contrário das funções, as closures podem dispensar os tipos: o compilador deduz pelo uso. O corpo pode ser uma expressão só ou um bloco com várias instruções (algo que o `lambda` do Python não permite):

```rust
let descreve = |x: i32| {
    let paridade = if x % 2 == 0 { "par" } else { "ímpar" };
    format!("{x} é {paridade}")
};
println!("{}", descreve(7)); // 7 é ímpar
```

## Capturando o ambiente

A closure enxerga as variáveis do escopo onde foi criada, sem precisar recebê-las como parâmetro:

```rust
let fator = 3;
let vezes_fator = |x| x * fator;
println!("{}", vezes_fator(5)); // 15
```

Em C++, você declara a captura (`[&]`, `[=]`, `[fator]`). Em Rust, o compilador deduz pelo corpo da closure como cada variável é usada. Os detalhes dessa captura e o `move` (que força a closure a tomar posse do que usa) ficam para o Módulo 07. Para os exercícios, basta saber que funciona.

## A inferência trava no primeiro uso

Uma closure sem anotação **não é genérica**. O tipo dos parâmetros é decidido no primeiro uso e fica fixo:

```rust
// ❌ NÃO COMPILA
fn main() {
    let ident = |x| x;
    let s = ident(String::from("oi"));
    let n = ident(5);
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:4:19
  |
4 |     let n = ident(5);
  |             ----- ^ expected `String`, found integer
  |             |
  |             arguments to this function are incorrect
  |
note: expected because the closure was earlier called with an argument of type `String`
 --> src/main.rs:3:19
  |
3 |     let s = ident(String::from("oi"));
  |             ----- ^^^^^^^^^^^^^^^^^^ expected because this argument is of type `String`
  |             |
  |             in this closure call
...
```

A `note:` conta a história: a primeira chamada fixou o tipo de `x` como `String`. Em Python, `lambda x: x` aceita qualquer coisa. Em Rust, para uma função que aceita vários tipos, você usa *generics* (Módulo 06).

## Onde as closures aparecem

Na prática, você raramente guarda uma closure numa variável. O uso comum é passar uma closure para um método que decide **quando** chamá-la:

```rust
let numeros = vec![3, 1, 4, 1, 5];

// transformar cada elemento
let dobrados: Vec<i32> = numeros.iter().map(|x| x * 2).collect();
println!("{dobrados:?}"); // [6, 2, 8, 2, 10]

let mut pessoas = vec![("Ana", 31), ("Bruno", 25), ("Carla", 40)];

// ordenar por um critério
pessoas.sort_by_key(|p| p.1);
println!("{pessoas:?}"); // [("Bruno", 25), ("Ana", 31), ("Carla", 40)]

// contar os que satisfazem uma condição
let maiores_de_30 = pessoas.iter().filter(|p| p.1 > 30).count();
println!("{maiores_de_30}"); // 2

// manter só os que satisfazem uma condição (altera o Vec)
pessoas.retain(|p| p.1 >= 30);
println!("{pessoas:?}"); // [("Ana", 31), ("Carla", 40)]
```

O padrão `v.iter().map(...).collect()` é o equivalente de uma *list comprehension* do Python (`[x * 2 for x in numeros]`). O `.iter()` percorre o vetor sem consumi-lo, e o `.collect()` junta o resultado num novo `Vec`, cujo tipo vem da anotação. Por enquanto, use como receita: iteradores e seus adaptadores são o centro do Módulo 07.

Outro uso frequente é calcular um valor padrão **só quando for preciso**:

```rust
let vazio: Vec<i32> = Vec::new();
let primeiro = vazio.first().copied().unwrap_or_else(|| {
    eprintln!("vetor vazio, usando 0");
    0
});
```

A closure sem parâmetros (`|| ...`) só roda se o `Option` for `None`. Com `unwrap_or(valor)`, o valor é calculado sempre, mesmo quando não é usado.

### Funções também são valores

Uma closure que só repassa o argumento para uma função é redundante, e o Clippy avisa:

```rust
fn dobro(x: i32) -> i32 {
    x * 2
}

fn main() {
    let numeros = vec![3, 1, 4];
    let v: Vec<i32> = numeros.into_iter().map(|x| dobro(x)).collect();
    println!("{v:?}");
}
```

```text
warning: redundant closure
 --> src/main.rs:7:47
  |
7 |     let v: Vec<i32> = numeros.into_iter().map(|x| dobro(x)).collect();
  |                                               ^^^^^^^^^^^^ help: replace the closure with the function itself: `dobro`
```

Você pode passar o nome da função direto: `.map(dobro)`. Em qualquer lugar que aceita uma closure, uma função com a assinatura certa também serve.

## Armadilhas comuns

- **Chamar a mesma closure com tipos diferentes** (E0308 com `the closure was earlier called with...`): anote o tipo do parâmetro ou crie outra closure.
- **`|x| f(x)`**: passe `f` direto (`redundant_closure`).
- **`unwrap_or(calculo_caro())`**: o cálculo roda sempre, mesmo quando o valor existe. Use `unwrap_or_else(calculo_caro)` (ou uma closure `|| ...`, quando houver mais de uma chamada).
- **Assustar-se com `&&` dentro de um `filter`.** Com números, o parâmetro de um `filter` sobre `.iter()` é uma referência para uma referência, e às vezes é preciso escrever `**x`. Com tuplas e acesso a campo (`p.1`), isso não aparece. O porquê fica para o Módulo 07.

## Resumo

- Closure: `|parâmetros| expressão` ou `|parâmetros| { bloco }`. Os tipos são opcionais.
- Ela enxerga as variáveis do escopo onde foi criada.
- Não é genérica: o tipo de cada parâmetro fica fixo no primeiro uso.
- Usos comuns: `map`, `filter(...).count()`, `sort_by_key`, `retain` e `unwrap_or_else`.
- Funções podem ser passadas no lugar de closures: `.map(dobro)`.

## Exercícios

Veja os exercícios 5 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 13.1 Closures](https://doc.rust-lang.org/stable/book/ch13-01-closures.html)
- [Rust by Example: Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html)

---

[← Lição anterior](07-kit-vec-e-hashmap.md) · [Índice do módulo](README.md) · [Próxima lição →](09-kit-modulos-lib-e-main.md)
