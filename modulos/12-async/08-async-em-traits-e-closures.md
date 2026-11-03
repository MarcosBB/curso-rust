# 08 — `async fn` em traits e async closures

**Objetivos.** Ao final, você vai saber:

- declarar `async fn` em traits (estável desde a 1.75) e usá-las com genéricos;
- exigir que a future de um método seja `Send`, para usá-la com `tokio::spawn`;
- contornar a falta de `dyn` com `Pin<Box<dyn Future>>` (ou a crate `async-trait`);
- usar async closures (1.85) e os bounds `AsyncFn`, `AsyncFnMut` e `AsyncFnOnce`.

## `async fn` em traits

Desde a 1.75, uma trait pode ter métodos `async`:

```rust
trait Fonte {
    async fn ler(&self, chave: &str) -> Option<String>;
}

struct Memoria(HashMap<String, String>);

impl Fonte for Memoria {
    async fn ler(&self, chave: &str) -> Option<String> {
        self.0.get(chave).cloned()
    }
}

struct Lenta;

impl Fonte for Lenta {
    async fn ler(&self, chave: &str) -> Option<String> {
        sleep(Duration::from_millis(20)).await;
        Some(chave.to_uppercase())
    }
}

async fn ler_varias<F: Fonte>(fonte: &F, chaves: &[&str]) -> Vec<Option<String>> {
    let mut saida = Vec::new();
    for c in chaves {
        saida.push(fonte.ler(c).await);
    }
    saida
}
```

```text
[Some("Ana"), None]
[Some("NOME")]
```

Por baixo, `async fn ler(&self, ...) -> T` numa trait é açúcar para `fn ler(&self, ...) -> impl Future<Output = T>`. Cada implementação devolve o seu próprio tipo de future, e o genérico `F: Fonte` gera uma versão de `ler_varias` por tipo (monomorfização, Módulo 06). Sem alocação, sem `Box`.

Duas limitações aparecem logo, e as duas vêm do mesmo fato: **o tipo da future é escondido**.

## Limitação 1: `Send`

Tente rodar o método numa task:

```rust
// ❌ NÃO COMPILA
async fn processar<F: Fonte + Send + Sync + 'static>(f: F) {
    let h = tokio::spawn(async move { f.ler().await });
    println!("{}", h.await.unwrap());
}
```

```text
error: future cannot be sent between threads safely
   --> src/main.rs:14:13
    |
 14 |     let h = tokio::spawn(async move { f.ler().await });
    |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ future created by async block is not `Send`
    |
    = help: within `{async block@src/main.rs:14:26: 14:36}`, the trait `Send` is not implemented for `impl Future<Output = String>`
note: future is not `Send` as it awaits another future which is not `Send`
   --> src/main.rs:14:39
    |
 14 |     let h = tokio::spawn(async move { f.ler().await });
    |                                       ^^^^^^^ await occurs here on type `impl Future<Output = String>`, which is not `Send`
...
help: `Send` can be made part of the associated future's guarantees for all implementations of `Fonte::ler`
    |
  2 -     async fn ler(&self) -> String;
  2 +     fn ler(&self) -> impl std::future::Future<Output = String> + Send;
    |
```

(Neste teste, `ler` não recebia a chave.) Exigir `F: Send` não basta: o tipo `F` pode ser `Send`, mas a future que o método **devolve** pode conter um `Rc` ou um `MutexGuard` atravessando um `.await`. A trait não promete nada sobre ela, então o compilador não pode supor.

A correção é a que o próprio compilador sugere: escrever o método na forma "sem açúcar" e pôr o `+ Send` na promessa:

```rust
trait FonteSend {
    fn ler(&self, chave: &str) -> impl Future<Output = Option<String>> + Send;
}

impl FonteSend for Lenta {
    // A implementação pode continuar usando `async fn`.
    async fn ler(&self, chave: &str) -> Option<String> {
        sleep(Duration::from_millis(20)).await;
        Some(chave.to_uppercase())
    }
}

async fn ler_em_task<F>(fonte: F, chave: String) -> Option<String>
where
    F: FonteSend + Send + 'static,
{
    tokio::spawn(async move { fonte.ler(&chave).await })
        .await
        .unwrap()
}
```

Agora toda implementação é obrigada a devolver uma future `Send`, e o compilador verifica isso **nas implementações**. A desvantagem é que nenhuma implementação pode usar `Rc` através de um `.await`, mesmo quem nunca vai usar a trait com `spawn`.

É por isso que o compilador avisa quando você usa `async fn` numa trait **pública**:

```text
warning: use of `async fn` in public traits is discouraged as auto trait bounds cannot be specified
 --> src/lib.rs:2:5
  |
2 |     async fn ler(&self) -> String;
  |     ^^^^^
  |
  = note: you can suppress this lint if you plan to use the trait only in your own code, or do not care about auto traits like `Send` on the `Future`
  = note: `#[warn(async_fn_in_trait)]` on by default
