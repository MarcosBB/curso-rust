# 01 — `Vec` em profundidade

**Objetivos.** Ao final, você vai saber:

- controlar a capacidade de um `Vec` e saber quando isso importa;
- tirar elementos do jeito certo para cada caso (`remove`, `swap_remove`, `retain`, `drain`, `extract_if`, `dedup`...);
- ordenar por critérios compostos, e ordenar `f64`;
- buscar e percorrer pedaços de um vetor com `binary_search`, `windows` e `chunks`.

O [kit do Módulo 01](../01-tour-da-sintaxe/07-kit-vec-e-hashmap.md) deu o básico: `vec![]`, `push`, `get`, `sort` e o `for` com `&v` e `&mut v`. O [Módulo 02](../02-ownership/05-string-e-str.md) mostrou o layout: um `Vec` é um ponteiro para o *heap*, um tamanho (`len`) e uma capacidade (`capacity`). Esta lição parte daí.

## Capacidade: quando o `Vec` realoca

`len` é quantos elementos o vetor tem. `capacity` é quantos cabem no buffer atual sem realocar. Quando um `push` encontra o buffer cheio, o `Vec` aloca um maior, copia tudo para lá e libera o antigo:

```rust
let mut v = Vec::new();
let mut cap = v.capacity();
print!("capacidades: {cap}");
for i in 0..100 {
    v.push(i);
    if v.capacity() != cap {
        cap = v.capacity();
        print!(" {cap}");
    }
}
println!();
```

```text
capacidades: 0 4 8 16 32 64 128
```

A capacidade dobra a cada realocação, então 100 `push` custam só 6 alocações. Isso torna o `push` O(1) *amortizado*: caro de vez em quando, barato na média. A política exata não é garantida pela documentação, então não escreva código que dependa desses números.

Quando você sabe o tamanho de antemão, peça a capacidade logo de início com `Vec::with_capacity(n)` (ou `reserve(n)` num vetor que já existe). Assim, nenhuma realocação acontece no caminho:

```rust
let mut c: Vec<u8> = Vec::with_capacity(10);
c.extend([1, 2, 3]);
println!("len {} cap {}", c.len(), c.capacity());

c.clear();
println!("depois do clear: len {} cap {}", c.len(), c.capacity());
c.shrink_to_fit();
println!("depois do shrink_to_fit: cap {}", c.capacity());
```

```text
len 3 cap 10
depois do clear: len 0 cap 10
depois do shrink_to_fit: cap 0
```

Repare no `clear`: ele zera o `len`, mas mantém a memória. As coleções da std **nunca encolhem sozinhas**. Isso é bom quando você reaproveita o vetor num laço (limpa e enche de novo, sem alocar), e só vira problema se um vetor enorme ficar quase vazio por muito tempo. Nesse caso, `shrink_to_fit` devolve a sobra.

É o mesmo modelo do `std::vector` do C++ (`reserve`, `capacity`, `shrink_to_fit`). O `list` do Python também superaloca por baixo, mas não expõe nada disso.

## Tirar elementos

O [Módulo 02](../02-ownership/02-regras-de-ownership-e-move.md) mostrou que não dá para mover um elemento para fora com `let x = v[0];` (E0507), porque o vetor ficaria com um buraco. A saída é pedir ao `Vec` que **tire** o elemento, e ele se encarrega de fechar o buraco. Há um método para cada situação.

### Um elemento por posição

```rust
let mut v = vec!['a', 'b', 'c', 'd', 'e'];

let x = v.remove(1); // tira 'b' e desloca o resto: O(n)
println!("{x} {v:?}");

let y = v.swap_remove(0); // tira 'a' e põe o último no lugar: O(1)
println!("{y} {v:?}");

let z = v.pop(); // tira do fim: O(1)
println!("{z:?} {v:?}");
```

```text
b ['a', 'c', 'd', 'e']
a ['e', 'c', 'd']
Some('d') ['e', 'c']
```

