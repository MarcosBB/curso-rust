# 05 — `Weak` e ciclos de referência

**Objetivos.** Ao final, você vai saber:

- por que dois `Rc` que apontam um para o outro vazam memória, e como perceber isso;
- usar `Weak` para observar um valor sem mantê-lo vivo (`downgrade` e `upgrade`);
- montar uma árvore em que o pai é dono dos filhos e o filho enxerga o pai;
- escolher entre `&T`, `Box`, `Rc`, `Rc<RefCell<T>>` e `Weak`.

## O ciclo que vaza

No Módulo 00 e no Módulo 02 ficou a promessa: o Rust impede *use-after-free* e *double free*, mas permite vazar memória, e o exemplo clássico são dois `Rc` em ciclo. Com o `Rc<RefCell<T>>` da lição anterior, dá para montar um:

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct No {
    nome: &'static str,
    proximo: RefCell<Option<Rc<No>>>,
}

impl Drop for No {
    fn drop(&mut self) {
        println!("  drop {}", self.nome);
    }
}

fn ciclo_que_vaza() {
    let a = Rc::new(No { nome: "a", proximo: RefCell::new(None) });
    let b = Rc::new(No { nome: "b", proximo: RefCell::new(Some(Rc::clone(&a))) });
    *a.proximo.borrow_mut() = Some(Rc::clone(&b)); // a -> b -> a
    println!("a: {} donos, b: {} donos", Rc::strong_count(&a), Rc::strong_count(&b));
} // `a` e `b` saem de escopo, mas cada nó ainda tem 1 dono: o outro

// no main:
ciclo_que_vaza();
println!("fim do ciclo_que_vaza (nenhum drop acima)");
```

```text
a: 2 donos, b: 2 donos
fim do ciclo_que_vaza (nenhum drop acima)
```

O `Drop` imprime uma linha, como o `Barulhento` do Módulo 02, e ela **nunca aparece**. No fim da função, as variáveis `a` e `b` são descartadas, e cada contador desce de 2 para 1. Os nós continuam se segurando, e nenhum contador chega a zero. Nada mais no programa alcança esses nós, e eles ficam no heap até o processo terminar.

![Em cima, depois que a e b saem de escopo, o nó a aponta para b e b aponta para a, ambos com Rc e strong = 1: ninguém de fora os alcança, e a memória vaza. Embaixo, uma árvore: raiz: Rc<Pasta> aponta para a pasta home, que é dona da pasta ana pela lista de filhas (Rc); ana aponta de volta para home com um Weak tracejado. Quando raiz sai de escopo, home vai a strong = 0 e é descartada, levando as filhas.](assets/ciclo-rc-weak.svg)

O vazamento não é *undefined behavior*: nenhuma memória é lida depois de liberada, e nenhum dado é corrompido. Por isso o Rust o considera seguro (`std::mem::forget` é uma função segura pelo mesmo motivo). Mas é um bug, e o compilador não avisa.

### E em Python?

O mesmo código em Python não vaza:

```python
class No:
    def __init__(self, nome):
        self.nome, self.proximo = nome, None
    def __del__(self):
        print("del", self.nome)

a, b = No("a"), No("b")
a.proximo, b.proximo = b, a
del a, b
import gc; gc.collect()  # o coletor de ciclos encontra e libera os dois
```

O CPython conta referências, como o `Rc`, e também roda um **coletor de ciclos** de tempos em tempos, que procura grupos de objetos que só se alcançam entre si. O Rust não tem coletor nenhum, e o C++ também não: dois `shared_ptr` em ciclo vazam do mesmo jeito. Nas duas linguagens, a solução é a mesma: uma das setas não pode contar como dono.

## `Weak<T>`: observar sem ser dono

`Weak<T>` é um ponteiro para um valor de um `Rc` que **não** conta como dono. Ele aumenta o contador `weak` (o segundo contador da lição 03), não o `strong`. O valor é descartado quando o `strong` chega a zero, haja quantos `Weak` houver.

- `Rc::downgrade(&rc)` cria um `Weak` a partir de um `Rc`.
- `weak.upgrade()` tenta voltar: devolve `Some(Rc<T>)` se o valor ainda existe, ou `None` se já foi descartado.
- `Weak::new()` cria um `Weak` que não aponta para nada (o `upgrade` dá sempre `None`), útil como valor inicial.

O `upgrade` devolve `Option` porque, ao contrário de um `&T`, um `Weak` pode sobreviver ao valor. É o mesmo contrato do `weak_ptr::lock()` do C++, que devolve um `shared_ptr` vazio, e do `weakref.ref(obj)()` do Python, que devolve `None`. A diferença em Rust é que o `Option` obriga você a tratar o caso.

Trocando a volta do ciclo por `Weak`:

```rust
struct NoFraco {
    nome: &'static str,
    proximo: RefCell<Weak<NoFraco>>,
}

