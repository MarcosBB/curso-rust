# 04 — Lifetimes em funções

**Objetivos.** Ao final, você vai saber:

- reconhecer referências penduradas (E0597, E0515, E0716) e por que o compilador as recusa;
- quando uma assinatura precisa de `<'a>` (E0106) e o que a anotação quer dizer;
- relacionar só os parâmetros que importam ao retorno (E0621);
- preferir devolver um valor com posse quando não há de onde emprestar.

## Referências penduradas

Uma *dangling reference* (referência pendurada) aponta para um valor que já foi liberado. Em C++, é um dos bugs mais comuns. Em Rust seguro, ela não compila. O caso mais direto:

```rust
// ❌ NÃO COMPILA
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }
    println!("r: {r}");
}
```

```text
error[E0597]: `x` does not live long enough
 --> src/main.rs:5:13
  |
4 |         let x = 5;
  |             - binding `x` declared here
5 |         r = &x;
  |             ^^ borrowed value does not live long enough
6 |     }
  |     - `x` dropped here while still borrowed
7 |     println!("r: {r}");
  |                   - borrow later used here
```

Os rótulos mostram as duas regiões: `x` existe das linhas 4 a 6, e o empréstimo `&x` precisaria durar até a linha 7. O empréstimo não pode durar mais que o dono.

Esta é a palavra que dá nome à lição: o *lifetime* (tempo de vida) de uma referência é a região do código em que ela precisa ser válida. O compilador calcula essas regiões sozinho dentro de uma função, como você viu na lição 02.

O mesmo problema, atravessando uma função:

```rust
// ❌ NÃO COMPILA
fn make(prefix: &str) -> &str {
    let s = format!("{prefix}!");
    &s
}

fn main() {
    println!("{}", make("oi"));
}
```

```text
error[E0515]: cannot return reference to local variable `s`
 --> src/main.rs:3:5
  |
3 |     &s
  |     ^^ returns a reference to data owned by the current function
```

`s` é liberada quando `make` retorna. A correção é devolver o valor em si, com posse: `fn make(prefix: &str) -> String`, terminando em `s` (ou direto em `format!(...)`).

Em C++, o caso direto até gera aviso no clang (`-Wreturn-stack-address`). Mas basta um passo de indireção para o aviso sumir. A seção seguinte mostra um desses casos.

## Quando o compilador pede ajuda: E0106

Uma função que devolve a maior de duas strings:

```rust
// ❌ NÃO COMPILA
fn longest(x: &str, y: &str) -> &str {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    println!("{}", longest("abc", "de"));
}
```

```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:33
  |
1 | fn longest(x: &str, y: &str) -> &str {
  |               ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
  |
1 | fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
  |           ++++     ++          ++          ++
```

O `help` diz o essencial: a assinatura não informa **de quem** o retorno empresta. Você olha o corpo e vê que pode ser de qualquer um. Mas o compilador, ao verificar quem **chama** `longest`, nunca olha o corpo de `longest`. Ele usa só a assinatura.

Isso é uma escolha de projeto, e uma boa: a assinatura é um **contrato**. Quem chama confia nela, e o corpo é verificado contra ela. Se você mudar o corpo sem mudar a assinatura, nenhum código que chama a função quebra. E a verificação de cada função é local e rápida.

## A sintaxe `<'a>`

A correção sugerida:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}
```

- `<'a>` declara um **parâmetro de lifetime**. Assim como `<T>` (Módulo 06) declara "um tipo qualquer, escolhido por quem chama", `<'a>` declara "uma região qualquer, escolhida por quem chama".
- `&'a str` é "uma referência para `str` válida pelo menos durante a região `'a`".
- O nome é livre (`'a`, `'b`, `'texto`), mas o costume é usar letras curtas.

Lida em voz alta, a assinatura diz: **"para alguma região `'a` em que `x` e `y` sejam ambas válidas, o retorno também é válido em `'a`"**. Na prática, o compilador escolhe para `'a` a **interseção** das regiões dos argumentos, e o retorno não pode ser usado fora dela.

> [!NOTE]
> A anotação não muda quanto tempo nada vive. Ela não estende a vida de `x` nem de `y`. Ela só **descreve uma relação** entre as referências, para o compilador verificar os dois lados: o corpo precisa cumprir o contrato, e quem chama precisa respeitá-lo.