- `remove(i)` preserva a ordem, mas desloca todos os elementos depois de `i`. Num vetor grande, chamado num laço, isso vira O(n²).
- `swap_remove(i)` é O(1), porque move o último elemento para o buraco. Use quando a ordem não importa.
- `pop()` tira do fim e devolve `Option`, porque o vetor pode estar vazio. `remove` e `swap_remove` dão *panic* se o índice não existir.

Para tirar um elemento **sem** mudar o tamanho do vetor, troque-o por outro valor. `std::mem::take` deixa no lugar o valor padrão do tipo (`Default`, do [Módulo 06](../06-traits-e-generics/README.md)):

```rust
let mut nomes = vec![String::from("Ana"), String::from("Bia")];
let primeiro = std::mem::take(&mut nomes[0]);
println!("{primeiro:?} {nomes:?}");
```

```text
"Ana" ["", "Bia"]
```

### Vários elementos de uma vez

```rust
let mut notas = vec![7, 3, 9, 5, 10, 2];
notas.retain(|&n| n >= 5); // mantém só os que passam no teste
println!("{notas:?}");

let altas: Vec<i32> = notas.extract_if(.., |n| *n >= 9).collect(); // tira e devolve
println!("{altas:?} {notas:?}");

let mut fila = vec![1, 2, 3, 4, 5, 6];
let primeiros: Vec<i32> = fila.drain(..2).collect(); // tira uma faixa e devolve
println!("{primeiros:?} {fila:?}");

let topo = fila.pop_if(|x| *x > 5); // pop só se o último passar no teste
println!("{topo:?} {fila:?}");

fila.truncate(1); // corta o fim
println!("{fila:?}");
```

```text
[7, 9, 5, 10]
[9, 10] [7, 5]
[1, 2] [3, 4, 5, 6]
Some(6) [3, 4, 5]
[3]
```

- `retain(pred)` mantém os elementos para os quais o predicado devolve `true` e remove o resto, numa passada só, O(n). É a resposta do Rust ao *iterator invalidation* que o [Módulo 03](../03-borrowing-e-lifetimes/02-borrow-checker-e-nll.md) mostrou: em vez de remover dentro do `for`, você diz ao vetor o que manter.
- `extract_if(faixa, pred)` (estável desde a 1.87) faz o contrário do `retain` e ainda devolve os removidos, como um iterador. O `..` é a faixa inteira.
- `drain(faixa)` remove uma faixa de posições e devolve os elementos.
- `pop_if(pred)` (1.86) e `truncate(n)` completam o kit.

O `|&n| n >= 5` é uma *closure*, a mesma do [kit do Módulo 01](../01-tour-da-sintaxe/08-kit-closures.md). O `&` no parâmetro e o `*n` no outro exemplo aparecem porque esses métodos passam uma referência para cada elemento. A lição 06 explica de onde vêm essas referências.

Em C++, o equivalente de `retain` é o *erase-remove idiom*, `v.erase(std::remove_if(v.begin(), v.end(), pred), v.end())`, ou `std::erase_if(v, pred)` no C++20. Atenção à lógica invertida: `erase_if` recebe o que **remover**, e `retain` recebe o que **manter**, como uma *list comprehension* do Python (`[n for n in notas if n >= 5]`).

### Duplicatas: `dedup`

```rust
let mut letras = vec!['b', 'a', 'b', 'b', 'a'];
letras.dedup();
println!("{letras:?}");
letras.sort();
letras.dedup();
println!("{letras:?}");
```

```text
['b', 'a', 'b', 'a']
['a', 'b']
```

`dedup` remove só as repetições **consecutivas**, como o `uniq` do Unix e o `std::unique` do C++. Para tirar todas as duplicatas, ordene antes. Se a ordem original precisa ser preservada, há um truque com `HashSet` na lição 03.

### Resumo dos custos

