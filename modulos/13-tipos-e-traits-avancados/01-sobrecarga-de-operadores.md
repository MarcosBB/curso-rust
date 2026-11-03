# 01 — Sobrecarga de operadores

**Objetivos.** Ao final, você vai saber:

- implementar `+`, `-`, `*`, `-x`, `+=` e `[]` para os seus tipos com as traits de `std::ops`;
- escrever operadores com tipos diferentes dos dois lados (`Vetor * f64` e `f64 * Vetor`);
- implementar operadores por referência para tipos que não são `Copy`;
- decidir quando um operador ajuda e quando atrapalha.

## O problema: `+` não existe para o seu tipo

```rust
// ❌ NÃO COMPILA
#[derive(Debug, Clone, Copy, PartialEq)]
struct Vetor {
    x: f64,
    y: f64,
}

fn main() {
    let a = Vetor { x: 1.0, y: 2.0 };
    let b = Vetor { x: 3.0, y: 4.0 };
    let c = a + b;
    println!("{c:?}");
}
```

```text
error[E0369]: cannot add `Vetor` to `Vetor`
  --> src/main.rs:10:15
   |
10 |     let c = a + b;
   |             - ^ - Vetor
   |             |
   |             Vetor
   |
note: an implementation of `Add` might be missing for `Vetor`
  --> src/main.rs:2:1
   |
 2 | struct Vetor {
   | ^^^^^^^^^^^^ must implement `Add`
note: the trait `Add` must be implemented
  ...
```

