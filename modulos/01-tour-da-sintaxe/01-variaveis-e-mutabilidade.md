# 01 — Variáveis e mutabilidade

**Objetivos.** Ao final, você vai saber:

- por que variáveis são imutáveis por padrão e quando usar `mut`;
- o que é *shadowing* e por que ele não é a mesma coisa que reatribuir;
- a diferença entre `let` e `const`;
- até onde uma variável existe (escopo de bloco).

## Imutável por padrão

```rust
let x = 5;
let mut y = 5;
y += 1;
println!("{x} {y}"); // 5 6
```

`let` cria uma variável. Sem `mut`, ela não pode mudar depois de inicializada. Tentar mudar é erro de compilação:

```rust
// ❌ NÃO COMPILA
fn main() {
    let x = 5;
    x = 6;
    println!("{x}");
}
```

```text
error[E0384]: cannot assign twice to immutable variable `x`
 --> src/main.rs:3:5
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut x = 5;
  |         +++
```

Não é capricho. Se o compilador sabe que um valor não muda, ele pode garantir que ninguém o altera enquanto outra parte do código está lendo. Essa garantia é a base das regras de *borrowing* (empréstimo) do [Módulo 03](../03-borrowing-e-lifetimes/README.md). E o `mut` também serve de documentação: quem lê sabe na hora quais variáveis mudam.

Para quem vem de outras linguagens:

- **C++:** a mesma ideia do `const`, mas com o padrão invertido. Em C++, você pede `const` quando lembra. Em Rust, você pede `mut` quando precisa.
- **Python:** não existe variável imutável. `MAX_SIZE = 10` é só uma convenção, e nada impede alguém de reatribuir.

> [!TIP]
> Na dúvida, comece sem `mut`. Se precisar, o compilador avisa (e sugere o `mut`). O contrário também vale: um `mut` desnecessário gera o aviso `unused_mut`.

## *Shadowing*: um novo `let` com o mesmo nome

Você pode declarar de novo uma variável com o mesmo nome. Isso se chama *shadowing* (sombreamento): o novo `let` cria **outra variável**, e a anterior fica inacessível.

```rust
let espacos = "   ";          // &str (texto)
let espacos = espacos.len();  // usize (número): outra variável
println!("{espacos}");        // 3
```

O uso típico é transformar um valor em etapas sem inventar nomes como `espacos_str` e `espacos_num`. Repare em duas coisas:

1. **O tipo pode mudar**, porque é outra variável.
2. **As duas continuam imutáveis.** Depois da transformação, ninguém altera o resultado por acidente.

Com `mut` não dá para fazer o mesmo, porque `mut` permite mudar o *valor*, nunca o *tipo*:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut espacos = "   ";
    espacos = espacos.len();
    println!("{espacos}");
}
```

```text
error[E0308]: mismatched types
 --> src/main.rs:3:15
  |
2 |     let mut espacos = "   ";
  |                       ----- expected due to this value
3 |     espacos = espacos.len();
  |               ^^^^^^^^^^^^^ expected `&str`, found `usize`
```

O rótulo secundário explica: o tipo de `espacos` foi decidido na linha 2 (`&str`, um texto), e um `usize` não cabe ali. O `&str` aparece bastante neste módulo como "o tipo de um texto". O que ele é de verdade fica para o [Módulo 02](../02-ownership/README.md).

Compare com Python, onde a reatribuição faz o mesmo nome apontar para outro objeto, de qualquer tipo:

```python
contagem = "10"
contagem = int(contagem)   # mesma variável, agora com um int
```

```rust
let contagem = "10";
let contagem: i32 = contagem.parse().unwrap(); // NOVA variável; a anterior some
```

O resultado parece igual, mas em Rust são duas variáveis, cada uma com um tipo fixo. O `parse().unwrap()` converte texto em número e aparece na [próxima lição](02-tipos-primitivos-e-conversoes.md).

## Escopo de bloco

Uma variável existe do `let` até o `}` do bloco onde foi declarada. Um *shadowing* dentro de um bloco vale só até o fim dele:

```rust
let x = 5;
{
    let x = x * 2;
    println!("dentro do bloco: {x}"); // 10
}
println!("fora do bloco: {x}");       // 5
```

Usar a variável fora do bloco é erro:

```rust
// ❌ NÃO COMPILA
fn main() {
    let x = 1;
    {
        let y = x + 1;
        println!("{y}");
    }
    println!("{y}");
}
```

```text
error[E0425]: cannot find value `y` in this scope
 --> src/main.rs:7:16
  |
7 |     println!("{y}");
  |                ^
  |
help: the binding `y` is available in a different scope in the same function
 --> src/main.rs:4:13
  |
4 |         let y = x + 1;
  |             ^
```

Em Python, uma variável criada dentro de um `if` ou `for` continua viva depois dele, até o fim da função. Em Rust (como em C++), o bloco é o limite. Isso vai importar muito no [Módulo 02](../02-ownership/README.md): quando a variável sai de escopo, o valor dela é liberado.

## Constantes

```rust
const SEGUNDOS_POR_HORA: u32 = 60 * 60;

fn main() {
    println!("{SEGUNDOS_POR_HORA}");
}
```

Uma `const` é diferente de um `let` sem `mut`:

| | `let` | `const` |
|---|---|---|
| tipo | pode ser inferido | **obrigatório** |
| valor | qualquer expressão, calculada em tempo de execução | só expressões que o compilador calcula na compilação |
| onde | dentro de funções | em qualquer lugar, inclusive fora de funções (global) |
| nome | `snake_case` | `MAIUSCULAS_COM_SUBLINHADO` |

Existe também o `static`, uma variável global com endereço fixo na memória. Você quase não vai precisar dele agora. Estado global (e por que `static mut` é uma má ideia) é assunto do Módulo 11.

## Armadilhas comuns

- **Esquecer o `mut`** (E0384): o `help:` do compilador já mostra a correção. Mas pergunte-se antes: a variável precisa mesmo mudar, ou um *shadowing* deixaria o código mais claro?
- **Usar `mut` para trocar o tipo** (E0308): use *shadowing* (`let x = ...;` de novo).
- **Achar que o *shadowing* dentro de um bloco altera a variável de fora.** Não altera: o novo `x` morre no `}`, e o de fora continua com o valor antigo.
- **Variável criada dentro de um bloco e usada depois** (E0425): declare antes do bloco ou faça o bloco devolver o valor (veja a [lição 03](03-expressoes-blocos-e-funcoes.md)).

## Resumo

- Variáveis são imutáveis por padrão. Use `mut` quando o valor precisa mudar.
- *Shadowing* cria uma nova variável com o mesmo nome. Pode mudar o tipo, e o resultado continua imutável.
- `mut` muda o valor, nunca o tipo.
- Variáveis vivem até o fim do bloco (`}`) em que foram declaradas.
- `const` exige tipo, é calculada na compilação e pode ser global.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 3.1 Variables and Mutability](https://doc.rust-lang.org/stable/book/ch03-01-variables-and-mutability.html)
- [Comprehensive Rust: Variables](https://google.github.io/comprehensive-rust/types-and-values/variables.html)
- [Rust Reference: Constant items](https://doc.rust-lang.org/reference/items/constant-items.html)

---

[Índice do módulo](README.md) · [Próxima lição →](02-tipos-primitivos-e-conversoes.md)