| Método | Custo | Ordem dos que ficam |
|---|---|---|
| `push`, `pop`, `pop_if` | O(1) amortizado | mantida |
| `insert(i, x)`, `remove(i)` | O(n) | mantida |
| `swap_remove(i)` | O(1) | **muda** |
| `retain`, `extract_if`, `dedup` | O(n), uma passada | mantida |
| `drain(a..b)` | O(n) | mantida |
| `truncate`, `clear` | O(1) para tipos sem `Drop`; O(removidos) com `Drop` | mantida |

## Ordenar

### `sort`, `sort_by_key` e `sort_by`

```rust
use std::cmp::Reverse;

let mut pessoas = vec![("Bia", 25), ("Ana", 31), ("Caio", 25), ("Davi", 31)];

pessoas.sort_by_key(|p| p.1);
println!("{pessoas:?}");

// idade decrescente; empate, nome crescente
pessoas.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
println!("{pessoas:?}");

// o mesmo critério com uma chave composta
pessoas.sort_by_key(|p| (Reverse(p.1), p.0));
println!("{pessoas:?}");
```

```text
[("Bia", 25), ("Caio", 25), ("Ana", 31), ("Davi", 31)]
[("Ana", 31), ("Davi", 31), ("Bia", 25), ("Caio", 25)]
[("Ana", 31), ("Davi", 31), ("Bia", 25), ("Caio", 25)]
```

- `sort_by_key(|x| chave)` ordena pela chave que a closure devolve, como o `key=` do Python.
- `sort_by(|a, b| ...)` recebe uma função de comparação que devolve um `Ordering` (`Less`, `Equal` ou `Greater`). Para inverter a ordem, troque `a` e `b`. Para desempatar, encadeie com `Ordering::then` (ou `then_with`, que recebe uma closure e só calcula o desempate quando precisa).
- Tuplas comparam campo a campo, e `std::cmp::Reverse` inverte a ordem de um campo. Por isso a chave `(Reverse(idade), nome)` faz o mesmo que o `sort_by`.

`sort` e `sort_by*` são **estáveis**: elementos com chaves iguais mantêm a ordem em que estavam. Na primeira linha da saída, Bia continua antes de Caio. Os `sort_unstable*` podem trocar a ordem dos empates, mas são mais rápidos e não alocam memória extra. Para números e outros tipos em que "igual" é indistinguível, prefira `sort_unstable`.

> [!WARNING]
> Os nomes são o contrário do C++. Lá, `std::sort` é instável e quem garante estabilidade é `std::stable_sort`. Em Rust, `sort` é o estável. O `list.sort()` do Python também é estável.

Se a chave é cara de calcular (um `to_lowercase()`, por exemplo), `sort_by_key` a recalcula a cada comparação, O(n log n) vezes. `sort_by_cached_key` calcula uma vez por elemento e guarda.

### `f64` não tem `sort`

```rust
// ❌ NÃO COMPILA
fn main() {
    let mut notas = vec![7.5, 9.0, 6.25];
    notas.sort();
    println!("{notas:?}");
}
```

```text
error[E0277]: the trait bound `{float}: Ord` is not satisfied
   --> src/main.rs:3:11
    |
  3 |     notas.sort();
    |           ^^^^ the trait `Ord` is not implemented for `{float}`
    |
...
note: required by a bound in `slice::<impl [T]>::sort`
...
133 |         T: Ord,
    |            ^^^ required by this bound in `slice::<impl [T]>::sort`
```

`sort` exige `Ord`, uma ordem **total**: para quaisquer `a` e `b`, um deles é menor ou os dois são iguais. Pontos flutuantes não têm isso, por causa do `NaN`: `NaN < 1.0`, `NaN > 1.0` e `NaN == NaN` são todos `false`. Por isso `f64` só implementa `PartialOrd` (o [Módulo 06](../06-traits-e-generics/README.md) apresenta essas traits). Em C++, ordenar um `vector<double>` com `NaN` compila e quebra a pré-condição do `std::sort`, o que é comportamento indefinido. O Rust obriga você a decidir o que fazer com o `NaN`.

