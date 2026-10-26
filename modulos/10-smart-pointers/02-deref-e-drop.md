# 02 — `Deref` e `Drop` próprios

**Objetivos.** Ao final, você vai saber:

- implementar `Deref` e `DerefMut` para que um tipo seu funcione com `*` e com a *deref coercion*;
- por que implementar `Deref` num newtype para "herdar" métodos é um antipadrão;
- escrever um `impl Drop` próprio, e reconhecer quando ele vale a pena;
- por que não se chama `drop` à mão (E0040).

Um ponteiro inteligente é feito de duas traits: `Deref`, que o faz se comportar como uma referência, e `Drop`, que faz a limpeza. No Módulo 06 você só **usou** a *deref coercion*; no Módulo 02, só **observou** o `Drop` com o `Barulhento`. Agora vamos escrever as duas.

## Um `MeuBox<T>` de brinquedo

Uma tupla com um campo, que guarda o valor (na stack mesmo; o que interessa aqui é o comportamento, não o heap):

```rust
// ❌ NÃO COMPILA
struct MeuBox<T>(T);

impl<T> MeuBox<T> {
    fn new(x: T) -> MeuBox<T> {
        MeuBox(x)
    }
}

fn main() {
    let y = MeuBox::new(5);
    assert_eq!(5, *y);
}
```

```text
error[E0614]: type `MeuBox<{integer}>` cannot be dereferenced
  --> src/main.rs:11:19
   |
11 |     assert_eq!(5, *y);
   |                   ^^ can't be dereferenced
```

O operador `*` só funciona em referências e em tipos que implementam `Deref`. A trait tem um tipo associado (o `Target`, para onde o ponteiro aponta) e um método:

```rust
// ✅
use std::ops::Deref;

impl<T> Deref for MeuBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}
```

Agora `*y` compila. O compilador reescreve `*y` como `*(y.deref())`: `deref` devolve uma referência comum, e o `*` de fora a segue. Por que não devolver o valor direto? Porque isso moveria o `T` para fora do `MeuBox`, e quem só tem um `&MeuBox` não pode mover nada.

Para alterar através do ponteiro (`*y = 6`, ou chamar um método `&mut self` do conteúdo), implemente também `DerefMut`, que reaproveita o `Target`:

```rust
use std::ops::DerefMut;

impl<T> DerefMut for MeuBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
```

## A *deref coercion*, agora por dentro

Com `Deref` implementado, `MeuBox<String>` entra na mesma cadeia de conversões que você viu no Módulo 06 para `&String` → `&str`:

```rust
fn saudar(nome: &str) {
    println!("olá, {nome}");
}

let mut nome = MeuBox::new(String::from("Ana"));
saudar(&nome);           // &MeuBox<String> -> &String -> &str
nome.push_str(" Maria"); // DerefMut: chama String::push_str
println!("tamanho: {}", nome.len());
```

```text
olá, Ana
tamanho: 9
```

Quando o tipo do argumento não bate, o compilador tenta aplicar `deref` quantas vezes forem necessárias: `MeuBox<String>` vira `String`, que vira `str`. Na chamada de método acontece o mesmo (*auto-deref*): `nome.len()` procura `len` em `MeuBox`, não acha, e tenta em `String`. Tudo isso é resolvido na compilação, sem custo em tempo de execução.

As regras para `&mut`:

- `&T` → `&U` quando `T: Deref<Target = U>`;
- `&mut T` → `&mut U` quando `T: DerefMut<Target = U>`;
- `&mut T` → `&U` quando `T: Deref<Target = U>` (perder a mutabilidade é seguro);
- `&T` → `&mut U` **nunca**: transformar um empréstimo compartilhado em exclusivo quebraria a regra do Módulo 03.

Código completo em [`codigo/meu-box`](codigo/meu-box/src/main.rs).

## `Deref` não é herança

Esta é a promessa do Módulo 06. A tentação: um newtype `Nomes(Vec<String>)` que implementa `Deref<Target = Vec<String>>` para ganhar todos os métodos do `Vec` de graça. Parece funcionar:

```rust
let nomes = Nomes(vec![String::from("Ana"), String::from("Bia")]);
println!("{} nomes, primeiro {}", nomes.len(), nomes[0]); // ok
```

Mas o `Deref` só atua na chamada de método e na coerção de referências. **Traits não atravessam o `Deref`**: `Nomes` não implementa `IntoIterator`, `Debug`, `PartialEq` nem nada que o `Vec` implemente.

