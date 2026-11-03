# 04 — Builders e outros idiomas de construção

**Objetivos.** Ao final, você vai saber:

- construir structs com muitos campos opcionais usando `Default` e a sintaxe de atualização (`..Default::default()`);
- escrever um *builder*, nas versões que consomem `self` e que recebem `&mut self`;
- usar `#[must_use]` para avisar de um builder esquecido;
- exigir campos obrigatórios na compilação com um builder *typestate*.

## O problema: Rust não tem argumentos nomeados nem valores padrão

Em Python, uma função com dez opções é normal:

```python
def requisicao(url, metodo="GET", tempo_limite=30, cabecalhos=None, tentativas=0):
    ...

requisicao("https://exemplo.com", tempo_limite=5)
```

Em C++, você tem parâmetros com valor padrão (só no fim, e posicionais) e, desde o C++20, *designated initializers* para structs: `Opcoes{.largura = 40}`.

Rust não tem nenhum dos dois. Uma função recebe todos os argumentos, na ordem, sempre. A decisão foi deliberada (as chamadas ficam explícitas, e acrescentar um parâmetro é sempre visível), mas deixa a pergunta: como criar um valor que tem muitas opções, quase todas com um padrão razoável? Há dois idiomas, e a escolha depende de a struct ter campos públicos ou não.

## Idioma 1: `Default` e a sintaxe de atualização

Se os campos são públicos e todos têm um padrão sensato, implemente `Default` (Módulo 06) e use a sintaxe de atualização de struct (Módulo 04):

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Opcoes {
    pub largura: usize,
    pub recuo: usize,
    pub quebrar_palavras: bool,
    pub prefixo: String,
}

impl Default for Opcoes {
    fn default() -> Self {
        Opcoes {
            largura: 80,
            recuo: 0,
            quebrar_palavras: true,
            prefixo: String::new(),
        }
    }
}

let o = Opcoes {
    largura: 40,
    prefixo: "> ".to_string(),
    ..Default::default()
};
```

```text
Opcoes { largura: 40, recuo: 0, quebrar_palavras: true, prefixo: "> " }
```

É o mais próximo dos argumentos nomeados do Python: você escreve só o que muda, pelo nome. Funciona bem para structs de configuração simples, sem invariantes entre os campos.

Os limites aparecem rápido. Os campos precisam ser `pub`, então não há validação (nada impede `largura: 0`). Acrescentar um campo, mesmo com padrão, quebra quem constrói a struct sem `..Default::default()`. E não dá para exigir um campo: todos têm padrão.

## Idioma 2: o builder

Um *builder* é um tipo separado que acumula as opções e, no fim, constrói o valor com `build()`. Cada método de configuração devolve o próprio builder, para encadear:

```rust
pub struct Requisicao {
    url: String,
    metodo: &'static str,
    cabecalhos: Vec<(String, String)>,
    tempo_limite: Duration,
}

pub struct RequisicaoBuilder {
    url: String,
    metodo: &'static str,
    cabecalhos: Vec<(String, String)>,
    tempo_limite: Duration,
}

impl RequisicaoBuilder {
    pub fn new(url: impl Into<String>) -> Self {
        RequisicaoBuilder {
            url: url.into(),
            metodo: "GET",
            cabecalhos: Vec::new(),
            tempo_limite: Duration::from_secs(30),
        }
    }

    pub fn metodo(mut self, m: &'static str) -> Self {
        self.metodo = m;
        self
    }

    pub fn cabecalho(mut self, nome: &str, valor: &str) -> Self {
        self.cabecalhos.push((nome.to_string(), valor.to_string()));
        self
    }

    pub fn tempo_limite(mut self, t: Duration) -> Self {
        self.tempo_limite = t;
        self
    }

    pub fn build(self) -> Requisicao {
        Requisicao {
            url: self.url,
            metodo: self.metodo,
            cabecalhos: self.cabecalhos,
            tempo_limite: self.tempo_limite,
        }
    }
}

let r = RequisicaoBuilder::new("https://exemplo.com/api")
    .cabecalho("Accept", "application/json")
    .tempo_limite(Duration::from_secs(5))
    .build();