A decisão mais comum é `total_cmp`, que define uma ordem total (a do padrão IEEE 754) e põe o `NaN` positivo no fim:

```rust
// ✅
let mut notas = vec![7.5, f64::NAN, 9.0, 6.25];
notas.sort_by(|a, b| a.total_cmp(b));
println!("{notas:?}");

let validas = [7.5, 9.0, 6.25];
let maior = validas.iter().copied().fold(f64::NEG_INFINITY, f64::max);
let menor = validas.iter().min_by(|a, b| a.total_cmp(b));
println!("{maior} {menor:?}");
```

```text
[6.25, 7.5, 9.0, NaN]
9 Some(6.25)
```

O mesmo vale para `max` e `min` de iteradores (lição 07): com `f64`, use `max_by`/`min_by` com `total_cmp`, ou um `fold` com `f64::max`.

## Buscar

`contains(&x)` e `iter().position(|x| ...)` fazem busca linear. Num vetor **ordenado**, `binary_search` é O(log n) e devolve um `Result` com uma informação a mais:

```rust
let mut v = vec![10, 20, 30, 40];
println!("{:?} {:?}", v.binary_search(&20), v.binary_search(&25));

// inserir mantendo a ordem: o Err diz onde
let pos = v.binary_search(&25).unwrap_or_else(|i| i);
v.insert(pos, 25);
println!("{v:?}");

println!("{} {:?}", v.contains(&30), v.iter().position(|&x| x > 22));
```

```text
Ok(1) Err(2)
[10, 20, 25, 30, 40]
true Some(2)
```

`Ok(i)` diz que achou na posição `i`. `Err(i)` diz que não achou e que `i` é onde o valor deveria entrar para manter a ordem. É o `bisect_left` do Python e o `std::lower_bound` do C++ no mesmo método. Num vetor fora de ordem, o resultado não tem significado (não dá erro, só responde errado).

## Pedaços: `windows`, `chunks` e companhia

Muitos métodos que você usa num `Vec` são, na verdade, métodos de *slice* (`[T]`). Funcionam no `Vec` porque ele se converte automaticamente em `&[T]` quando preciso (a *deref coercion*, apresentada no [Módulo 06](../06-traits-e-generics/README.md)). Os mais úteis para percorrer pedaços:

```rust
let temps = vec![18, 21, 19, 25, 24];

let variacoes: Vec<i32> = temps.windows(2).map(|w| w[1] - w[0]).collect();
let mesmas: Vec<i32> = temps.array_windows().map(|[a, b]| b - a).collect();
println!("{variacoes:?} {mesmas:?}");

for bloco in temps.chunks(2) {
    print!("{bloco:?} ");
}
println!();

let (manha, tarde) = temps.split_at(2);
println!("{manha:?} {tarde:?} {:?} {:?}", temps.first(), temps.last());

let partes = vec![vec![1, 2], vec![3]];
let palavras = vec!["a", "b", "c"];
println!("{:?} {}", partes.concat(), palavras.join("-"));
```

```text
[3, -2, 6, -1] [3, -2, 6, -1]
[18, 21] [19, 25] [24]
[18, 21] [19, 25, 24] Some(18) Some(24)
[1, 2, 3] a-b-c
```

- `windows(n)`: janelas sobrepostas de tamanho `n` (cada uma é um `&[T]`).
- `array_windows()` (1.94): o mesmo, mas cada janela é um array `&[T; N]`, com `N` deduzido do uso. Isso permite desestruturar no parâmetro (`|[a, b]|`) e dispensa a checagem de índice do `w[1]`.
- `chunks(n)`: blocos sem sobreposição. O último pode ser menor.
- `split_at(i)`: duas fatias, antes e depois de `i`.
- `first`/`last`: `Option`, porque o vetor pode estar vazio.
- `concat` e `join`: achatam um vetor de vetores (ou de strings).