Em Rust, um operador é só açúcar para a chamada de um método de trait. `a + b` vira `Add::add(a, b)`, `a * b` vira `Mul::mul(a, b)`, e assim por diante. As traits ficam em [`std::ops`](https://doc.rust-lang.org/std/ops/index.html). O `Add` você já leu no [Módulo 06](../06-traits-e-generics/08-tipos-associados-e-traits-genericas.md):

```rust
trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
```

`Rhs` é o tipo do lado direito (por padrão, o mesmo do esquerdo) e `Output` é o tipo do resultado. Implementar a trait é tudo o que falta:

```rust
use std::ops::Add;

impl Add for Vetor {
    type Output = Vetor;

    fn add(self, outro: Vetor) -> Vetor {
        Vetor::new(self.x + outro.x, self.y + outro.y)
    }
}
```

Os outros operadores seguem o mesmo molde. `Sub` é igual ao `Add`. `Neg` (o `-` de um operando só) não tem `Rhs`:

```rust
impl Neg for Vetor {
    type Output = Vetor;

    fn neg(self) -> Vetor {
        Vetor::new(-self.x, -self.y)
    }
}
```

## Comparando com Python e C++

O mecanismo é o mesmo nas três linguagens: o operador chama uma função com nome especial.

| Operador | Python | C++ | Rust (`std::ops`) |
|---|---|---|---|
| `a + b` | `__add__` | `operator+` | `Add` |
| `a - b` | `__sub__` | `operator-` | `Sub` |
| `a * b` | `__mul__` | `operator*` | `Mul` |
| `-a` | `__neg__` | `operator-()` | `Neg` |
| `a += b` | `__iadd__` | `operator+=` | `AddAssign` |
| `a[i]` | `__getitem__` / `__setitem__` | `operator[]` | `Index` / `IndexMut` |
| `a == b`, `a < b` | `__eq__`, `__lt__` | `operator==`, `operator<=>` | `PartialEq`, `PartialOrd` (Módulo 06) |
| `!a` | `__invert__` (para `~`) | `operator!` | `Not` |
| `a & b`, `a << b`... | `__and__`, `__lshift__` | `operator&`, `operator<<` | `BitAnd`, `Shl`... |

O que muda é **quando** você descobre que um operador está faltando. Em Python, `v + 1` com um `Vetor` que só sabe somar com `Vetor` falha ao rodar aquela linha. Em Rust, falha na compilação, como você acabou de ver. Em C++, também na compilação, mas o erro de um `operator+` ausente dentro de um template pode ficar a dezenas de linhas do lugar certo.

Algumas coisas **não** dá para sobrecarregar em Rust: `&&` e `||` (curto-circuito), `=` (atribuição é sempre um *move* ou uma cópia), `.` (acesso a campo) e a conversão implícita. Você também não cria operadores novos, como em Haskell ou Swift. A lista fechada é de propósito: quem lê `a = b` sabe exatamente o que acontece.

## Tipos diferentes dos dois lados

Multiplicar um vetor por um número é natural. O `Rhs` deixa de ser `Self`:

```rust
use std::ops::Mul;

// Vetor * escalar
impl Mul<f64> for Vetor {
    type Output = Vetor;

    fn mul(self, k: f64) -> Vetor {
        Vetor::new(self.x * k, self.y * k)
    }
}
```

Agora `v * 2.0` funciona. E `2.0 * v`?

```rust
// ❌ NÃO COMPILA
println!("{:?}", v * 2.0);
println!("{:?}", 2.0 * v);
```

```text
error[E0277]: cannot multiply `{float}` by `Vetor`
  --> src/main.rs:19:26
   |
19 |     println!("{:?}", 2.0 * v);
   |                          ^ no implementation for `{float} * Vetor`
   |
   = help: the trait `Mul<Vetor>` is not implemented for `{float}`
   = help: the following other types implement trait `Mul<Rhs>`:
             `&f128` implements `Mul<f128>`
             ...
           and 57 others
```

A multiplicação não é comutativa para o compilador: `2.0 * v` é `<f64 as Mul<Vetor>>::mul(2.0, v)`, outra implementação. Você pode escrevê-la, mesmo `f64` sendo da biblioteca padrão, porque a *orphan rule* (Módulo 06) só exige que **um** dos tipos envolvidos seja seu, e `Vetor` é:

```rust
// escalar * Vetor
impl Mul<Vetor> for f64 {
    type Output = Vetor;

    fn mul(self, v: Vetor) -> Vetor {
        v * self
    }
}
```

O Python resolve o mesmo problema em tempo de execução. Quando `int.__mul__(2, v)` não sabe lidar com um `Vetor`, ele devolve `NotImplemented`, e o interpretador tenta `v.__rmul__(2)`. Sem o `__rmul__`:

```python
print(v * 2)   # Vetor(2, 4)
print(2 * v)   # TypeError: unsupported operand type(s) for *: 'int' and 'Vetor'
```

Em Rust não existe esse "tenta o outro lado": cada combinação de tipos é uma `impl` própria, escolhida na compilação.

## `+=` e companhia

`a += b` **não** é traduzido para `a = a + b`. Ele tem a sua própria trait, `AddAssign`, que recebe `&mut self` e muda o valor no lugar:

```rust
use std::ops::AddAssign;

impl AddAssign for Vetor {
    fn add_assign(&mut self, outro: Vetor) {
        self.x += outro.x;
        self.y += outro.y;
    }
}

let mut posicao = Vetor::default();
posicao += a;
posicao += b;
```

Para um `Vetor` tanto faz. Para um tipo que guarda um `Vec` ou uma `String`, o `+=` no lugar evita criar um valor novo a cada soma, e é por isso que `String` implementa `AddAssign<&str>`.

> [!TIP]
> Para que `iter.sum()` funcione com o seu tipo, implemente [`std::iter::Sum`](https://doc.rust-lang.org/std/iter/trait.Sum.html). Ela não é um operador, mas anda junto: `impl Sum for Vetor { fn sum<I: Iterator<Item = Vetor>>(iter: I) -> Vetor { iter.fold(Vetor::default(), |acc, v| acc + v) } }`. Para somar um iterador de `&Vetor`, implemente também `Sum<&'a Vetor>`, ou use `.copied()`.

## Tipos que não são `Copy`: operadores por referência

`Vetor` é `Copy`, então `a + b` copia os dois e ninguém percebe que `add` recebe `self` por valor. Com um tipo que é dono de memória, o *move* aparece:

```rust
// ❌ NÃO COMPILA
/// Coeficientes do menor para o maior grau: `[1, 2]` é `1 + 2x`.
#[derive(Debug, Clone, PartialEq)]
struct Polinomio(Vec<i64>);

impl Add for Polinomio { /* ... */ }

fn main() {
    let p = Polinomio(vec![1, 2]);
    let q = Polinomio(vec![0, 0, 3]);
    let r = p + q;
    println!("{r:?} {p:?}");
}
```

```text
error[E0382]: borrow of moved value: `p`
  --> src/main.rs:21:22
   |
18 |     let p = Polinomio(vec![1, 2]);
   |         - move occurs because `p` has type `Polinomio`, which does not implement the `Copy` trait
19 |     let q = Polinomio(vec![0, 0, 3]);
20 |     let r = p + q;
   |             ----- `p` moved due to usage in operator
21 |     println!("{r:?} {p:?}");
   |                      ^ value borrowed here after move
   |
note: calling this operator moves the left-hand side
  ...
help: consider cloning the value if the performance cost is acceptable
   |
20 |     let r = p.clone() + q;
   |              ++++++++
```

`p + q` consome os dois. O `clone()` sugerido funciona, mas copia o vetor inteiro só para ler. A saída idiomática é implementar o operador **para referências**: `Self` passa a ser `&Polinomio`, e o "consumir" vira consumir uma referência, o que não custa nada.

```rust
impl Add<&Polinomio> for &Polinomio {
    type Output = Polinomio;

    fn add(self, outro: &Polinomio) -> Polinomio {
        let n = self.0.len().max(outro.0.len());
        let coef = (0..n)
            .map(|i| self.0.get(i).unwrap_or(&0) + outro.0.get(i).unwrap_or(&0))
            .collect();
        Polinomio(coef)
    }
}

// A versão por valor reaproveita a por referência.
impl Add for Polinomio {
    type Output = Polinomio;

    fn add(self, outro: Polinomio) -> Polinomio {
        &self + &outro
    }
}

let r = &p + &q; // p e q continuam valendo
```

```text
Polinomio([1, 2]) + Polinomio([0, 0, 3]) = Polinomio([1, 2, 3])
```

A biblioteca padrão faz o mesmo: os números implementam `Add` nas quatro combinações (`i32 + i32`, `&i32 + i32`, `i32 + &i32`, `&i32 + &i32`), e foi isso que apareceu na lista do E0277 acima. Para os seus tipos, implemente as combinações que alguém vai de fato usar. Em geral, `&T op &T` e `T op T` bastam.

O C++ tem a mesma escolha, com outra sintaxe: `Polinomio operator+(const Polinomio& a, const Polinomio& b)` recebe por referência constante. A diferença é que, em C++, passar por valor **copia** em silêncio (chama o construtor de cópia), e em Rust **move**, com o erro acima se você tentar usar o original depois.

## Indexação: `Index` e `IndexMut`

`m[i]` chama `Index::index(&m, i)`, que devolve uma **referência**. O compilador desreferencia o resultado sozinho, então `m[i]` tem o tipo do elemento. O índice pode ser de qualquer tipo, como uma tupla:

```rust
use std::ops::{Index, IndexMut};

struct Matriz {
    colunas: usize,
    dados: Vec<f64>,
}

impl Index<(usize, usize)> for Matriz {
    type Output = f64;

    fn index(&self, (l, c): (usize, usize)) -> &f64 {
        assert!(c < self.colunas, "coluna {c} fora da matriz");
        &self.dados[l * self.colunas + c]
    }
}

impl IndexMut<(usize, usize)> for Matriz {
    fn index_mut(&mut self, (l, c): (usize, usize)) -> &mut f64 {
        assert!(c < self.colunas, "coluna {c} fora da matriz");
        &mut self.dados[l * self.colunas + c]
    }
}

let mut m = Matriz::zeros(2, 3);
m[(1, 2)] = 7.5;  // IndexMut
m[(0, 0)] += 1.0; // IndexMut, e depois o += do f64
println!("{}", m[(1, 2)]); // Index
```

```text
m[(1, 2)] = 7.5, m[(0, 0)] = 1
```

A convenção é a do `Vec`: índice fora da faixa é *panic* (por isso o `assert!` na coluna, que senão cairia em outra linha em silêncio). Ofereça também um método `get` que devolve `Option<&T>`, para quem não quer arriscar o panic.

## Quando **não** sobrecarregar

A regra das [API Guidelines](https://rust-lang.github.io/api-guidelines/predictability.html#c-overload) é curta: só implemente um operador se o tipo se comportar como aquele operador se comporta nos números. Somar vetores, matrizes, durações, dinheiro: sim. Usar `+` para "adicionar um usuário a um grupo" ou `<<` para escrever num arquivo (como o `std::cout << x` do C++): não. O leitor espera que `a + b` não tenha efeito colateral, que seja barato de ler e que `a + b == b + a` quando faz sentido.

Também resista a implementar `Add<i64> for Dinheiro`: somar um número cru a um valor em reais é justamente o erro que o tipo existe para impedir. Esse é o assunto da próxima lição.

Código completo, com testes, em [`codigo/operadores`](codigo/operadores/src/main.rs).

## Armadilhas comuns

- **Esperar que `2.0 * v` saia de graça do `v * 2.0`.** São duas implementações. Escreva a segunda delegando para a primeira.
- **Achar que `+=` usa o `Add`.** Ele precisa de `AddAssign`. Sem ela, o erro é E0368: *binary assignment operation `+=` cannot be applied to type `Vetor`*.
- **Implementar só `T + T` para um tipo grande.** Quem usa vai ser forçado a clonar. Implemente `&T + &T`.
- **`Index` devolvendo um valor.** `index` devolve `&Self::Output`. Se você precisa calcular o elemento na hora (não existe na memória), um método comum (`get(i) -> T`) é o caminho.

## Resumo

- Cada operador é uma trait de `std::ops`: `a + b` é `Add::add(a, b)`.
- `Rhs` escolhe o tipo do lado direito e `Output`, o do resultado. `f64 * Vetor` e `Vetor * f64` são `impl`s diferentes, e a orphan rule deixa escrever as duas.
- `+=` tem trait própria (`AddAssign`), com `&mut self`.
- Para tipos que não são `Copy`, implemente o operador para `&T`, para não consumir os operandos.
- `Index`/`IndexMut` devolvem referências e dão panic fora da faixa; ofereça um `get` também.
- Sobrecarregue só o que se parece com aritmética de verdade.

## Exercícios

Veja o exercício 1 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 20.2 Advanced Traits](https://doc.rust-lang.org/stable/book/ch20-02-advanced-traits.html) (seção *Default Generic Type Parameters and Operator Overloading*)
- [Documentação de `std::ops`](https://doc.rust-lang.org/std/ops/index.html)
- [Rust by Example: Operator Overloading](https://doc.rust-lang.org/rust-by-example/trait/ops.html)
- [Rust API Guidelines: C-OVERLOAD](https://rust-lang.github.io/api-guidelines/predictability.html#c-overload)

---

[← Índice do módulo](README.md) · [Próxima lição →](02-newtype-aliases-never-e-dsts.md)