```rust
// ❌ NÃO COMPILA
for n in &nomes {
    println!("{n}");
}
```

```text
error[E0277]: `&Nomes` is not an iterator
  --> src/main.rs:15:14
   |
15 |     for n in &nomes {
   |              ^^^^^^ `&Nomes` is not an iterator
   |
   = help: the trait `Iterator` is not implemented for `&Nomes`
   = note: required for `&Nomes` to implement `IntoIterator`
```

Funções genéricas (`fn f<T: AsRef<[String]>>(x: T)`) também não veem o `Vec` dentro. O resultado é um tipo que às vezes se passa pelo de dentro e às vezes não, o que confunde quem lê. E o newtype existe justamente para **restringir** o que se pode fazer com o valor (Módulo 04); expor o `Vec` inteiro desfaz isso.

A [documentação de `Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html#when-to-implement-deref-or-derefmut) dá o critério: implemente `Deref` só quando o tipo é um ponteiro inteligente, ou seja, quando "ser um acesso ao `Target`" é o propósito dele (`Box`, `Rc`, `String` → `str`, `Vec` → `[T]`, os guardas de `RefCell` e `Mutex`). Para um newtype, escreva os métodos que fazem sentido, delegando ao campo, ou implemente `AsRef<[String]>`.

## `Drop` próprio

A trait tem um método, chamado automaticamente quando o valor sai de escopo. Para o `MeuBox` avisar quando é descartado, mostrando o conteúdo:

```rust
// ❌ NÃO COMPILA
use std::fmt::Debug;

struct MeuBox<T>(T);

impl<T: Debug> Drop for MeuBox<T> {
    fn drop(&mut self) {
        println!("drop MeuBox({:?})", self.0);
    }
}
```

```text
error[E0367]: `Drop` impl requires `T: Debug` but the struct it is implemented for does not
 --> src/main.rs:5:9
  |
5 | impl<T: Debug> Drop for MeuBox<T> {
  |         ^^^^^
  |
note: the implementor must specify the same requirement
 --> src/main.rs:3:1
  |
3 | struct MeuBox<T>(T);
  | ^^^^^^^^^^^^^^^^
```

Esta é uma regra própria do `Drop`: ele não pode ser condicional. Com outras traits, `impl<T: Debug> Display for MeuBox<T>` seria normal (o `impl` condicional do Módulo 06). Mas todo `MeuBox` precisa saber se descartar, então o `Drop` vale para todos os `T` que a struct aceita, ou para nenhum. A correção é pôr o bound na própria struct, como faz o [projeto](codigo/meu-box/src/main.rs): `struct MeuBox<T: Debug>(T);`, repetido nos outros `impl`.

O `drop` roda **antes** dos campos serem descartados (eles ainda estão disponíveis em `&mut self`). Depois que ele termina, o compilador descarta cada campo, na ordem de declaração. Você nunca libera os campos à mão: uma `String` dentro da struct se libera sozinha.

### Quando vale a pena

Quase nunca para memória: `Vec`, `String` e `Box` já se liberam. `Drop` próprio vale para **recursos** que o compilador não conhece e para **guardas** (*guards*): objetos cujo trabalho é garantir que algo aconteça no fim do escopo, saia a função por onde sair. É o RAII do C++ usado de propósito.

O exemplo do projeto é uma transação que desfaz tudo se ninguém confirmar:

```rust
struct Transacao<'a> {
    log: &'a mut Vec<String>,
    confirmada: bool,
}

impl Transacao<'_> {
    // iniciar() grava "begin"; executar() grava o comando

    /// Consome a transação: depois de confirmar, não dá mais para usá-la.
    fn confirmar(mut self) {
        self.log.push(String::from("commit"));
        self.confirmada = true;
    } // `self` é descartado aqui, e o `drop` vê `confirmada == true`
}

impl Drop for Transacao<'_> {
    fn drop(&mut self) {
        if !self.confirmada {
            self.log.push(String::from("rollback"));
        }
    }
}