let a = Rc::new(NoFraco { nome: "a", proximo: RefCell::new(Weak::new()) });
let b = Rc::new(NoFraco { nome: "b", proximo: RefCell::new(Rc::downgrade(&a)) });
*a.proximo.borrow_mut() = Rc::downgrade(&b);
// imprime os contadores...
if let Some(prox) = a.proximo.borrow().upgrade() {
    println!("depois de a vem {}", prox.nome);
}
```

```text
a: 1 forte, 1 fraco; b: 1 forte, 1 fraco
depois de a vem b
  drop b
  drop a
fim do ciclo_com_weak
```

Agora cada nó tem um dono só (a variável), e os dois são descartados no fim da função.

> [!NOTE]
> Depois que o `strong` chega a zero, o **valor** é descartado, mas a **alocação** (com os dois contadores) fica até o último `Weak` sumir, para que o `upgrade` desses `Weak` possa responder `None`. Para um `T` grande, a memória do `T` fica reservada até lá, ainda que já descartada. Raramente importa.

## Uma árvore de pastas

O caso real de `Weak` é uma estrutura com "pai" e "filhos". A regra: **quem é dono aponta com `Rc`; quem só precisa enxergar aponta com `Weak`**. A pasta é dona das subpastas; a subpasta só precisa saber quem é a pasta de cima.

```rust
struct Pasta {
    nome: String,
    pai: RefCell<Weak<Pasta>>,
    filhas: RefCell<Vec<Rc<Pasta>>>,
}

impl Pasta {
    fn nova(nome: &str) -> Rc<Pasta> {
        Rc::new(Pasta {
            nome: nome.to_string(),
            pai: RefCell::new(Weak::new()),
            filhas: RefCell::new(Vec::new()),
        })
    }

    /// Cria uma subpasta dentro de `pai` e a devolve.
    fn criar_em(pai: &Rc<Pasta>, nome: &str) -> Rc<Pasta> {
        let filha = Pasta::nova(nome);
        *filha.pai.borrow_mut() = Rc::downgrade(pai);
        pai.filhas.borrow_mut().push(Rc::clone(&filha));
        filha
    }

    /// O caminho completo, subindo pelos pais enquanto eles existirem.
    fn caminho(&self) -> String {
        match self.pai.borrow().upgrade() {
            Some(pai) => format!("{}/{}", pai.caminho(), self.nome),
            None => self.nome.clone(),
        }
    }
}
```

Os dois campos são `RefCell` porque são alterados depois que o `Rc` existe (o pai de `filha` é preenchido em `criar_em`, e a lista de filhas cresce), e um `Rc` só dá `&Pasta`. O `criar_em` recebe `&Rc<Pasta>`, e não `&Pasta`, porque precisa do `Rc` para chamar `downgrade`.

```rust
let fotos;
{
    let raiz = Pasta::nova("home");
    let ana = Pasta::criar_em(&raiz, "ana");
    fotos = Pasta::criar_em(&ana, "fotos");
    println!("{}", fotos.caminho());
    println!(
        "ana: {} forte (raiz e a variável), {} fraco (fotos)",
        Rc::strong_count(&ana),
        Rc::weak_count(&ana)
    );
} // `raiz` cai: home não tem mais dono forte, e leva ana junto
println!("fora do bloco: {}", fotos.caminho());
println!("pai de fotos ainda existe? {}", fotos.pai.borrow().upgrade().is_some());
```

```text
home/ana/fotos
ana: 2 forte (raiz e a variável), 1 fraco (fotos)
  drop pasta home
  drop pasta ana
fora do bloco: fotos
pai de fotos ainda existe? false
  drop pasta fotos
