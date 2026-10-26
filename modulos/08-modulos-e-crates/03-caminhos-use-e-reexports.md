# 03 — Caminhos, `use` e re-exports

**Objetivos.** Ao final, você vai saber:

- escolher entre caminho absoluto e relativo (`crate`, `self`, `super`);
- por que um `use` só vale no módulo em que foi escrito;
- os idiomas do `use`: o que importar, `as`, chaves e globs;
- separar a estrutura interna da API pública com `pub use`.

## Caminhos absolutos e relativos

Todo item tem um caminho na árvore de módulos. Ele pode começar de quatro jeitos:

| Começo | A partir de | Exemplo |
|---|---|---|
| `crate::` | a raiz da crate atual | `crate::rede::tcp::conectar` |
| nome de uma crate | a raiz de uma dependência (ou `std`) | `std::collections::HashMap`, `chat::Conexao` |
| `self::` ou um nome local | o módulo atual | `tcp::conectar` (dentro de `rede`) |
| `super::` | o módulo pai | `super::host_valido` (dentro de `rede::tcp`) |

O Rust Book recomenda caminhos absolutos (`crate::...`) por padrão: quando você move código, a definição e o uso costumam se mover separadamente, e o caminho absoluto continua certo. `super::` é bom para o que é íntimo do pai, como em `rede::tcp`, e nos testes.

Desde a edition 2018 não existe mais `extern crate`: toda dependência do `Cargo.toml` já está disponível como raiz de caminho.

## O `use` é local ao módulo

O `use` cria um nome **dentro do módulo onde está escrito**. Um `use` na raiz não vale num submódulo, mesmo que ele esteja no mesmo arquivo:

```rust
// ❌ NÃO COMPILA
mod rede {
    pub mod tcp {
        pub fn conectar() {}
    }
}

use rede::tcp;

mod cliente {
    pub fn iniciar() {
        tcp::conectar(); // ❌ o `use` da raiz não vale aqui
    }
}

fn main() {
    tcp::conectar();
    cliente::iniciar();
}
```

```text
error[E0433]: cannot find module or crate `tcp` in this scope
  --> src/main.rs:11:9
   |
11 |         tcp::conectar(); // ❌ o `use` da raiz não vale aqui
   |         ^^^ use of unresolved module or unlinked crate `tcp`
   |
   = help: if you wanted to use a crate named `tcp`, use `cargo add tcp` to add it to your `Cargo.toml`
help: consider importing this module through its public re-export
   |
10 +     use crate::tcp;
   |
```

Duas coisas nessa mensagem. O primeiro `help` (`cargo add tcp`) não ajuda: o compilador não sabe se você queria um módulo ou uma crate externa, e cobre os dois casos. O segundo mostra algo útil: o `use rede::tcp;` da raiz criou um nome `crate::tcp`, e um submódulo pode usá-lo como qualquer item privado do ancestral. As correções são repetir o `use` dentro de `cliente` (`use crate::rede::tcp;`) ou escrever o caminho (`crate::rede::tcp::conectar()`).

Em Python, cada arquivo tem os próprios imports, e isso é a mesma ideia. A diferença é que, em Rust, a unidade é o módulo, não o arquivo: um `mod` inline também começa "vazio".

## Idiomas do `use`

O Rust Book sugere uma convenção que quase todo projeto segue:

- **Funções:** importe o módulo pai e chame com o prefixo. `tcp::conectar(...)` deixa claro que a função não é local.
- **Tipos, traits e macros:** importe o item. `use std::collections::HashMap;` e depois `HashMap::new()`.

Quando dois tipos têm o mesmo nome, importar os dois é erro:

```rust
// ❌ NÃO COMPILA
use std::fmt::Result;
use std::io::Result;
```

```text
error[E0252]: the name `Result` is defined multiple times
 --> src/main.rs:2:5
  |
1 | use std::fmt::Result;
  |     ---------------- previous import of the type `Result` here
2 | use std::io::Result;
  |     ^^^^^^^^^^^^^^^ `Result` reimported here
  |
  = note: `Result` must be defined only once in the type namespace of this module
help: you can use `as` to change the binding name of the import
  |
2 | use std::io::Result as OtherResult;
  |                     ++++++++++++++
```

Duas saídas: importar os módulos (`use std::{fmt, io};` e escrever `fmt::Result`, `io::Result`) ou renomear com `as` (`use std::io::Result as IoResult;`).

Chaves agrupam imports com o mesmo começo, e `self` dentro delas importa o próprio módulo. O `cliente.rs` do [`chat`](codigo/chat/src/cliente.rs) faz isso:

```rust
use crate::rede::tcp::{self, Conexao}; // `tcp` e `tcp::Conexao`
```

Não se preocupe com a ordem: o `cargo fmt` ordena e agrupa os `use`.

### Globs (`*`)

`use modulo::*;` importa tudo o que é público. É aceitável em três situações: `use super::*;` nos testes, preludes (abaixo) e módulos da sua própria crate. De crates de terceiros, evite: se uma versão nova acrescentar um nome que já existe no seu código, o seu código para de compilar sem você ter mudado nada.