fn transferir(log: &mut Vec<String>, valor: u32) -> Result<(), String> {
    let mut t = Transacao::iniciar(log);
    t.executar(&format!("debita {valor}"));
    if valor > 100 {
        return Err(format!("limite excedido: {valor}")); // `t` cai aqui: rollback
    }
    t.executar(&format!("credita {valor}"));
    t.confirmar();
    Ok(())
}
```

```text
Err("limite excedido: 500")
["begin", "debita 50", "credita 50", "commit", "begin", "debita 500", "rollback"]
```

O `return` antecipado, um `?` ou até um *panic* descartam `t`, e o rollback acontece. Você não precisa lembrar de chamá-lo em cada saída da função. Repare também no `confirmar(mut self)`: ele **consome** a transação, então usar `t` depois de confirmar é erro de compilação (E0382). O tipo garante a ordem certa das operações.

Na std, o mesmo padrão aparece no `MutexGuard` (destrava no drop, Módulo 11), no `File` (fecha) e nos guardas `Ref`/`RefMut` do `RefCell` (lição 04).

```python
# Python: o mesmo efeito com um gerenciador de contexto
with transacao(log) as t:
    t.executar("debita 50")
    ...  # __exit__ faz rollback se houver exceção
```

A diferença: o `with` precisa ser lembrado por quem usa; o `Drop` é automático para todo valor do tipo.

### Não se chama `drop` à mão

```rust
// ❌ NÃO COMPILA
struct Arquivo(&'static str);

impl Drop for Arquivo {
    fn drop(&mut self) {
        println!("fechando {}", self.0);
    }
}

fn main() {
    let a = Arquivo("dados.txt");
    a.drop();
}
```

```text
error[E0040]: explicit use of destructor method
  --> src/main.rs:11:7
   |
11 |     a.drop();
   |       ^^^^ explicit destructor calls not allowed
   |
help: consider using `drop` function
   |
11 -     a.drop();
11 +     drop(a);
```

Se fosse permitido, o compilador ainda chamaria `drop` de novo no fim do escopo: um *double free*. A função `std::mem::drop(a)` (do *prelude*, por isso só `drop(a)`) resolve: ela **recebe a posse** de `a`, e o valor é descartado no fim dela. O corpo de `mem::drop` é vazio, literalmente `pub fn drop<T>(_x: T) {}`.

## Armadilhas comuns

**`Drop` e `Copy` no mesmo tipo** (E0184, visto no Módulo 02). Se um tipo tem limpeza, copiá-lo bit a bit daria duas limpezas.

**Contar com o `Drop` para algo obrigatório.** O Rust garante que o `drop` não roda duas vezes, mas não garante que ele rode: `std::mem::forget` e os ciclos de `Rc` (lição 05) pulam o descarte, sem `unsafe`. Um `Drop` serve para limpeza, e não para manter uma invariante de segurança.

**`Drop` com bound que a struct não tem** (E0367), visto acima: o bound vai na struct.

**Erro dentro do `drop`.** `drop` não devolve `Result`, e um *panic* durante um *panic* aborta o processo. Se a limpeza pode falhar (gravar um buffer em disco, por exemplo), ofereça também um método explícito que devolve `Result`, como o `File::sync_all` e o `BufWriter::into_inner`, e deixe o `Drop` como rede de segurança.

**`Deref` para imitar herança**, visto acima.

## Resumo

- `Deref` (com `type Target`) faz `*x` funcionar e liga o tipo à *deref coercion*: `&MeuBox<String>` vira `&str` sozinho. `DerefMut` faz o mesmo para `&mut`.
- `&mut T` vira `&U`, mas `&T` nunca vira `&mut U`.
- Implemente `Deref` só em ponteiros inteligentes. Num newtype, traits não atravessam o `Deref` (o `for` dá E0277); delegue os métodos à mão.
- `Drop` próprio é para recursos e guardas (RAII). Os campos se liberam sozinhos depois do seu `drop`.
- Não se chama `x.drop()` (E0040); use `drop(x)`, que toma a posse.

## Exercícios

Veja o exercício 2 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 15.2 Treating Smart Pointers Like Regular References](https://doc.rust-lang.org/stable/book/ch15-02-deref.html) e [15.3 Running Code on Cleanup with the `Drop` Trait](https://doc.rust-lang.org/stable/book/ch15-03-drop.html)
- [Documentação de `Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html) e de [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html)
- [Rust Design Patterns: `Deref` polymorphism (antipadrão)](https://rust-unofficial.github.io/patterns/anti_patterns/deref.html) e [RAII guards](https://rust-unofficial.github.io/patterns/patterns/behavioural/RAII.html)

---

[← Lição anterior](01-box-e-tipos-recursivos.md) · [Índice do módulo](README.md) · [Próxima lição →](03-rc-posse-compartilhada.md)
