# 07 — Paralelismo de dados com `rayon`

**Objetivos.** Ao final, você vai saber:

- paralelizar uma cadeia de iteradores trocando `iter()` por `par_iter()`;
- por que o `rayon` não deixa a closure alterar variáveis de fora, e o que usar no lugar (`sum`, `fold`/`reduce`, `collect`);
- medir se o paralelismo compensou, e reconhecer quando ele não compensa;
- quando escolher `rayon`, threads com `scope` ou canais.

## O problema: dividir o trabalho dá trabalho

A `soma_paralela` da lição 01 dividia um slice em pedaços, criava uma thread por pedaço e juntava os resultados. Funciona, mas você teve de escolher quantos pedaços, e se um pedaço demorar mais que os outros (números grandes são mais lentos de testar), as outras threads ficam paradas esperando.

O *paralelismo de dados* é o caso em que a mesma operação se aplica a muitos itens independentes. A crate [`rayon`](https://docs.rs/rayon) resolve esse caso com iteradores paralelos: você escreve a cadeia de `map`/`filter`/`sum` do Módulo 07, e o rayon divide o trabalho entre um conjunto fixo de threads (um *thread pool*, uma por núcleo), redistribuindo os pedaços enquanto roda (*work stealing*: uma thread ociosa "rouba" trabalho de outra).

```toml
[dependencies]
rayon = "1.12"
```

## `par_iter`: a troca de uma palavra

```rust
use rayon::prelude::*;

fn eh_primo(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    let mut d = 2;
    while d * d <= n {
        if n.is_multiple_of(d) {
            return false;
        }
        d += 1;
    }
    true
}

fn contar_primos_seq(limite: u64) -> usize {
    (0..limite).filter(|&n| eh_primo(n)).count()
}

fn contar_primos_par(limite: u64) -> usize {
    (0..limite).into_par_iter().filter(|&n| eh_primo(n)).count()
}
```

A única diferença é o `.into_par_iter()`. O `use rayon::prelude::*` traz as traits que acrescentam os métodos paralelos:

| Sequencial | Paralelo |
|---|---|
| `v.iter()` | `v.par_iter()` |
| `v.iter_mut()` | `v.par_iter_mut()` |
| `v.into_iter()`, `(a..b)` | `v.into_par_iter()`, `(a..b).into_par_iter()` |
| `v.chunks(n)` | `v.par_chunks(n)` |
| `v.sort()`, `v.sort_unstable()` | `v.par_sort()`, `v.par_sort_unstable()` |

Depois disso, a cadeia continua com os nomes que você conhece: `map`, `filter`, `sum`, `count`, `min`, `max_by_key`, `any`, `collect`...

Medindo em `--release`, numa máquina com 10 núcleos (código completo em [`codigo/rayon-basico`](codigo/rayon-basico/src/main.rs)):

```text
threads do rayon: 10
  sequencial:  290.3ms
    paralelo:   42.9ms
216816 primos abaixo de 3000000
```

Cerca de 7 vezes mais rápido com 10 núcleos. Não chega a 10 porque há custo para dividir e juntar, e porque nem todos os núcleos de um processador são iguais. Os números vão ser outros na sua máquina, e mudam um pouco a cada execução.

> [!WARNING]
> Meça sempre com `cargo run --release`. Em modo debug, o código não é otimizado, e a comparação entre sequencial e paralelo não diz nada sobre o programa de verdade.

## A ordem e o resultado

Os itens são processados fora de ordem, em threads diferentes. Mas as operações que produzem um resultado preservam o que você espera:

- `collect()` num `Vec` mantém a **ordem original** dos itens;
- `sum`, `count`, `min`, `max` dão o mesmo resultado que a versão sequencial;
- `for_each(|x| println!(...))` imprime numa ordem que varia a cada execução.

Os testes de [`codigo/rayon-basico`](codigo/rayon-basico/src/main.rs) comparam as duas versões. É um bom hábito: escreva a sequencial, teste, troque para paralela e confira que dá o mesmo.

## A closure não pode alterar o que está fora

O reflexo de quem vem de um laço `for` é acumular numa variável:

```rust
// ❌ NÃO COMPILA
use rayon::prelude::*;

fn main() {
    let v: Vec<u64> = (1..=100).collect();
    let mut total = 0;
    v.par_iter().for_each(|x| total += x);
    println!("{total}");
}
```

```text
error[E0596]: cannot borrow `total` as mutable, as it is a captured variable in a `Fn` closure
 --> src/main.rs:6:31
  |
5 |     let mut total = 0;
  |         --------- `total` declared here, outside the closure
6 |     v.par_iter().for_each(|x| total += x);
  |                  -------------^^^^^------
  |                  |        |   |
  |                  |        |   cannot borrow as mutable
  |                  |        in this closure
  |                  expects `Fn` instead of `FnMut`
```

O `for_each` do rayon pede uma closure `Fn + Send + Sync`, não `FnMut`: a mesma closure roda em várias threads **ao mesmo tempo**, e um `FnMut` precisaria de `&mut total` em todas elas. É a regra das lições anteriores (um `&mut` só), aplicada pelas traits de closure do Módulo 07. Sem o rayon, isso seria a data race da lição 02.

A solução quase nunca é um `Mutex` ou um atômico. É deixar o rayon combinar os resultados:

```rust
let total: u64 = v.par_iter().sum();
```

### `fold` e `reduce`: acumular algo maior

E se o resultado for um `HashMap` de frequências? O par `fold` + `reduce` faz isso sem compartilhar nada:

```rust
fn frequencias(linhas: &[String]) -> HashMap<String, usize> {
    linhas
        .par_iter()
        .fold(HashMap::new, |mut mapa, linha| {
            for p in linha.split_whitespace() {
                *mapa.entry(p.to_string()).or_insert(0) += 1;
            }
            mapa
        })
        .reduce(HashMap::new, |mut a, b| {
            for (p, n) in b {
                *a.entry(p).or_insert(0) += n;
            }
            a
        })
}
```

- `fold(inicial, f)`: cada pedaço do trabalho começa com um acumulador novo (`HashMap::new`, uma função, porque cada pedaço precisa do seu) e o vai preenchendo. O resultado é um iterador paralelo de mapas parciais.
- `reduce(inicial, f)`: junta os mapas parciais dois a dois, até sobrar um.

É a mesma ideia da `contar_palavras` com `Mutex` da lição 04 (contar localmente, juntar no fim), só que sem lock nenhum: cada mapa parcial tem um dono só.

## Quando o paralelismo não compensa

Paralelizar tem custo: dividir o trabalho, acordar threads, juntar os resultados. Ele compensa quando há **muito** trabalho por item ou **muitos** itens:

- somar um `Vec` de 1000 números: o sequencial costuma ganhar (a soma inteira leva menos tempo que dividir o trabalho);
- testar 3 milhões de números para primalidade: o paralelo ganha com folga;
- um laço que faz E/S (ler arquivos, rede): o gargalo é o disco ou a rede, não a CPU. Threads comuns ou async (Módulo 12) servem melhor.

E nem toda tarefa é paralela: se cada passo depende do anterior (um saldo acumulado, por exemplo), não há o que dividir.

O rayon cria por padrão uma thread por núcleo (`rayon::current_num_threads()`). Para mudar, defina a variável de ambiente `RAYON_NUM_THREADS`, útil para medir como o programa escala:

```bash
RAYON_NUM_THREADS=1 cargo run --release
```

## Qual ferramenta usar

| Situação | Ferramenta |
|---|---|
| a mesma operação em muitos itens de uma coleção | `rayon` (`par_iter`) |
| algumas tarefas diferentes sobre dados locais, esperar todas | `thread::scope` (lição 01) |
| etapas que passam dados adiante, produtor e consumidor | canais (lição 03) |
| uma thread de fundo que vive o programa todo | `thread::spawn` |
| muitas esperas por E/S (rede, milhares de conexões) | async (Módulo 12) |

## E o Python?

Em Python, cálculo pesado em paralelo pede `multiprocessing` (ou `concurrent.futures.ProcessPoolExecutor`): processos separados, cada um com o seu interpretador. Os dados são serializados com `pickle` para ir de um processo a outro, e isso custa caro para estruturas grandes. Bibliotecas como NumPy contornam o problema fazendo o trabalho em C, fora do GIL.

O rayon roda tudo no mesmo processo, sobre os mesmos dados, sem cópias: `par_iter()` empresta o `Vec`. E a segurança vem do mesmo lugar de sempre: se a closure capturar um `Rc` ou um `RefCell`, o erro de `Send`/`Sync` da lição 02 aparece na compilação.

> [!NOTE]
> O [Projeto 4](../../projetos/04-processador-paralelo/README.md), que vem depois deste módulo, junta o rayon com threads e canais para processar muitos arquivos em paralelo.

## Armadilhas comuns

- **Medir em debug.** Os números não valem nada. Use `--release`.
- **`for_each` com acumulador externo.** Dá E0596. Use `sum`, `count`, `collect` ou `fold`/`reduce`.
- **Trocar para um `Mutex` para calar o erro.** Compila, mas todas as threads disputam o mesmo lock, e o programa pode ficar mais lento que o sequencial.
- **Paralelizar trabalho pequeno.** Com poucos itens e trabalho trivial, o custo de dividir supera o ganho. Meça antes de trocar.
- **Contar com a ordem de efeitos colaterais.** `for_each` com `println!` imprime em qualquer ordem. Para resultados ordenados, use `collect`.

## Resumo

- `rayon` paraleliza cadeias de iteradores: `iter()` → `par_iter()`, `into_iter()` → `into_par_iter()`, `sort` → `par_sort`.
- Um *thread pool* com uma thread por núcleo e *work stealing* divide o trabalho sozinho.
- `collect` preserva a ordem; `sum`/`count`/`min`/`max` dão o mesmo resultado; `for_each` não tem ordem.
- As closures precisam ser `Fn + Send + Sync`: nada de alterar variáveis de fora. Use `sum`, `collect` ou `fold` + `reduce`.
- Meça em `--release`. Paralelismo compensa com muito trabalho por item ou muitos itens, e não ajuda em E/S.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação do `rayon`](https://docs.rs/rayon/latest/rayon/) (veja o módulo `rayon::iter` e o [FAQ](https://github.com/rayon-rs/rayon/blob/main/FAQ.md))
- [Comprehensive Rust: Concurrency](https://google.github.io/comprehensive-rust/concurrency/welcome.html)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/) (para medir antes de otimizar; voltamos a ele no Módulo 18)

---

[← Lição anterior](06-estado-global.md) · [Índice do módulo](README.md) · [Próximo: exercícios →](exercicios.md)
