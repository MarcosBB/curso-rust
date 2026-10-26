# 03 — `Rc`: posse compartilhada

**Objetivos.** Ao final, você vai saber:

- compartilhar a posse de um valor entre vários donos com `Rc<T>`;
- o que `Rc::clone` custa, e como acompanhar os contadores;
- por que o conteúdo de um `Rc` é só de leitura, e as saídas (`make_mut`, `try_unwrap`, e a próxima lição);
- como `Rc` se compara ao `shared_ptr` do C++ e aos objetos do Python, e por que ele não atravessa threads.

## Quando um dono não basta

Uma configuração é lida uma vez e usada por vários componentes, que vivem por tempos diferentes e não se conhecem. Quem é o dono? Com referências (`&Config`), alguém precisa viver mais que todos os outros, e o *lifetime* tem que provar isso. Às vezes não dá: os componentes são criados e destruídos em momentos que só se sabem em tempo de execução.

`Rc<T>` (*reference counted*) resolve com contagem de referências: o valor fica no heap junto com um contador de donos, cada `clone` soma 1, cada `drop` subtrai 1, e o valor é descartado quando o contador chega a zero.

```rust
use std::rc::Rc;

#[derive(Debug, Clone)]
struct Config {
    nome: String,
    nivel: u8,
}

struct Servidor {
    config: Rc<Config>,
}

struct Monitor {
    config: Rc<Config>,
}

let config = Rc::new(Config { nome: String::from("prod"), nivel: 2 });
println!("donos: {}", Rc::strong_count(&config));

let servidor = Servidor { config: Rc::clone(&config) };
{
    let monitor = Monitor { config: Rc::clone(&config) };
    println!("donos: {}", Rc::strong_count(&config));
    println!("mesmo valor? {}", Rc::ptr_eq(&servidor.config, &monitor.config));
} // `monitor` sai de escopo: o contador desce
println!("donos: {}", Rc::strong_count(&config));
println!("nível do servidor: {}", servidor.config.nivel);
```

```text
donos: 1
donos: 3
mesmo valor? true
donos: 2
nível do servidor: 2
```

Os três `Rc` apontam para **o mesmo** `Config` (`Rc::ptr_eq` compara os endereços). Nenhum deles é "o dono principal": quem sair por último apaga a luz. Código completo em [`codigo/rc-e-refcell`](codigo/rc-e-refcell/src/main.rs).

![Na stack, três Rc<Config> (config, servidor.config e monitor.config) e um Weak<Config> tracejado apontam para uma única alocação no heap, com strong = 3, weak = 1 e o Config. O Config é só de leitura. Quando strong chega a 0, o Config é descartado; quando strong e weak chegam a 0, a alocação é liberada.](assets/rc-contadores.svg)

Na alocação há **dois** contadores: `strong` (donos) e `weak` (observadores, os `Weak` da lição 05). O desenho já mostra um `Weak` para você saber que ele existe.

### `Rc::clone(&a)` em vez de `a.clone()`

As duas formas fazem a mesma coisa. A convenção do Rust Book é escrever `Rc::clone(&a)`, porque deixa claro, na leitura, que é um incremento de contador e não uma cópia profunda do `Config`. É a promessa do Módulo 02: o `clone` de um `Rc` não copia o dado, porque o `Rc` não é dono sozinho. O custo é somar 1 num inteiro.

## Só leitura

`Rc<T>` implementa `Deref` (lição 02), mas não `DerefMut`. Você recebe `&T`, nunca `&mut T`:

```rust
// ❌ NÃO COMPILA
use std::rc::Rc;

fn main() {
    let compartilhado = Rc::new(vec![1, 2, 3]);
    let outro = Rc::clone(&compartilhado);
    outro.push(4);
}
```

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
 --> src/main.rs:6:5
  |
6 |     outro.push(4);
  |     ^^^^^ cannot borrow as mutable
  |
  = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Vec<i32>>`
```

