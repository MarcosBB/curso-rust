# 07 — Kit: `Vec` e `HashMap`

> [!NOTE]
> As lições 07 a 10 formam o **kit mínimo**: só o necessário para fazer os exercícios e o Projeto 1. Elas são uma receita, e de propósito deixam vários "porquês" para depois. O aprofundamento vem nos Módulos 07 (coleções, iteradores e closures), 08 (módulos e crates) e 09 (testes).

**Objetivos.** Ao final, você vai saber:

- criar um `Vec`, acrescentar elementos e percorrê-lo para ler ou alterar;
- acessar elementos com segurança usando `get` e lidar com o `Option` que ele devolve;
- criar um `HashMap`, inserir, consultar e percorrer;
- contar ocorrências com `entry().or_insert()`.

## `Vec<T>`: a lista que cresce

`Vec<T>` é a lista dinâmica, equivalente ao `list` do Python e ao `std::vector` do C++. Todos os elementos têm o mesmo tipo `T`.

```rust
let mut numeros: Vec<i32> = Vec::new();
numeros.push(10);
numeros.push(20);

let mut outros = vec![3, 1, 2];   // a macro vec! já infere Vec<i32>
outros.sort();
println!("{numeros:?} {outros:?}"); // [10, 20] [1, 2, 3]
```

Dois erros aparecem logo de cara. O primeiro: `Vec::new()` sozinho não diz o tipo dos elementos. Se nada no código disser, o compilador não adivinha:

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = Vec::new();
    println!("{}", v.len());
}
```

```text
error[E0282]: type annotations needed for `Vec<_>`
 --> src/main.rs:2:9
  |
2 |     let v = Vec::new();
  |         ^   ---------- type must be known at this point
  |
help: consider giving `v` an explicit type, where the type for type parameter `T` is specified
  |
2 |     let v: Vec<T> = Vec::new();
  |          ++++++++
```

Basta anotar (`let v: Vec<i32> = Vec::new();`) ou acrescentar um elemento depois, que o compilador deduz o tipo a partir dele.

O segundo: `push` altera o vetor, e então a variável precisa de `mut`:

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = Vec::new();
    v.push(1);
    println!("{v:?}");
}
```

```text
error[E0596]: cannot borrow `v` as mutable, as it is not declared as mutable
 --> src/main.rs:3:5
  |
3 |     v.push(1);
  |     ^ cannot borrow as mutable
  |
help: consider changing this to be mutable
  |
2 |     let mut v = Vec::new();
  |         +++
```

É a primeira vez que aparece a palavra *borrow* (empréstimo): para alterar o vetor, o `push` precisa de um empréstimo mutável dele. O [Módulo 03](../03-borrowing-e-lifetimes/README.md) explica o que isso significa. Por enquanto, a correção é a do `help:`.

Equivalências úteis:

| Python | Rust |
|---|---|
| `lst = []` | `let mut v = Vec::new();` |
| `lst = [1, 2, 3]` | `let mut v = vec![1, 2, 3];` |
| `lst.append(x)` | `v.push(x);` |
| `lst.pop()` | `v.pop()` (devolve `Option`) |
| `len(lst)` | `v.len()` |
| `x in lst` | `v.contains(&x)` |
| `lst.sort()` | `v.sort();` |
| `if not lst:` | `if v.is_empty() {` |

## Percorrer: `&v` para ler, `&mut v` para alterar

```rust
let mut numeros = vec![10, 20];

for n in &numeros {        // lê cada elemento
    println!("lendo {n}");
}

for n in &mut numeros {    // altera cada elemento
    *n += 1;
}
println!("{numeros:?}");   // [11, 21]
```

Guarde a receita:

- `for x in &v`: lê, e o vetor continua disponível depois.
- `for x in &mut v`: altera. O `*x` é necessário para mexer no valor para o qual `x` aponta.
- `for x in v` (sem `&`): **consome** o vetor, que não pode mais ser usado depois do laço. O porquê é o assunto do [Módulo 02](../02-ownership/README.md). Na dúvida, use `&v`.

## Acessar: `v[i]` ou `v.get(i)`

`v[i]` funciona como no array da [lição 05](05-tuplas-arrays-e-slices.md): dá *panic* se `i` estiver fora dos limites. Quando o índice pode não existir, use `get`, que **devolve um `Option`**:

```rust
let numeros = vec![11, 21];
println!("{:?}", numeros.get(0));   // Some(11)
println!("{:?}", numeros.get(99));  // None
```

### Prévia: `Option`

`Option` é o jeito do Rust de dizer "pode haver um valor, ou não". Um `Option` vale `Some(valor)` ou `None`, e o compilador não deixa você usar o valor sem antes tratar o caso `None`. Não existe `null`. O tipo completo é assunto do [Módulo 04](../04-structs-enums-e-patterns/README.md). Para os exercícios, três receitas bastam:

```rust
let numeros = vec![11, 21];

// 1. match: trata os dois casos
match numeros.get(99) {
    Some(n) => println!("achei {n}"),
    None => println!("não tem posição 99"),
}

// 2. valor padrão quando for None
let n = numeros.get(99).copied().unwrap_or(0);   // 0

// 3. comparar direto num teste
assert_eq!(numeros.get(0), Some(&11));
```

