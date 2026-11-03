# 06 — Estado global seguro: `static`, `LazyLock` e `OnceLock`

**Objetivos.** Ao final, você vai saber:

- por que todo `static` precisa ser `Sync`, e o que isso tem a ver com threads;
- declarar globais com inicialização `const` (atômicos, `Mutex`) e com inicialização preguiçosa (`LazyLock`);
- definir um valor global uma vez, em tempo de execução, com `OnceLock`;
- por que `static mut` é uma má ideia, e o que mudou na edition 2024.

## `static` x `const`

O Módulo 01 apresentou os dois de passagem. A diferença importa agora:

- **`const`** é um valor copiado em cada lugar onde é usado. Não tem endereço fixo, e cada uso é independente.
- **`static`** é **uma** variável, com um endereço fixo, que existe o programa todo. Todo mundo que usa o `static` usa a mesma.

"Todo mundo" inclui todas as threads. Um `static` é, por definição, compartilhado entre threads, e por isso o compilador exige que o tipo seja `Sync` (lição 02):

```rust
// ❌ NÃO COMPILA
use std::cell::Cell;

static VISITAS: Cell<u32> = Cell::new(0);

fn main() {
    VISITAS.set(VISITAS.get() + 1);
}
```

```text
error[E0277]: `Cell<u32>` cannot be shared between threads safely
 --> src/main.rs:3:17
  |
3 | static VISITAS: Cell<u32> = Cell::new(0);
  |                 ^^^^^^^^^ `Cell<u32>` cannot be shared between threads safely
  |
  = help: the trait `Sync` is not implemented for `Cell<u32>`
  = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` or `std::sync::atomic::AtomicU32` instead
  = note: shared static variables must have a type that implements `Sync`
```

Um `static` comum é imutável. Para alterar um global, você usa um tipo com mutabilidade interior que seja `Sync`: exatamente as ferramentas das lições 04 e 05.

## Inicialização `const`: atômicos e `Mutex`

O valor inicial de um `static` é calculado na compilação. Só pode usar `const fn`. Os construtores dos atômicos, `Mutex::new`, `RwLock::new`, `Vec::new` e `String::new` são `const fn`, então estes funcionam direto:

```rust
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

static PROXIMO_ID: AtomicU64 = AtomicU64::new(1);
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn novo_id() -> u64 {
    PROXIMO_ID.fetch_add(1, Ordering::Relaxed)
}

fn registrar(msg: &str) {
    LOG.lock().unwrap().push(msg.to_string());
}
```

Nada de `Arc`: o `static` vive o programa todo, então qualquer thread pode usar `&PROXIMO_ID` (é `&'static`). Três threads pegando ids, duas execuções:

```text
thread 0 pegou o id 1        thread 0 pegou o id 2
thread 1 pegou o id 2        thread 1 pegou o id 1
thread 2 pegou o id 3        thread 2 pegou o id 3
```

Quem pega qual id depende de quem chega primeiro. Que os ids nunca se repetem, não.

## Inicialização em tempo de execução: `LazyLock`

E um `HashMap`, ou um vetor com valores? Eles precisam alocar, e alocação não acontece na compilação:

```rust
// ❌ NÃO COMPILA
static NOMES: Vec<String> = vec![String::from("Ana")];
```

```text
error[E0015]: cannot call non-const associated function `<String as From<&str>>::from` in statics
 --> src/main.rs:1:34
  |
1 | static NOMES: Vec<String> = vec![String::from("Ana")];
  |                                  ^^^^^^^^^^^^^^^^^^^
  |
  = note: calls in statics are limited to constant functions, tuple structs and tuple variants
  = note: consider wrapping this expression in `std::sync::LazyLock::new(|| ...)`
...
```

O compilador já diz a solução. `LazyLock<T>` guarda uma closure e só a executa no **primeiro acesso**. Se várias threads chegarem juntas, só uma roda a closure; as outras esperam e recebem o mesmo valor.

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static UFS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    println!("  (montando a tabela de UFs)");
    HashMap::from([
        ("SP", "São Paulo"),
        ("RJ", "Rio de Janeiro"),
        ("MG", "Minas Gerais"),
    ])
});

fn nome_da_uf(sigla: &str) -> Option<&'static str> {
    UFS.get(sigla).copied()
}

fn main() {
    println!("antes de usar a tabela");
    println!("SP = {:?}", nome_da_uf("SP"));
    println!("BA = {:?}", nome_da_uf("BA"));
}
```

```text
antes de usar a tabela
  (montando a tabela de UFs)
SP = Some("São Paulo")
BA = None
```

`LazyLock` implementa `Deref`, então `UFS.get(...)` chama o método do `HashMap` direto, como acontecia com o `Box` (Módulo 10). Ele é o `LazyCell` do Módulo 10 com sincronização, e substitui a macro `lazy_static!` e a crate `once_cell`, que você ainda vai encontrar em código antigo:

```rust
// Jeito antigo (crate lazy_static), que não é mais necessário:
lazy_static! {
    static ref UFS: HashMap<&'static str, &'static str> = { ... };
}
```

## Definir uma vez, depois: `OnceLock`

`LazyLock` sabe como calcular o valor desde o começo. Às vezes o valor só é conhecido depois: uma configuração lida dos argumentos da linha de comando, por exemplo. `OnceLock<T>` é uma célula que começa vazia e pode ser escrita **uma vez**:

```rust
use std::sync::OnceLock;

#[derive(Debug)]
struct Config {
    verboso: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config { verboso: false })
}