help: you can alternatively desugar to a normal `fn` that returns `impl Future` and add any desired bounds such as `Send`, but these cannot be relaxed without a breaking API change
```

Quem usa a sua biblioteca com tokio vai precisar de `Send`, e acrescentar o `Send` depois quebra quem já implementou a trait sem ele. A regra prática:

- trait privada ao seu crate: `async fn` à vontade;
- trait pública usada com tokio: `fn ...(&self) -> impl Future<Output = T> + Send`.

> [!NOTE]
> Existe uma sintaxe para pedir `Send` só no ponto de uso, sem mexer na trait: `F::ler(..): Send`, chamada *return type notation* (RTN). Em novembro de 2026 ela ainda é **instável** (só no nightly). A crate `trait-variant`, do próprio time do Rust, gera as duas versões da trait (com e sem `Send`) a partir de uma só.

## Limitação 2: `dyn`

```rust
// ❌ NÃO COMPILA
let fontes: Vec<Box<dyn Fonte>> = vec![Box::new(Fixa)];
```

```text
error[E0038]: the trait `Fonte` is not dyn compatible
  --> src/main.rs:15:29
   |
15 |     let fontes: Vec<Box<dyn Fonte>> = vec![Box::new(Fixa)];
   |                             ^^^^^ `Fonte` is not dyn compatible
   |
note: for a trait to be dyn compatible it needs to allow building a vtable
      for more information, visit <https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility>
  --> src/main.rs:2:14
   |
 1 | trait Fonte {
   |       ----- this trait is not dyn compatible...
 2 |     async fn ler(&self) -> String;
   |              ^^^ ...because method `ler` is `async`
```

É a regra do Módulo 06: "se não dá para pôr numa tabela de funções, a trait não vira `dyn`". Cada implementação devolve uma future de tipo e **tamanho** diferentes, e uma entrada da *vtable* precisa de um tipo de retorno só.

A saída é fazer todas as implementações devolverem o **mesmo** tipo: uma future no heap, atrás de um ponteiro, o `Pin<Box<dyn Future>>` da lição 07:

```rust
/// O tipo de future que um método "async" dyn-compatível devolve.
type FutureBox<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait FonteDyn {
    fn ler<'a>(&'a self, chave: &'a str) -> FutureBox<'a, Option<String>>;
}

impl FonteDyn for Lenta {
    fn ler<'a>(&'a self, chave: &'a str) -> FutureBox<'a, Option<String>> {
        Box::pin(async move {
            sleep(Duration::from_millis(20)).await;
            Some(chave.to_uppercase())
        })
    }
}

async fn primeira_que_tem(fontes: &[Box<dyn FonteDyn>], chave: &str) -> Option<String> {
    for f in fontes {
        if let Some(v) = f.ler(chave).await {
            return Some(v);
        }
    }
    None
}
```

```text
Some("Ana")
Some("CIDADE")
```

(Com uma `Memoria` que tem `"nome"` e uma `Lenta` na lista.) As peças do tipo:

- `Box<dyn Future<...>>`: tamanho fixo, cabe na vtable;
- `Pin<...>`: o runtime pode fazer poll nela;
- `+ Send`: pode ir para outra thread (tire se não precisar);
- `+ 'a`: a future empresta `self` e `chave`, então não pode viver mais que eles.

O custo é uma alocação por chamada, parecido com o que o C# paga ao criar cada `Task`. Na maioria dos programas, é desprezível perto da E/S que o método faz.

### A crate `async-trait`

Escrever `Box::pin(async move { ... })` em toda implementação cansa. A crate [`async-trait`](https://docs.rs/async-trait) (0.1.92) faz essa transformação com uma *macro* de atributo (macros são o Módulo 14):

```rust
#[async_trait::async_trait]
trait FonteDyn {
    async fn ler(&self, chave: &str) -> Option<String>;
}
```

Ela gera exatamente o `Pin<Box<dyn Future + Send + '_>>` acima. Antes da 1.75, era o único jeito de ter `async fn` em traits, e por isso aparece em muito código e tutorial antigo. Hoje, **use a `async-trait` só quando precisar de `dyn`**. Para genéricos, `async fn` nativa é mais rápida (sem alocação) e dispensa dependência.

| Preciso de | Use |
|---|---|
| genéricos, trait só minha | `async fn` na trait |
| genéricos + `tokio::spawn`, trait pública | `fn ... -> impl Future<Output = T> + Send` |
| `dyn Trait` | `Pin<Box<dyn Future + Send + 'a>>` à mão, ou `#[async_trait]` |
| um conjunto fechado de tipos | um `enum` com um `match` (Módulo 06): sem `dyn`, sem alocação |

## Async closures

Uma função que recebe "algo assíncrono para chamar" era difícil de escrever antes da 1.85. O jeito antigo era uma closure comum que devolve uma future, com dois parâmetros genéricos:

```rust
// ❌ NÃO COMPILA
async fn para_cada<F, Fut>(nomes: &[String], f: F)
where
    F: Fn(&String) -> Fut,
    Fut: Future<Output = ()>,
{
    for n in nomes {
        f(n).await;
    }
}

#[tokio::main]
async fn main() {
    let nomes = vec![String::from("ana"), String::from("bia")];
    para_cada(&nomes, |n| async move {
        println!("olá, {n}");
    })
    .await;
}
```

```text
error: lifetime may not live long enough
  --> src/main.rs:14:27
   |
14 |       para_cada(&nomes, |n| async move {
   |  ________________________--_^
   | |                        ||
   | |                        |return type of closure `()` contains a lifetime `'2`
   | |                        has type `&'1 String`
15 | |         println!("olá, {n}");
16 | |     })
   | |_____^ returning this value requires that `'1` must outlive `'2`