Não é falta de recurso, é a regra do Módulo 03 de volta: vários donos são várias referências compartilhadas, e alterar algo compartilhado precisa de exclusividade. Se `outro.push(4)` fosse permitido, `compartilhado` poderia estar percorrendo o mesmo `Vec` naquele momento, e a realocação deixaria o percurso com um ponteiro pendente.

Há três saídas:

1. **`Rc::make_mut`**: *clone-on-write*. Se só há um dono, devolve `&mut T` direto. Se há outros, **clona** o valor, aponta o seu `Rc` para a cópia e devolve `&mut` da cópia. Os outros donos não veem a mudança.
2. **`Rc::try_unwrap`**: se só há um dono, devolve o `T` de dentro (`Ok`); senão, devolve o próprio `Rc` de volta (`Err`).
3. **Mutabilidade interior** (`Rc<RefCell<T>>`): todos os donos veem a mudança. É a próxima lição.

```rust
let mut a = Rc::new(Config { nome: String::from("dev"), nivel: 1 });
let b = Rc::clone(&a);
// Há dois donos: make_mut clona o Config e `a` passa a apontar para a cópia.
Rc::make_mut(&mut a).nivel = 9;
println!("a: {} {}, b: {} {}", a.nome, a.nivel, b.nome, b.nivel);
println!("donos de b: {}", Rc::strong_count(&b));

// Com um dono só, try_unwrap devolve o valor de dentro.
let sozinho = Rc::try_unwrap(b).expect("só há um dono");
println!("de volta: {sozinho:?}");
```

```text
a: dev 9, b: dev 1
donos de b: 1
de volta: Config { nome: "dev", nivel: 1 }
```

`make_mut` exige `T: Clone`, e é por isso que `Config` deriva `Clone`.

## Comparando com C++ e Python

```cpp
auto config = std::make_shared<Config>(Config{"prod", 2});
auto outra = config;            // incremento atômico do contador
config->nivel = 3;              // permitido: shared_ptr<Config> não é const
std::cout << config.use_count();
```

```rust
let config = Rc::new(Config { nome: String::from("prod"), nivel: 2 });
let outra = Rc::clone(&config); // incremento não atômico
// config.nivel = 3;            // E0594: o conteúdo de um Rc não é mutável
println!("{}", Rc::strong_count(&config));
```

| | C++ `shared_ptr<T>` | Rust `Rc<T>` |
|---|---|---|
| Contador | atômico, sempre | comum (não atômico); o atômico é o `Arc` |
| Alterar o conteúdo | permitido (a menos que seja `shared_ptr<const T>`) | só com `make_mut` ou mutabilidade interior |
| Mandar para outra thread | permitido; a corrida no conteúdo é problema seu | erro de compilação (veja abaixo) |
| Nulo | pode ser | não; use `Option<Rc<T>>` |
| Ciclos | vazam; use `weak_ptr` | vazam; use `Weak` (lição 05) |

O `shared_ptr` paga um incremento atômico a cada cópia, mesmo num programa de uma thread. O Rust separa: `Rc` para uma thread, mais barato, e `Arc` (Módulo 11) quando precisa atravessar threads. Usar o errado não é uma questão de desempenho, e sim um erro de compilação.

Em Python, **todo** objeto já funciona como um `Rc`: cada variável é uma referência, e o CPython mantém um contador por objeto (`sys.getrefcount` mostra o valor). Por isso `b = a` numa lista faz as duas variáveis verem a mesma lista. A diferença é que, em Rust, a contagem é opcional e explícita: você escreve `Rc` só onde precisa, e o resto do programa não paga contador nenhum. E o Python tem um coletor de ciclos por cima da contagem, que o Rust não tem (lição 05).

## `Rc` não é `Send`: uma prévia do Módulo 11

O contador de um `Rc` é um inteiro comum. Se duas threads fizessem `clone` ao mesmo tempo, os dois incrementos poderiam se atropelar e o contador ficaria errado, levando a um *use-after-free*. O compilador impede:

