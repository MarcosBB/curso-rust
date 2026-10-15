# 06 — Macros e formatação

**Objetivos.** Ao final, você vai saber:

- o que o `!` em `println!` significa;
- formatar valores com `{}`, `{:?}` e os especificadores de largura, alinhamento e precisão;
- depurar com `dbg!`;
- quando usar `vec!`, `assert!`, `todo!`, `unreachable!` e `eprintln!`.

## O `!` indica uma macro

`println!` não é uma função: é uma **macro**, e o `!` no nome indica isso. Uma macro é expandida em código durante a compilação. É por isso que `println!` aceita qualquer número de argumentos (Rust não tem funções variádicas) e consegue conferir a *format string* na compilação: um `{}` sem argumento correspondente é erro de compilação, e não uma surpresa em tempo de execução.

Por enquanto, isso basta: use as macros como se fossem funções, lembrando do `!`. Como escrever as suas próprias é o assunto do Módulo 14.

## `{}` x `{:?}`

```rust
let nome = "Ana";
let v = vec![1, 2, 3];
let t = ("x", 1);
println!("{nome}");        // Ana
println!("{v:?} {t:?}");   // [1, 2, 3] ("x", 1)
```

- `{}` usa a *trait* `Display`: a formatação **para o usuário**. Números, `bool`, `char` e textos têm.
- `{:?}` usa a *trait* `Debug`: a formatação **para o programador**. Quase todos os tipos da biblioteca padrão têm, inclusive `Vec`, tuplas e `Option`.
- `{:#?}` é o `Debug` "bonito", com um elemento por linha.

Coleções não têm `Display`, porque não existe um jeito único de mostrar uma lista para o usuário (com colchetes? separada por vírgulas? uma por linha?):

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = vec![1, 2, 3];
    println!("{}", v);
}
```

```text
error[E0277]: `Vec<{integer}>` doesn't implement `std::fmt::Display`
 --> src/main.rs:3:20
  |
3 |     println!("{}", v);
  |               --   ^ `Vec<{integer}>` cannot be formatted with the default formatter
  |               |
  |               required by this formatting parameter
  |
  = help: the trait `std::fmt::Display` is not implemented for `Vec<{integer}>`
  = note: in format strings you may be able to use `{:?}` (or `{:#?}` for pretty-print) instead
```

A `note:` já dá a resposta: use `{:?}`. Para os seus próprios tipos, `Debug` se ganha com uma linha (`#[derive(Debug)]`, Módulo 04), e `Display` se implementa à mão (Módulo 06).

## Argumentos e especificadores

O nome de uma variável pode ir direto entre as chaves, como na f-string do Python. Expressões, não: `{x + 1}` não funciona, e aí você usa um argumento posicional.

```rust
let x = 2;
println!("{x} + 1 = {}", x + 1);        // 2 + 1 = 3
println!("{0} {1} {0}", "a", "b");      // a b a
```

Depois do `:` vêm os especificadores, quase iguais aos do Python:

```rust
let nome = "Ana";
let pi = 3.14159;
let largura = 10;
println!("[{nome:>8}] [{nome:<6}] [{nome:^7}]");
println!("{pi:.2} {:05} {:#x} {:#b} {:e}", 42, 255, 5, 1234.5);
println!("[{nome:>largura$}]");
```

```text
[     Ana] [Ana   ] [  Ana  ]
3.14 00042 0xff 0b101 1.2345e3
[       Ana]
```

| Especificador | Efeito |
|---|---|
| `{:>8}`, `{:<8}`, `{:^8}` | alinha à direita, à esquerda ou ao centro em 8 colunas |
| `{:.2}` | 2 casas decimais |
| `{:05}` | completa com zeros até 5 dígitos |
| `{:#x}`, `{:#b}` | hexadecimal e binário com prefixo |
| `{:e}` | notação científica |
| `{nome:>largura$}` | largura vinda de uma variável |

Os especificadores também valem com `Debug`: `{pi:.1?}`, por exemplo.

## A família do `println!`

| Macro | O que faz |
|---|---|
| `println!` / `print!` | escreve no **stdout**, com ou sem quebra de linha |
| `eprintln!` / `eprint!` | escreve no **stderr**: mensagens de erro e diagnósticos |
| `format!` | devolve uma `String` em vez de imprimir |