## O contrato funcionando

Dentro da interseção, tudo certo:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("uma string longa");
    {
        let string2 = String::from("xyz");
        let result = longest(string1.as_str(), string2.as_str());
        println!("a maior é: {result}");
    }
}
```

```text
a maior é: uma string longa
```

Agora, usando o resultado depois que `string2` morreu:

```rust
// ❌ NÃO COMPILA
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    let string1 = String::from("uma string longa");
    let result;
    {
        let string2 = String::from("xyz");
        result = longest(string1.as_str(), string2.as_str());
    }
    println!("a maior é: {result}");
}
```

```text
error[E0597]: `string2` does not live long enough
  --> src/main.rs:10:44
   |
 9 |         let string2 = String::from("xyz");
   |             ------- binding `string2` declared here
10 |         result = longest(string1.as_str(), string2.as_str());
   |                                            ^^^^^^^ borrowed value does not live long enough
11 |     }
   |     - `string2` dropped here while still borrowed
12 |     println!("a maior é: {result}");
   |                           ------ borrow later used here
```

Você sabe que `result` aponta para `string1` (a mais longa), então "na verdade" seria seguro. Mas o contrato diz que o retorno pode vir de qualquer um dos dois, e o compilador confia no contrato, não nos valores:

![O código de main com barras ao lado. string1 (verde) vale da linha 1 à 8. string2 (verde) vale das linhas 4 a 6. A região 'a (roxa) é a interseção, linhas 5 e 6. Uma extensão tracejada cinza mostra que result precisaria valer também na linha 7, onde string2 já foi liberada.](assets/longest-lifetimes.svg)

## Em C++, o mesmo contrato fica implícito

A versão C++ de `longest`, chamada com um temporário:

```cpp
const std::string& maior(const std::string& a, const std::string& b) {
    return a.size() >= b.size() ? a : b;
}

int main() {
    std::string s = "oi";
    const std::string& r = maior(s, std::string("temporário"));
    std::cout << r << "\n";  // r aponta para um temporário já destruído
}
```

Compila sem aviso com `-Wall -Wextra` e, nos testes deste curso, até imprimiu `temporário`. Funcionar por acaso é o pior tipo de bug. Com o AddressSanitizer:

```text
==44447==ERROR: AddressSanitizer: stack-use-after-scope on address 0x00016ce3d577 ...
```

O temporário morre no fim da linha que o criou, e `r` fica pendurada. A mesma ideia em Rust:

```rust
// ❌ NÃO COMPILA
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() { x } else { y }
}

fn main() {
    let s = String::from("oi");
    let r = longest(&s, &String::from("temporário"));
    println!("{r}");
}
```

```text
error[E0716]: temporary value dropped while borrowed
 --> src/main.rs:7:26
  |
7 |     let r = longest(&s, &String::from("temporário"));
  |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^ - temporary value is freed at the end of this statement
  |                          |
  |                          creates a temporary value which is freed while still in use
8 |     println!("{r}");
  |                - borrow later used here
  |
help: consider using a `let` binding to create a longer lived value
  |
7 ~     let binding = String::from("temporário");
8 ~     let r = longest(&s, &binding);
  |
```

C++ não tem como escrever "o retorno vem de `a` ou de `b`" na assinatura. Existe o atributo `[[clang::lifetimebound]]` e há propostas nas C++ Core Guidelines, mas são opcionais e heurísticos. Em Rust, essa informação é obrigatória sempre que o compilador não consegue deduzi-la.

## Relacione só o que importa

Nem todo parâmetro precisa do mesmo lifetime. Se o retorno vem sempre de `x`, diga isso:

```rust
fn first<'a>(x: &'a str, _y: &str) -> &'a str {
    x
}

fn main() {
    let a = String::from("fica");
    let r;
    {
        let b = String::from("morre");
        r = first(&a, &b);
    }
    println!("{r}");
}
```

```text
fica
```

`b` morre antes do `println!`, e está tudo bem: a assinatura promete que o retorno só depende de `x`. Com `<'a>` nos dois parâmetros, este mesmo `main` daria E0597, porque o contrato seria mais restritivo do que precisa.

E se o corpo não cumprir o contrato, o erro aparece **na função**:

```rust
// ❌ NÃO COMPILA
fn pick<'a>(x: &'a str, y: &str) -> &'a str {
    if x.is_empty() { y } else { x }
}

fn main() {
    println!("{}", pick("a", "b"));
}
```

```text
error[E0621]: explicit lifetime required in the type of `y`
 --> src/main.rs:2:23
  |
2 |     if x.is_empty() { y } else { x }
  |                       ^ lifetime `'a` required
  |
help: add explicit lifetime `'a` to the type of `y`
  |
1 | fn pick<'a>(x: &'a str, y: &'a str) -> &'a str {
  |                             ++
```

O corpo devolve `y`, mas a assinatura não promete nada sobre a validade de `y`. O compilador verifica os dois lados do contrato.

> [!TIP]
> Você vai usar exatamente isso no Projeto 1 (`minigrep`): `fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str>`. As linhas encontradas apontam para dentro de `contents`, e a `query` pode morrer logo depois da busca.

## Sem de onde emprestar: devolva um valor com posse

Uma função sem parâmetros de referência não tem de onde emprestar:

```rust
// ❌ NÃO COMPILA
fn greeting() -> &str {
    let s = String::from("olá");
    &s
}

fn main() {
    println!("{}", greeting());
}
```

```text
error[E0106]: missing lifetime specifier
 --> src/main.rs:1:18
  |
1 | fn greeting() -> &str {
  |                  ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
help: consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`
  |
1 | fn greeting() -> &'static str {
  |                   +++++++
help: instead, you are more likely to want to return an owned value
  |
1 - fn greeting() -> &str {
1 + fn greeting() -> String {
  |
```

O compilador oferece duas saídas e já avisa qual é a mais provável. `'static` (lição 06) só serve se o texto existir o programa todo, como um literal. Aqui a `String` é criada na função, então a saída certa é a segunda: devolver `String`, transferindo a posse para quem chamou.

```rust
fn greeting(name: &str) -> String {
    format!("olá, {name}")
}
```

## Armadilhas comuns

**Achar que `'a` faz o dado viver mais.** A anotação descreve, não estende. Se o dado morre cedo, a correção é mudar onde ele vive (ou devolver um valor com posse), e não "ajustar o lifetime".

**Seguir a sugestão de `'static` por reflexo.** Em E0106 sem parâmetros, quase sempre a resposta é devolver `String`/`Vec`. `'static` só para literais, `const` e `static`.

**Pôr o mesmo `'a` em tudo.** Funciona, mas restringe quem chama sem necessidade. Ligue ao retorno só os parâmetros dos quais ele realmente pode vir.

**Temporário morrendo no fim da linha (E0716).** `let r: &str = String::from("x").as_str();` não compila: a `String` temporária morre no `;`. Dê um nome a ela com `let`, como o próprio `help:` sugere.

## Resumo

- O lifetime de uma referência é a região em que ela precisa ser válida. Ele nunca pode ultrapassar a vida do dono (E0597, E0515, E0716).
- O compilador verifica cada chamada usando **só a assinatura** da função. A assinatura é um contrato.
- `<'a>` é um parâmetro genérico de região. `fn f<'a>(x: &'a str, y: &'a str) -> &'a str` diz que o retorno vale onde os dois argumentos valem.
- Anotações descrevem relações. Não mudam a vida de nada.
- Ligue ao retorno só os parâmetros dos quais ele pode vir (E0621 se o corpo não cumprir). Sem de onde emprestar, devolva um valor com posse.

## Exercícios

Veja os exercícios 2 e 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 10.3 Validating References with Lifetimes](https://doc.rust-lang.org/stable/book/ch10-03-lifetime-syntax.html) (o livro mostra lifetimes depois de generics; aqui só precisamos da parte de funções e structs)
- [Rust by Example: Lifetimes](https://doc.rust-lang.org/rust-by-example/scope/lifetime.html)
- [Comprehensive Rust: Lifetimes](https://google.github.io/comprehensive-rust/lifetimes.html)
- [Reference: escopo de temporários](https://doc.rust-lang.org/reference/destructors.html#temporary-scopes)
- Código desta lição: [`codigo/lifetimes`](codigo/lifetimes/)

---

[← Lição anterior](03-slices-em-profundidade.md) · [Índice do módulo](README.md) · [Próxima lição →](05-elisao-e-lifetime-anonimo.md)