```rust
// ❌ NÃO COMPILA
use std::rc::Rc;
use std::thread;

fn main() {
    let dados = Rc::new(vec![1, 2, 3]);
    let copia = Rc::clone(&dados);
    let h = thread::spawn(move || println!("{copia:?}"));
    h.join().unwrap();
}
```

```text
error[E0277]: `Rc<Vec<i32>>` cannot be sent between threads safely
   --> src/main.rs:7:27
    |
  7 |     let h = thread::spawn(move || println!("{copia:?}"));
    |             ------------- -------^^^^^^^^^^^^^^^^^^^^^^
    |             |             |
    |             |             `Rc<Vec<i32>>` cannot be sent between threads safely
    |             |             within this `{closure@src/main.rs:7:27: 7:34}`
    |             required by a bound introduced by this call
    |
    = help: within `{closure@src/main.rs:7:27: 7:34}`, the trait `Send` is not implemented for `Rc<Vec<i32>>`
...
```

`Send` é a trait marcadora de "pode ser enviado para outra thread", e o `Rc` não a implementa. A correção é trocar por `Arc`, com o mesmo jeito de usar e contador atômico. O [Módulo 11](../11-concorrencia/README.md) explica `Send`, `Sync` e `Arc` em detalhe. Por ora, guarde a regra: **`Rc` para uma thread, `Arc` para várias**.

## Armadilhas comuns

**`Rc` como saída para todo erro do borrow checker.** O fluxograma do [Módulo 03](../03-borrowing-e-lifetimes/07-brigando-menos-com-o-borrow-checker.md) deixou `Rc`/`RefCell` por último, e continua valendo. Antes, pergunte se um dono só não basta (uma struct que guarda os dados, e índices em vez de ponteiros), ou se uma referência com lifetime resolve. `Rc` espalhado pelo código esconde quem é responsável por quê.

**Esperar que todos vejam a mudança feita com `make_mut`.** Com mais de um dono, `make_mut` clona: a alteração fica só no seu `Rc`. Se a intenção é que todos vejam, o tipo certo é `Rc<RefCell<T>>`.

**`.clone()` num `Rc<T>` achando que copia o `T`.** Não copia. Se você quer uma cópia independente do conteúdo, é `(*rc).clone()` ou `T::clone(&rc)`.

**`Rc<T>` onde `&T` basta.** Uma função que só lê deve pedir `&Config`, e não `Rc<Config>`. Quem tem um `Rc<Config>` passa `&config`, e a *deref coercion* faz o resto. Peça `Rc` só se a função vai **guardar** um dono.

## Resumo

- `Rc<T>` dá vários donos para um valor no heap. `Rc::clone` soma 1 no contador, cada `drop` subtrai 1, e o valor some quando o contador chega a zero.
- O conteúdo é só de leitura (E0596, não há `DerefMut`). Saídas: `make_mut` (clona se houver outros donos), `try_unwrap` (se você é o único) ou `RefCell` (próxima lição).
- `Rc` é o `shared_ptr` do C++ com contador não atômico e sem alteração livre do conteúdo. Em Python, todo objeto já é contado.
- `Rc` não é `Send`: usá-lo numa thread é erro de compilação. O equivalente para threads é o `Arc` (Módulo 11).
- Peça `&T` nas funções que só leem; `Rc` é para quem guarda.

## Exercícios

Veja os exercícios 3 e 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 15.4 `Rc<T>`, the Reference Counted Smart Pointer](https://doc.rust-lang.org/stable/book/ch15-04-rc.html)
- [Documentação de `std::rc`](https://doc.rust-lang.org/std/rc/index.html) (`make_mut`, `try_unwrap`, `ptr_eq`)
- [Comprehensive Rust: Rc](https://google.github.io/comprehensive-rust/smart-pointers/rc.html)

---

[← Lição anterior](02-deref-e-drop.md) · [Índice do módulo](README.md) · [Próxima lição →](04-cell-e-refcell.md)