```rust
let nome = "Ana";
let s = format!("{nome} tem {} letras", nome.len());
println!("{s}");                       // Ana tem 3 letras
eprintln!("isto vai para o stderr");
```

`format!` devolve uma `String`, o texto que pertence a quem o criou e pode crescer. A diferença entre `String` e `&str` é um dos primeiros assuntos do [Módulo 02](../02-ownership/README.md).

## `dbg!`: o `print` de depuração

`dbg!` imprime no stderr o arquivo, a linha, a expressão e o valor, e **devolve o valor**. Por isso dá para embrulhar só um pedaço de uma expressão, sem mudar o resto:

```rust
let a = 2;
let b = dbg!(a * 3) + 1;
dbg!(b);
```

```text
[src/main.rs:3:13] a * 3 = 6
[src/main.rs:4:5] b = 7
```

É mais prático que um `println!("{:?}", ...)` provisório. Só não deixe o `dbg!` no código final: o Clippy tem um lint para isso (`dbg_macro`), desligado por padrão, que muitos projetos ligam.

## Outras macros que você vai ver logo

```rust
let v = vec![1, 2, 3];          // cria um Vec com esses elementos (lição 07)
assert!(v.len() == 3);          // panic se a condição for falsa
assert_eq!(v[0], 1);            // panic se os dois lados forem diferentes (lição 10)
```

E duas que marcam código "que não deveria rodar":

```rust
fn ainda_vou_escrever(x: i32) -> i32 {
    todo!() // compila; dá panic "not yet implemented" se for chamada
}

fn eh_dia_util(dia_da_semana: u8) -> bool {
    match dia_da_semana {
        1..=5 => true,
        6 | 7 => false,
        _ => unreachable!("dia inválido: {dia_da_semana}"),
    }
}
```

- `todo!()` compila em qualquer lugar porque é do tipo `!` (veja a [lição 03](03-expressoes-blocos-e-funcoes.md)). É o que os exercícios deste curso usam no lugar do código que você vai escrever: o projeto compila desde o começo, e os testes falham até você implementar. Enquanto o `todo!()` estiver lá, o compilador avisa que os parâmetros não são usados (`unused_variables`), e isso é esperado.
- `unreachable!()` documenta "isto nunca deveria acontecer" e dá *panic* com a mensagem se acontecer.

## Armadilhas comuns

- **`{}` num `Vec`, tupla ou `Option`** (E0277 `doesn't implement Display`): use `{:?}`.
- **Expressão dentro das chaves**: `{x + 1}` não funciona. Use `{}` com o argumento depois da *format string*.
- **Esquecer o `!`**: `println("oi")` é erro, porque não existe uma *função* `println`.
- **Mensagem de erro no stdout.** Use `eprintln!`, para que ela apareça mesmo quando a saída do programa for redirecionada para um arquivo. O Projeto 1 depende disso.

## Resumo

- O `!` indica uma macro, expandida na compilação. Ela aceita argumentos variáveis e confere a *format string*.
- `{}` = `Display` (usuário), `{:?}` = `Debug` (programador), `{:#?}` = `Debug` formatado.
- Especificadores como no Python: `{:>8}`, `{:.2}`, `{:05}`, `{:#x}`, `{nome:>largura$}`.
- `eprintln!` escreve no stderr, `format!` devolve uma `String` e `dbg!` imprime e devolve o valor.
- `todo!()` e `unreachable!()` marcam código que não deveria rodar.

## Exercícios

As macros aparecem em todos os exercícios de [`exercicios.md`](exercicios.md).

## Leitura complementar

- [std: módulo `fmt`](https://doc.rust-lang.org/std/fmt/index.html) (a referência completa dos especificadores)
- [Comprehensive Rust: Macros](https://google.github.io/comprehensive-rust/control-flow-basics/macros.html)
- [std: macro `dbg!`](https://doc.rust-lang.org/std/macro.dbg.html)

---

[← Lição anterior](05-tuplas-arrays-e-slices.md) · [Índice do módulo](README.md) · [Próxima lição →](07-kit-vec-e-hashmap.md)