```

A future devolvida empresta `n`, então o tipo dela depende do *lifetime* de cada argumento. Mas `Fut` é **um** tipo, escolhido uma vez para todas as chamadas, e não pode depender desse lifetime. Não há como escrever esse bound com `Fn` + `Future`.

As async closures resolvem isso. `async |n| { ... }` é uma closure cuja chamada devolve uma future que **pode** emprestar dos argumentos e das capturas. E o bound é uma das três traits novas, `AsyncFn`, `AsyncFnMut` e `AsyncFnOnce`, espelhos de `Fn`, `FnMut` e `FnOnce` (Módulo 07):

```rust
async fn para_cada<F>(nomes: &[String], f: F)
where
    F: AsyncFn(&String),
{
    for n in nomes {
        f(n).await;
    }
}

let nomes = vec![String::from("ana"), String::from("bia")];
let saudacao = String::from("olá");
para_cada(&nomes, async |n| println!("{saudacao}, {n}")).await;
```

```text
olá, ana
olá, bia
```

`AsyncFnMut` permite que a closure mude o que capturou, como o `FnMut`. Uma função de retentativa genérica:

```rust
async fn com_retentativa<T, E, F>(tentativas: u32, mut op: F) -> Result<T, E>
where
    F: AsyncFnMut(u32) -> Result<T, E>,
{
    let mut tentativa = 1;
    loop {
        match op(tentativa).await {
            Ok(v) => return Ok(v),
            Err(e) if tentativa >= tentativas => return Err(e),
            Err(_) => tentativa += 1,
        }
    }
}

let mut chamadas = Vec::new();
let r: Result<&str, String> = com_retentativa(5, async |t| {
    chamadas.push(t);
    sleep(Duration::from_millis(5)).await;
    if t < 3 {
        Err(format!("falhou na {t}"))
    } else {
        Ok("conectou")
    }
})
.await;
println!("{r:?} depois das tentativas {chamadas:?}");
```

```text
Ok("conectou") depois das tentativas [1, 2, 3]
```

A closure muta `chamadas`, um `Vec` do chamador, a cada tentativa. Com o jeito antigo (`FnMut() -> Fut`), a future devolvida não poderia emprestar `chamadas`, pelo mesmo motivo do erro acima.

> [!TIP]
> Uma closure comum que devolve um bloco `async`, `|| async { ... }`, ainda funciona onde o bound é `AsyncFn` quando a future não empresta nada da closure. Mas prefira escrever `async || { ... }`: é mais curto e funciona nos dois casos.

Código completo, com testes, em [`codigo/traits-e-closures`](codigo/traits-e-closures/src/main.rs).

## Armadilhas comuns

- **`async fn` em trait pública usada com tokio.** Sem `+ Send` na assinatura, quem chama não consegue usar `tokio::spawn`. Leia o lint `async_fn_in_trait`.
- **`dyn Trait` com método `async`.** É E0038. Devolva `Pin<Box<dyn Future + Send + 'a>>` ou use `#[async_trait]`.
- **Usar `async-trait` por hábito.** Desde a 1.75, só vale para `dyn`.
- **`F: Fn(&T) -> Fut` com futures que emprestam o argumento.** Não tem como escrever o bound. Use `AsyncFn(&T)`.

## Resumo

- `async fn` em traits é estável (1.75) e funciona sem custo com genéricos.
- A future do método não é garantidamente `Send`; para isso, escreva `fn ... -> impl Future<Output = T> + Send`.
- Traits com `async fn` não são dyn-compatíveis; a saída é `Pin<Box<dyn Future + Send + 'a>>`, à mão ou com `async-trait`.
- Async closures (`async |x| ...`, 1.85) e `AsyncFn`/`AsyncFnMut`/`AsyncFnOnce` permitem que a future empreste argumentos e capturas.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Blog do Rust: Announcing `async fn` and return-position `impl Trait` in traits](https://blog.rust-lang.org/2023/12/21/async-fn-rpit-in-traits/)
- [Blog do Rust: Rust 1.85.0 (async closures)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
- [Documentação de `AsyncFn`](https://doc.rust-lang.org/std/ops/trait.AsyncFn.html)
- [The Rust Reference: dyn compatibility](https://doc.rust-lang.org/reference/items/traits.html#dyn-compatibility)
- [Documentação da crate `async-trait`](https://docs.rs/async-trait)

---

[← Lição anterior](07-pin-e-unpin.md) · [Índice do módulo](README.md) · [Próximo: exercícios →](exercicios.md)