Repare no `&`: `get` devolve um `Option<&i32>`, uma referência para o elemento que continua dentro do vetor. O `.copied()` transforma em `Option<i32>`, copiando o número, e então o `unwrap_or(0)` funciona com um `i32` comum. Em Python, `lst[99]` lança `IndexError` e `d.get(k)` devolve `None` silenciosamente. Em Rust, o `Option` deixa a ausência **no tipo**, e você é obrigado a lidar com ela.

## `HashMap<K, V>`: o dicionário

```rust
use std::collections::HashMap;

let mut idades: HashMap<String, u32> = HashMap::new();
idades.insert(String::from("Ana"), 31);
idades.insert(String::from("Bruno"), 25);
idades.insert(String::from("Ana"), 32);   // substitui o valor anterior

println!("{:?}", idades.get("Ana"));      // Some(32)
println!("{:?}", idades.get("Zé"));       // None
println!("{}", idades.contains_key("Bruno")); // true

for (nome, idade) in &idades {
    println!("{nome}: {idade}");
}
```

- `HashMap` **não está no *prelude*** (o conjunto de nomes que todo arquivo enxerga sem importar), ao contrário de `Vec`. Você precisa do `use std::collections::HashMap;`.
- `get` devolve `Option<&V>`, como no `Vec`.
- A **ordem do `for` é arbitrária** e muda de uma execução para outra: a função de *hash* padrão (SipHash) usa uma semente aleatória, para resistir a ataques que forçam colisões. Se precisar de ordem, copie os pares para um `Vec` e ordene (ou use um `BTreeMap`, no Módulo 07).
- `String::from("Ana")` cria uma `String`, o texto que pertence ao mapa. Ao inserir, o mapa fica dono da chave. O [Módulo 02](../02-ownership/README.md) explica isso direito.

Sem o `use`, o erro já sugere a correção:

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut m = HashMap::new();
    m.insert("a", 1);
}
```

```text
error[E0433]: cannot find type `HashMap` in this scope
 --> src/main.rs:2:17
  |
2 |     let mut m = HashMap::new();
  |                 ^^^^^^^ use of undeclared type `HashMap`
  |
help: consider importing this struct
  |
1 + use std::collections::HashMap;
  |
```

### A receita da contagem

Contar ocorrências é tão comum que vale decorar esta linha:

```rust
use std::collections::HashMap;

let texto = "a b a c b a";
let mut contagem: HashMap<&str, usize> = HashMap::new();
for palavra in texto.split_whitespace() {
    *contagem.entry(palavra).or_insert(0) += 1;
}
println!("{:?}", contagem.get("a"));   // Some(3)
```

Lendo da esquerda para a direita: `entry(palavra)` encontra a posição da chave no mapa, `or_insert(0)` insere 0 se a chave ainda não existir, e o `*... += 1` soma 1 ao valor que está lá dentro. É o `defaultdict(int)` (ou o `Counter`) do Python. Para valores que são listas, a mesma ideia funciona com `.or_default()`, que insere um `Vec` vazio: veja o `agrupar` em [`codigo/boletim-escolar`](codigo/boletim-escolar/src/lib.rs). A *entry API* completa fica no Módulo 07.

Para mostrar o resultado em ordem, copie para um `Vec` e ordene:

```rust
let mut pares: Vec<_> = contagem.into_iter().collect();
pares.sort();
println!("{pares:?}"); // [("a", 3), ("b", 2), ("c", 1)]
```

O `Vec<_>` pede ao compilador que deduza o tipo dos elementos. O `into_iter().collect()` transforma o mapa num vetor de pares. Iteradores são o assunto do Módulo 07.

## Armadilhas comuns

- **`Vec::new()` sem tipo** (E0282): anote o tipo ou deixe que um `push` posterior defina.
- **`push` sem `mut`** (E0596): declare com `let mut`.
- **`HashMap` sem `use`** (E0433): acrescente `use std::collections::HashMap;`.
- **`v[i]` com índice que pode não existir**: use `v.get(i)` e trate o `None`.
- **Esperar ordem no `HashMap`**: a ordem muda a cada execução. Ordene um `Vec` de pares quando a ordem importar (nos testes também).
- **`for x in v` e depois usar `v`**: use `for x in &v`. O erro que aparece sem o `&` é o assunto do Módulo 02.

## Resumo

- `Vec<T>`: `Vec::new()` ou `vec![...]`, `push` (exige `mut`), `len`, `sort`, `contains`.
- Percorra com `for x in &v` (ler) ou `for x in &mut v` (alterar, com `*x`).
- `v.get(i)` e `m.get(&k)` devolvem `Option`: `Some(valor)` ou `None`. Trate com `match`, `unwrap_or` ou compare num teste.
- `HashMap` precisa de `use std::collections::HashMap;`, e a ordem do `for` é arbitrária.
- Contagem: `*mapa.entry(chave).or_insert(0) += 1;`.

## Exercícios

Veja os exercícios 5 e 6 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 8.1 Vectors](https://doc.rust-lang.org/stable/book/ch08-01-vectors.html)
- [The Rust Programming Language, 8.3 Hash Maps](https://doc.rust-lang.org/stable/book/ch08-03-hash-maps.html)
- [std: `Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) e [`HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)

---

[← Lição anterior](06-macros-e-formatacao.md) · [Índice do módulo](README.md) · [Próxima lição →](08-kit-closures.md)