Dois globs com o mesmo nome só dão erro **quando o nome é usado**:

```rust
// ❌ NÃO COMPILA (a e b têm, cada um, uma fn formatar)
use a::*;
use b::*;

fn main() {
    println!("{}", formatar());
}
```

```text
error[E0659]: `formatar` is ambiguous
  --> src/main.rs:17:20
   |
17 |     println!("{}", formatar());
   |                    ^^^^^^^^ ambiguous name
   |
   = note: ambiguous because of multiple glob imports of a name in the same module
note: `formatar` could refer to the function imported here
  --> src/main.rs:13:5
   |
13 | use a::*;
   |     ^^^^
   = help: consider adding an explicit import of `formatar` to disambiguate
...
```

Um import explícito (`use a::formatar;`) tem prioridade sobre os globs e resolve. Comparação: é o mesmo problema de `using namespace std;` no C++ e de `from x import *` no Python, só que detectado na compilação.

## `pub use`: a API não precisa ser a estrutura interna

`pub use` reexporta: cria um nome público que aponta para um item de outro lugar. Com ele, a organização dos arquivos fica livre para mudar sem quebrar quem usa a biblioteca.

O `chat` deixa os módulos públicos, mas oferece um atalho para o tipo mais usado:

```rust
// src/lib.rs
pub mod cliente;
pub mod rede;

pub use rede::tcp::Conexao;
```

Agora `chat::Conexao` e `chat::rede::tcp::Conexao` são o mesmo tipo, e o `main.rs` usa o curto:

```rust
use chat::rede::udp;
use chat::{Conexao, cliente};
```

Um passo além é o padrão **fachada**: módulos privados e `pub use` só do que forma a API. O `lib.rs` do [Projeto 2](../../projetos/02-csvlite/README.md) segue esse padrão (trecho):

```rust
mod error;
mod reader;
mod record;

pub use error::{Error, ErrorKind};
pub use reader::{Options, Reader};
pub use record::Record;
```

Quem usa a crate `csvlite` vê `csvlite::Reader` e `csvlite::Error`, e a página do `cargo doc` mostra tudo na raiz. Os arquivos internos podem ser reorganizados à vontade.

> [!WARNING]
> `pub mod` não reexporta nada: ele só torna o módulo visível. Reexportar é `pub use`. Alguns materiais confundem os dois.

O equivalente em Python é o `__init__.py` com `from .leitor import Leitor` e um `__all__`. Em C++, um header "guarda-chuva" que inclui os outros, ou `using` num namespace.

Só dá para reexportar o que está visível no nível desejado:

```rust
// ❌ NÃO COMPILA
mod leitor {
    pub struct Leitor;

    fn interno() {}

    pub use self::interno as exposto; // ❌ reexportar um item privado
}

pub use leitor::Leitor; // ✅ módulo privado, tipo público reexportado
```

```text
error[E0364]: `interno` is private, and cannot be re-exported
 --> src/lib.rs:6:13
  |
6 |     pub use self::interno as exposto; // ❌ reexportar um item privado
  |             ^^^^^^^^^^^^^^^^^^^^^^^^
  |
note: consider marking `interno` as `pub` in the imported module
...
```

Um re-export também pode apontar para outra crate (`pub use moeda;`). A lição 05 mostra quando isso é necessário.

### Preludes

Você nunca escreve `use std::vec::Vec;` porque o compilador importa automaticamente o **prelude** da std (`Vec`, `String`, `Option`, `Result`, `Box`, as traits mais comuns). Algumas crates oferecem o próprio: um módulo `prelude` com `pub use` do que quase todo usuário precisa, para ser importado com `use minha_crate::prelude::*;`. É um dos usos aceitáveis de glob.

## Armadilhas comuns

- **`use` na raiz e uso dentro de um `mod`** (E0433). Repita o `use` no módulo.
- **Dois `use` com o mesmo nome** (E0252). Importe o módulo ou use `as`.
- **Glob de crate externa.** Uma versão nova pode criar um conflito (E0659) no seu código.
- **`pub mod` achando que reexporta.** Para encurtar o caminho, é `pub use`.
- **Reexportar item privado** (E0364).

## Resumo

- Caminhos começam em `crate::`, no nome de uma crate, no módulo atual ou em `super::`. Prefira o absoluto.
- O `use` vale só no módulo onde está escrito.
- Funções pelo módulo pai, tipos pelo nome; conflitos com `as` ou pelo módulo.
- Globs só em testes, preludes e módulos seus.
- `pub use` separa a API da estrutura interna (padrão fachada).

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 7.3 Paths](https://doc.rust-lang.org/stable/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html) e [7.4 `use`](https://doc.rust-lang.org/stable/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html)
- [The Rust Reference: Use declarations](https://doc.rust-lang.org/reference/items/use-declarations.html)
- [Effective Rust, Item 23: Avoid wildcard imports](https://effective-rust.com/wildcard.html)

---

[← Lição anterior](02-visibilidade.md) · [Índice do módulo](README.md) · [Próxima lição →](04-pacotes-crates-e-alvos.md)