fn main() {
    let verboso = std::env::args().any(|a| a == "-v");
    CONFIG.set(Config { verboso }).unwrap();
    println!("config: {:?}", config());
    println!("segundo set: {:?}", CONFIG.set(Config { verboso: true }).is_err());
}
```

```text
config: Config { verboso: false }
segundo set: true
```

- `set` grava se estiver vazio e devolve `Err(valor)` se já tinha algo: ninguém troca a configuração por acidente depois.
- `get` devolve `Option<&T>`.
- `get_or_init` devolve o valor, inicializando com a closure se ainda estiver vazio. Aqui ele dá um valor padrão para quando ninguém chamou `set` (nos testes, por exemplo).

Como o valor nunca muda depois de escrito, a leitura devolve um `&'static Config` comum, sem guarda e sem lock. Código completo em [`codigo/estado-global`](codigo/estado-global/src/main.rs).

## Por que não `static mut`

Rust tem `static mut`, uma variável global mutável sem nenhuma proteção, como uma global em C. Toda leitura ou escrita exige `unsafe`, porque o compilador não tem como saber se outra thread está mexendo nela ao mesmo tempo. Uma data race num `static mut` é UB, exatamente como em C++.

Na edition 2024, criar uma **referência** para um `static mut` passou a ser erro. E muita coisa cria uma referência sem parecer, como o `println!`:

```rust
// ❌ NÃO COMPILA
static mut CONTADOR: u32 = 0;

fn main() {
    unsafe {
        CONTADOR += 1;
        println!("{}", CONTADOR);
    }
}
```

```text
error: creating a shared reference to mutable static
 --> src/main.rs:6:24
  |
6 |         println!("{}", CONTADOR);
  |                        ^^^^^^^^ shared reference to mutable static
  |
  = note: shared references to mutable statics are dangerous; it's undefined behavior if the static is mutated or if a mutable reference is created for it while the shared reference lives
  = help: use a type that relies on "interior mutability" instead; to read more on this, visit <https://doc.rust-lang.org/reference/interior-mutability.html>
  = note: for more information, see <https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html>
  = note: `#[deny(static_mut_refs)]` (part of `#[deny(rust_2024_compatibility)]`) on by default
```

O `help` aponta o caminho: mutabilidade interior. O contador vira um `static CONTADOR: AtomicU32`, sem `unsafe` e sem UB. Para o raro caso legítimo (FFI com C, por exemplo), existem os ponteiros crus `&raw const`/`&raw mut`, que o Módulo 15 apresenta. No código de aplicação, a resposta é: **não use `static mut`**.

| Necessidade | Use |
|---|---|
| constante | `const` |
| global imutável com valor `const` | `static` |
| contador, id, flag global | `static X: AtomicU64` |
| global mutável maior | `static X: Mutex<T>` (com `Mutex::new` `const`) |
| global calculado no primeiro uso | `static X: LazyLock<T>` |
| global definido uma vez, em tempo de execução | `static X: OnceLock<T>` |
| global mutável sem proteção | não existe em Rust seguro (`static mut` exige `unsafe`) |

> [!NOTE]
> Existe ainda `thread_local!`, que cria uma variável com uma cópia **por thread**. Como cada thread só vê a sua, ela aceita tipos não-`Sync` como `Cell` e `RefCell`. Serve para caches por thread e para geradores de números aleatórios, por exemplo. Veja a documentação de [`thread_local!`](https://doc.rust-lang.org/std/macro.thread_local.html).

## E em C++ e Python?

Em C++, uma variável `static` local a uma função é inicializada uma vez, de forma segura entre threads, desde o C++11 (é a ideia do `LazyLock`). Globais mutáveis comuns não têm proteção nenhuma: protegê-las com um mutex é por conta do programador.

Em Python, globais de módulo são compartilhados por todas as threads, e o padrão para inicializar uma vez é `functools.cache` numa função, ou um `if _config is None:` protegido por um `threading.Lock`. Nada obriga a usar o lock.

## Armadilhas comuns

- **Estado global como primeira opção.** Globais tornam testes mais difíceis (os testes do `cargo test` rodam em paralelo, no mesmo processo, e compartilham os `static`). Prefira passar um `&Config` ou um `Arc<Config>` para quem precisa. Use `static` para o que é realmente global: tabelas fixas, contadores de diagnóstico, configuração do processo.
- **`OnceLock::set` ignorado.** `set` devolve `Result`. Se você ignorar o `Err`, uma segunda configuração some sem aviso.
- **Inicialização pesada no `LazyLock`.** Ela roda no primeiro acesso, no meio do código de quem acessou. Se ela pode falhar ou demorar, considere inicializar explicitamente no começo do `main`.
- **Copiar exemplos com `lazy_static!` ou `static mut`.** São o jeito antigo. `LazyLock` (1.80), `OnceLock` (1.70) e os atômicos cobrem os mesmos casos na std.

## Resumo

- Um `static` é compartilhado entre threads, por isso o tipo precisa ser `Sync`. `Cell`/`RefCell` não servem.
- Inicialização `const`: atômicos, `Mutex::new(Vec::new())` e afins vão direto num `static`.
- `LazyLock<T>`: calcula no primeiro acesso, uma vez só, mesmo com várias threads. Substitui `lazy_static`/`once_cell`.
- `OnceLock<T>`: escrito uma vez em tempo de execução (`set`, `get`, `get_or_init`).
- `static mut`: referências são erro na edition 2024. Use mutabilidade interior.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html) e de [`OnceLock`](https://doc.rust-lang.org/std/sync/struct.OnceLock.html)
- [The Rust Reference: Static items](https://doc.rust-lang.org/reference/items/static-items.html)
- [Edition Guide: Disallow references to `static mut`](https://doc.rust-lang.org/edition-guide/rust-2024/static-mut-references.html)

---

[← Lição anterior](05-atomicos.md) · [Índice do módulo](README.md) · [Próxima lição →](07-rayon.md)