Todos devolvem **referências para dentro do vetor**, sem copiar nada. São as *slices* do [Módulo 03](../03-borrowing-e-lifetimes/03-slices-em-profundidade.md).

> [!NOTE]
> Desde a 1.95, `v.push_mut(x)` faz o `push` e devolve um `&mut` para o elemento recém-inserido. Útil quando você quer inserir e logo em seguida ajustar o elemento sem procurar por ele de novo.

## Armadilhas comuns

**`sort_by_key` devolvendo uma referência.** Ordenar pelo nome parece pedir `|p| &p.nome`, para não copiar a `String`:

```rust
// ❌ NÃO COMPILA
struct Pessoa {
    nome: String,
    idade: u32,
}

fn main() {
    let mut v = vec![
        Pessoa { nome: String::from("Bia"), idade: 25 },
        Pessoa { nome: String::from("Ana"), idade: 31 },
    ];
    v.sort_by_key(|p| &p.nome);
    println!("{}", v[0].idade);
}
```

```text
error: lifetime may not live long enough
  --> src/main.rs:11:23
   |
11 |     v.sort_by_key(|p| &p.nome);
   |                    -- ^^^^^^^ returning this value requires that `'1` must outlive `'2`
   |                    ||
   |                    |return type of closure is &'2 String
   |                    has type `&'1 Pessoa`
```

A assinatura é `sort_by_key<K, F>(&mut self, f: F) where F: FnMut(&T) -> K`. O tipo `K` da chave é um só para todas as chamadas, e por isso não pode carregar o *lifetime* do empréstimo de cada elemento (o `'1` da mensagem, diferente a cada chamada). Não é um defeito do seu código, é uma limitação da assinatura. Use `sort_by`, que compara duas referências sem guardar nada:

```rust
// ✅
v.sort_by(|a, b| a.nome.cmp(&b.nome));
// ou, se a chave precisa ser calculada:
v.sort_by_cached_key(|p| p.nome.to_lowercase());
```

**Usar `dedup` sem ordenar antes.** Não dá erro, só deixa duplicatas que não são vizinhas.

**Inverter a lógica do `retain`.** O predicado diz o que **fica**. Quem vem do `std::erase_if` costuma escrever o contrário.

**`remove(0)` num laço para consumir uma fila.** Cada chamada desloca o vetor inteiro. Para tirar da frente com frequência, use `VecDeque` (lição 03), ou inverta a ordem e use `pop`.

**`binary_search` em vetor não ordenado.** Compila e devolve um resultado sem sentido.

## Resumo

- `len` é o tamanho, `capacity` o espaço reservado. Use `with_capacity`/`reserve` quando souber o tamanho. Nada encolhe sozinho.
- Para tirar: `pop`/`remove`/`swap_remove` (um), `retain`/`extract_if`/`drain`/`dedup` (vários). `retain` é o jeito seguro de "remover enquanto percorre".
- `sort` é estável e `sort_unstable` não. Critérios compostos com `sort_by` + `then`, ou com uma tupla de chaves e `Reverse`.
- `f64` não é `Ord`: ordene com `sort_by(|a, b| a.total_cmp(b))`.
- `binary_search` devolve `Ok(posição)` ou `Err(onde inserir)`.
- `windows`, `array_windows`, `chunks` e `split_at` são métodos de slice que emprestam pedaços do vetor.

## Exercícios

Veja os exercícios 2 e 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 8.1 Storing Lists of Values with Vectors](https://doc.rust-lang.org/stable/book/ch08-01-vectors.html)
- [Documentação de `Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) (seção *Capacity and reallocation*) e [dos métodos de slice](https://doc.rust-lang.org/std/primitive.slice.html)
- [Documentação de `f64::total_cmp`](https://doc.rust-lang.org/std/primitive.f64.html#method.total_cmp)
- [Documentação de `std::collections`](https://doc.rust-lang.org/std/collections/index.html) (custos de cada operação)

---

[Índice do módulo](README.md) · [Próxima lição →](02-hashmap-e-entry-api.md)