```

O que se ganha em relação ao `Default`:

- **Campos privados.** `Requisicao` pode manter uma invariante, e o `build()` pode validar (devolvendo `Result<Requisicao, Erro>` se algo puder dar errado).
- **Obrigatórios no construtor.** A URL vai no `new`; não existe requisição sem URL.
- **Métodos mais ricos que um campo.** `cabecalho` acrescenta a uma lista; um `json(&valor)` poderia preencher corpo e `Content-Type` ao mesmo tempo.
- **Evoluir sem quebrar.** Acrescentar uma opção é acrescentar um método.

Repare no `mut self`: o método recebe o builder **por valor**, o muda e o devolve. Isso permite a cadeia `.a().b().build()` numa expressão só, e o `build(self)` pode mover os campos sem clonar.

### `self` ou `&mut self`?

Há duas escolas, e a biblioteca padrão usa as duas.

| | Consome `self` | Recebe `&mut self` |
|---|---|---|
| Assinatura | `fn opcao(mut self, x) -> Self` | `fn opcao(&mut self, x) -> &mut Self` |
| Cadeia numa expressão | sim | sim, mas o `build` não pode mover |
| Configurar em vários passos (`if`) | `b = b.opcao(x);` | `b.opcao(x);` |
| Reaproveitar o builder | não (foi consumido) | sim |
| Na std | `thread::Builder` | `process::Command`, `fs::OpenOptions` |

O `Command` mostra por que `&mut self` existe: é comum montar um comando aos poucos, com `if`, e rodar o mesmo comando mais de uma vez.

```rust
use std::process::Command;

let mut cmd = Command::new("ls");
cmd.arg("-l");
if mostrar_ocultos {
    cmd.arg("-a");
}
let saida = cmd.output()?;
```

Com um builder que consome, o mesmo `if` vira `if mostrar_ocultos { b = b.opcao(); }`. Funciona, só é menos natural. A regra prática: consumir `self` quando o `build` precisa mover campos caros (`String`, `Vec`) e o valor é construído uma vez; `&mut self` quando o objeto final é "executado" e pode ser reaproveitado.

### `#[must_use]`: o builder esquecido

Um erro comum com builders que consomem `self`: chamar um método e jogar fora o resultado.

```rust
let b = Requisicao::get();
b.url("https://exemplo.com"); // o builder com a URL foi descartado aqui
```

Marque o tipo do builder com `#[must_use]`, e o compilador avisa:

```rust
#[must_use = "um builder não faz nada até `build()`"]
pub struct RequisicaoBuilder<U> { /* ... */ }
```

```text
warning: unused `RequisicaoBuilder` that must be used
 --> src/main.rs:5:5
  |
5 |     b.url("https://exemplo.com");
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: um builder não faz nada até `build()`
  = note: `#[warn(unused_must_use)]` (part of `#[warn(unused)]`) on by default
help: use `let _ = ...` to ignore the resulting value
```

O `#[must_use]` também vale num método só (`#[must_use] fn verboso(self, v: bool) -> Self`). É o mesmo mecanismo que faz o compilador reclamar de um `Result` ignorado (Módulo 05) e de uma future sem `.await` (Módulo 12).

## Idioma 3: builder com typestate

E se a URL não puder ir no construtor, porque você quer `Requisicao::get().url(...)` e `Requisicao::post().url(...)`? Aí ela vira opcional no builder, e um `build()` sem URL só seria pego em tempo de execução (com um `Result` ou um `unwrap`).

Junte o builder com o typestate da lição anterior: o builder ganha um parâmetro que diz se a URL já foi informada, e `build` só existe no estado certo.

```rust
pub struct SemUrl;
pub struct ComUrl;

#[must_use = "um builder não faz nada até `build()`"]
pub struct RequisicaoBuilder<U> {
    url: Option<String>,
    metodo: &'static str,
    cabecalhos: Vec<(String, String)>,
    tempo_limite: Duration,
    _u: PhantomData<U>,
}

impl Requisicao {
    pub fn get() -> RequisicaoBuilder<SemUrl> {
        RequisicaoBuilder::new("GET")
    }

    pub fn post() -> RequisicaoBuilder<SemUrl> {
        RequisicaoBuilder::new("POST")
    }
}

impl RequisicaoBuilder<SemUrl> {
    pub fn url(self, url: impl Into<String>) -> RequisicaoBuilder<ComUrl> {
        RequisicaoBuilder {
            url: Some(url.into()),
            metodo: self.metodo,
            cabecalhos: self.cabecalhos,
            tempo_limite: self.tempo_limite,
            _u: PhantomData,
        }
    }
}

// Opcionais: valem antes e depois da URL.
impl<U> RequisicaoBuilder<U> {
    pub fn cabecalho(mut self, nome: &str, valor: &str) -> Self { /* ... */ }
    pub fn tempo_limite(mut self, t: Duration) -> Self { /* ... */ }
}

impl RequisicaoBuilder<ComUrl> {
    pub fn build(self) -> Requisicao {
        Requisicao {
            url: self.url.expect("o estado ComUrl garante a URL"),
            metodo: self.metodo,
            cabecalhos: self.cabecalhos,
            tempo_limite: self.tempo_limite,
        }
    }
}
```

A ordem dos opcionais é livre, e esquecer a URL é erro de compilação:

```rust
// ✅
let req = Requisicao::get()
    .url("https://exemplo.com/api/pedidos")
    .cabecalho("Accept", "application/json")
    .tempo_limite(Duration::from_secs(5))
    .build();
println!("{req:?}");
```

```text
Requisicao { url: "https://exemplo.com/api/pedidos", metodo: "GET", cabecalhos: [("Accept", "application/json")], tempo_limite: 5s }
```

```rust
// ❌ NÃO COMPILA
let r = Requisicao::get().cabecalho("Accept", "text/html").build();
```

```text
error[E0599]: no method named `build` found for struct `RequisicaoBuilder<SemUrl>` in the current scope
 --> src/main.rs:4:64
  |
4 |     let r = Requisicao::get().cabecalho("Accept", "text/html").build();
  |                                                                ^^^^^ method not found in `RequisicaoBuilder<SemUrl>`
  |
  = note: the method was found for `RequisicaoBuilder<ComUrl>`
```

O `expect` dentro do `build` nunca dispara: o único caminho até `RequisicaoBuilder<ComUrl>` passa por `url()`, que preenche o campo. Ele está ali porque o campo continua sendo `Option` (o mesmo tipo de struct serve aos dois estados).

Com um campo obrigatório, a técnica é ótima. Com três, são oito combinações de estado, e escrever isso à mão deixa de compensar. Crates como [`bon`](https://docs.rs/bon) e [`typed-builder`](https://docs.rs/typed-builder) geram builders com campos obrigatórios verificados na compilação a partir de um `#[derive]` ou de um atributo (são *macros procedurais*, assunto do [Módulo 14](../14-macros/README.md)). [`derive_builder`](https://docs.rs/derive_builder) gera a versão mais simples, que verifica em tempo de execução.

## Outros idiomas pequenos

**`impl Into<String>` em parâmetros de texto.** O `url(self, url: impl Into<String>)` aceita `&str`, `String` e `Box<str>`. Quem tem uma `String` a entrega sem cópia; quem tem um literal não precisa escrever `.to_string()` (o Módulo 06 mostrou o `Into`). Vale para construtores e builders, que vão guardar o texto. Para funções que só leem, `&str` continua sendo o certo.

**`new` sem argumentos + `Default`.** Se `Tipo::new()` não recebe nada, implemente também `Default` (o clippy pede, com o lint `new_without_default`). Assim o tipo funciona com `..Default::default()`, `Option::unwrap_or_default` e `#[derive(Default)]` de quem o usa como campo.

**Construtores com nome.** Rust não tem sobrecarga de funções, então em vez de três `new` com argumentos diferentes você escreve `Requisicao::get()`, `Duration::from_secs(5)`, `String::with_capacity(10)`, `Vec::from_iter(...)`. O nome diz o que o construtor faz, o que costuma ser mais legível que a sobrecarga do C++.

Código completo, com testes, em [`codigo/typestate-e-builder`](codigo/typestate-e-builder/src/lib.rs).

## Armadilhas comuns

- **Builder para struct de três campos.** Se todos são obrigatórios, um `new(a, b, c)` basta. Se são opcionais e públicos, `Default` + `..Default::default()` basta. O builder paga o próprio custo quando há validação, campos privados ou muitas opções.
- **Esquecer o resultado de um método que consome `self`.** `b.url(...)` sem atribuir descarta o builder. Ponha `#[must_use]` no tipo.
- **`build()` que dá panic.** Se uma combinação de opções é inválida e você não a impede pelo tipo, devolva `Result` do `build`, e não um `unwrap` escondido.
- **Typestate para cada campo.** Com muitos obrigatórios, a explosão de estados não compensa. Use uma crate geradora ou valide no `build`.

## Resumo

- Rust não tem argumentos nomeados nem valores padrão; os substitutos são `Default` + `..Default::default()` e o builder.
- `Default` serve para structs de configuração com campos públicos e sem invariantes.
- Builders permitem campos privados, validação, obrigatórios no construtor e evolução da API sem quebra.
- Consumir `self` encadeia e move sem clonar; `&mut self` (como `Command`) facilita configuração condicional e reuso.
- `#[must_use]` avisa de um builder descartado.
- Builder + typestate torna um campo obrigatório verificável na compilação.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md) (a conexão usa typestate, e o desafio extra pede um builder).

## Leitura complementar

- [Rust API Guidelines: C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html#c-builder)
- [Rust Design Patterns: Builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html)
- Documentação de [`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html) e [`std::thread::Builder`](https://doc.rust-lang.org/std/thread/struct.Builder.html) (os dois estilos na std)
- [The Rust Reference: atributo `must_use`](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute)

---

[← Lição anterior](03-phantomdata-e-typestate.md) · [Índice do módulo](README.md) · [Próxima lição →](05-blanket-extension-e-sealed.md)