```

Quando o bloco termina, a variável `ana` cai (o `strong` de ana vai a 1) e depois `raiz` (o de home vai a 0). Home é descartada, e com ela a lista de filhas, o que leva o `strong` de ana a 0. A pasta `fotos` sobrevive porque a variável de fora ainda é dona dela. O `caminho()` dela agora para em `fotos`, porque o `upgrade` do pai devolve `None`. Nenhum ponteiro pendente, nenhum vazamento. Código completo, com testes, em [`codigo/arvore-weak`](codigo/arvore-weak/src/main.rs).

> [!TIP]
> Para um nó que precisa de um `Weak` para **si mesmo** já na criação (um objeto que se registra num observador, por exemplo), existe `Rc::new_cyclic(|eu: &Weak<T>| ...)`.

## Qual ponteiro escolher

A ordem de preferência do Módulo 03 continua valendo: comece pelo mais simples e só suba quando precisar.

| Você precisa de... | Use | C++ | Python |
|---|---|---|---|
| acessar sem ser dono, com o dono conhecido | `&T` / `&mut T` | `const T&` / `T&` | (toda variável) |
| um dono, valor no heap (recursão, `dyn`, valor grande) | `Box<T>` | `unique_ptr<T>` | (todo objeto) |
| vários donos, só leitura | `Rc<T>` | `shared_ptr<const T>` | (todo objeto) |
| vários donos que alteram | `Rc<RefCell<T>>` | `shared_ptr<T>` | (todo objeto) |
| alterar através de `&self` | `Cell<T>` / `RefCell<T>` / `OnceCell<T>` | membro `mutable` | (todo atributo) |
| enxergar sem manter vivo (pai, cache, observador) | `Weak<T>` | `weak_ptr<T>` | `weakref.ref` |
| tudo isso entre threads | `Arc`, `Mutex`, `RwLock`, `OnceLock` (Módulo 11) | | |

Antes de montar um grafo com `Rc<RefCell<No>>` e `Weak`, considere a alternativa que costuma ser mais simples em Rust: guardar todos os nós num `Vec<No>` e usar **índices** (`usize`) no lugar dos ponteiros. Um índice não é dono de nada, não tem contador, não forma ciclo e passa pelo borrow checker sem cerimônia. A desvantagem é que um índice velho não "sabe" que o nó foi removido; crates como `slotmap` resolvem isso.

## Armadilhas comuns

**`Rc` nas duas direções.** Qualquer estrutura com setas de ida e volta (pai e filho, lista duplamente encadeada, observador e observado) tem que decidir quem é dono. Se as duas setas forem `Rc`, vaza. Teste com um `Drop` que imprime, ou confira `Rc::strong_count` num teste, como o projeto faz.

**`upgrade().unwrap()` por reflexo.** O `Weak` existe justamente porque o alvo pode ter sumido. Trate o `None`: no exemplo, uma pasta sem pai vivo é tratada como raiz.

**Segurar o `Rc` do `upgrade` por muito tempo.** O `Rc` que o `upgrade` devolve é um dono como outro qualquer: enquanto ele existir, o valor não é descartado. Use-o e solte-o, em vez de guardá-lo numa struct (aí o `Weak` deixou de servir para alguma coisa).

## Resumo

- Dois `Rc` em ciclo nunca chegam a zero: o `Drop` não roda e a memória vaza. É seguro (não corrompe nada), mas é bug, e o compilador não avisa.
- Python libera ciclos com o coletor de ciclos; Rust e C++ não têm coletor, e a solução é uma seta fraca.
- `Weak<T>` aponta sem ser dono: `Rc::downgrade` cria, `upgrade()` devolve `Option<Rc<T>>`. É o `weak_ptr` do C++ e o `weakref` do Python.
- Em árvores: o pai é dono dos filhos (`Rc`), o filho enxerga o pai (`Weak`).
- Escolha o ponteiro mais simples que resolve. Para grafos, considere um `Vec` com índices antes de `Rc<RefCell<T>>`.

## Exercícios

Veja o exercício 4 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 15.6 Reference Cycles Can Leak Memory](https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html)
- [Documentação de `std::rc::Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html) e de [`Rc::new_cyclic`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.new_cyclic)
- [The Rustonomicon: Leaking](https://doc.rust-lang.org/nomicon/leaking.html) (por que vazar é seguro)
- [Learning Rust With Entirely Too Many Linked Lists, cap. 5](https://rust-unofficial.github.io/too-many-lists/fourth.html) (lista dupla com `Rc<RefCell<T>>`; edition antiga)

---

[← Lição anterior](04-cell-e-refcell.md) · [Índice do módulo](README.md) · [Próxima lição →](06-cow.md)
