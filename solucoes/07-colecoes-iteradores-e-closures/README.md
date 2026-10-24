# Soluções — Módulo 07

Enunciados em [`modulos/07-colecoes-iteradores-e-closures/exercicios.md`](../../modulos/07-colecoes-iteradores-e-closures/exercicios.md).

Cada solução é um projeto Cargo com os mesmos testes do enunciado. Para conferir:

```bash
cd ex1-trio-de-closures
cargo test
```

## 1. Faça compilar: o trio das closures

Projeto: [`ex1-trio-de-closures/`](ex1-trio-de-closures/src/lib.rs).

| Função | Erro | Correção | Trait depois |
|---|---|---|---|
| `multiplicador` | E0373: a closure devolvida guardaria `&k` | `move \|x\| x * k` | `Fn`: só lê `k` |
| `conta_vogais` | E0596: chamar a closure exige `&mut` nela | `let mut conta = ...` | `FnMut`: altera `total` |
| `saudacao_dupla` | E0525: `let n = nome;` move para fora, e a closure vira só `FnOnce` | `move \|\| format!("oi, {nome}! ")` | `Fn`: só lê `nome` |

O (c) é o que mais ensina. O `move` **continua** lá: a closure é dona de `nome` (a função recebeu a `String` por valor, e a closure precisa sobreviver a ela dentro de `chamar_duas_vezes`). O que muda é o corpo: em vez de entregar `nome` a uma variável local, ele só o lê. `move` decide como a closure captura; o corpo decide qual trait ela implementa.

Outra correção válida seria `let n = nome.clone();` dentro do corpo (o segundo `help:` do E0507, se a closure estivesse escrita direto na chamada). Funciona, mas clona a cada chamada sem necessidade. Mudar o bound para `FnOnce` não serve: `chamar_duas_vezes` chama `f` duas vezes.

## 2. Traduzindo comprehensions

Projeto: [`ex2-comprehensions/`](ex2-comprehensions/src/lib.rs).

```rust
pub fn quadrados_pares(xs: &[i32]) -> Vec<i32> {
    xs.iter().filter(|&&x| x % 2 == 0).map(|x| x * x).collect()
}

pub fn pares(n: u32) -> Vec<(u32, u32)> {
    (0..n).flat_map(|i| (i + 1..n).map(move |j| (i, j))).collect()
}
```

- O `|&&x|` no `filter` é o da lição 06: `iter()` dá `&i32`, e o predicado recebe `&&i32`. No `map`, `x * x` funciona direto com `&i32`.
- Em `pares`, sem o `move`, a closure interna guardaria `&i`, e `i` é um parâmetro da closure externa que morre a cada volta. O compilador recusa com E0373.
- `tamanhos` usa `p.chars().count()`, e não `p.len()`: o teste com `"maçã"` (4 caracteres, 6 bytes) pega quem esquece. É a armadilha de UTF-8 do [Módulo 02](../../modulos/02-ownership/06-strings-utf8.md).
- `letras_unicas` usa `flat_map(char::to_lowercase)`, porque a conversão de um `char` para minúsculas pode dar mais de um `char` (o `'İ'` turco vira dois). O `BTreeSet` deixa a saída ordenada para o teste.
- `top3` conta com `*contagem.entry(p.to_lowercase()).or_insert(0) += 1`, coleta em `Vec` e ordena com `b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))`. O `Counter.most_common` do Python desempata pela ordem de inserção (o `Counter` é um `dict`); o `HashMap` não tem ordem, por isso o enunciado define o desempate alfabético.

## 3. Índice invertido

Projeto: [`ex3-indice-invertido/`](ex3-indice-invertido/src/lib.rs).

O índice é a *Entry API* com `or_default`, num `BTreeMap` de `BTreeSet`:

```rust
for (i, linha) in texto.lines().enumerate() {
    for bruta in linha.split_whitespace() {
        let palavra = bruta.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if palavra.is_empty() {
            continue;
        }
        idx.entry(palavra).or_default().insert(i + 1);
    }
}
```

Com `BTreeMap` e `BTreeSet`, as chaves e as linhas saem em ordem, sem ordenar nada à mão. Um `HashMap<String, Vec<usize>>` também funcionaria, mas uma palavra repetida na mesma linha entraria duas vezes no `Vec`.

A busca começa com uma cópia do conjunto da primeira palavra e usa `retain` com os outros:

```rust
let mut conjuntos = palavras.iter().map(|p| idx.get(&p.to_lowercase()));
let Some(Some(primeiro)) = conjuntos.next() else {
    return BTreeSet::new();
};
let mut resultado = primeiro.clone();
for c in conjuntos {
    match c {
        Some(c) => resultado.retain(|linha| c.contains(linha)),
        None => return BTreeSet::new(),
    }
}
```

O `let ... else` cobre os dois casos vazios de uma vez: sem palavras (`None`) e primeira palavra fora do índice (`Some(None)`). `intersection` também resolveria (`resultado = resultado.intersection(c).copied().collect()`), criando um conjunto novo a cada palavra.

## 4. Implemente dois iteradores

Projeto: [`ex4-iteradores/`](ex4-iteradores/src/lib.rs).

```rust
impl Iterator for Collatz {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let atual = self.proximo?;
        self.proximo = match atual {
            1 => None,
            x if x % 2 == 0 => Some(x / 2),
            x => Some(3 * x + 1),
        };
        Some(atual)
    }
}
```

Guardar o **próximo** como `Option` resolve os casos de borda: `collatz(0)` começa com `None` (`(n > 0).then_some(n)`), e o 1 é devolvido antes de o próximo virar `None`. O `?` encerra o iterador.

```rust
impl<'a, T> Iterator for Pares<'a, T> {
    type Item = (&'a T, &'a T);

    fn next(&mut self) -> Option<(&'a T, &'a T)> {
        let [a, b, ..] = self.resto else {
            return None;
        };
        self.resto = &self.resto[1..];
        Some((a, b))
    }
}
```

O ponto importante é o `'a` no `Item`: as referências apontam para o slice original, e não para o iterador. Por isso o teste `pares_com_strings_nao_copia` consegue guardar o resultado de `max_by_key` e comparar ponteiros. O padrão de slice `[a, b, ..]` só casa com dois ou mais elementos, e já entrega `a` e `b` como `&T`.

O bônus com `successors` é o mesmo `match` dentro da closure, sem struct. O teste usa `Iterator::eq`, que compara duas sequências item a item. Para uma sequência "cada termo a partir do anterior", `successors` é mais curto; a struct vale a pena quando o estado tem mais de um campo ou quando você quer dar um nome ao tipo.

## 5. Closures guardadas

Projeto: [`ex5-closures-guardadas/`](ex5-closures-guardadas/src/lib.rs).

```rust
pub struct Pipeline {
    etapas: Vec<Box<dyn Fn(i32) -> i32>>,
}

pub fn aplicar(&self, x: i32) -> i32 {
    self.etapas.iter().fold(x, |acc, f| f(acc))
}

pub fn filtrar<'a>(&self, linhas: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    linhas.into_iter().filter(|l| (self.pred)(l)).collect()
}
```

**As perguntas.**

- `Pipeline<F>` com `Vec<F>` exigiria que **todas** as etapas tivessem o mesmo tipo `F`. Como cada closure tem um tipo próprio, só daria para usar uma closure (ou várias cópias da mesma expressão). `Box<dyn Fn>` apaga o tipo concreto, e closures diferentes cabem no mesmo `Vec`. O preço é uma alocação por etapa e uma chamada indireta.
- `Filtro` guarda um predicado só, conhecido na criação: o genérico não tem custo extra e permite que o predicado capture referências (`&nivel` no teste). O `Pipeline` exige `'static` porque `Box<dyn Fn>` sem anotação de lifetime significa `Box<dyn Fn + 'static>` (o [Módulo 06](../../modulos/06-traits-e-generics/README.md) explica o `'static` implícito), e por isso as etapas usam `move`.
- O bound é `Fn` porque `filtrar` recebe `&self`: através de um `&`, não dá para chamar uma `FnMut`. Se o filtro precisasse de estado mutável (contar quantas linhas passaram, por exemplo), `filtrar` teria de receber `&mut self`.

No `filter`, o predicado recebe `&&str` (a referência para o item, lição 06), e `(self.pred)(l)` funciona porque o `&&str` vira `&str` por *deref coercion* na chamada.

## 6. Minigrep, retomada

Projeto: [`ex6-minigrep/`](ex6-minigrep/), uma cópia da [solução do Projeto 1](../projeto-01-minigrep/) com as mudanças abaixo. Os 25 testes unitários originais continuam passando, mais 4 novos, e os 10 de integração.

```bash
cd ex6-minigrep
cargo test
cargo run -q -- -n -i how poem.txt
```

**(a) [`search`](ex6-minigrep/src/search.rs) e [`render`](ex6-minigrep/src/lib.rs).** As duas viraram cadeias `filter`/`map`/`collect`. O teste `results_borrow_from_contents` continua passando porque `lines()` produz `&'a str` apontando para dentro de `contents`, e nenhum passo da cadeia copia. O teste `pattern_can_be_dropped_before_results` também: o `Vec<Match<'a>>` final só depende de `contents`.

**(b) [`from_args`](ex6-minigrep/src/config.rs).** Com `args: impl IntoIterator<Item = String>`, o laço `for arg in args` recebe cada `String` por valor:

- os posicionais vão para um `Vec<String>`, e não mais `Vec<&str>`;
- a opção desconhecida vira `ArgsError::UnknownFlag(arg)`, sem `clone`. Isso compila mesmo dentro do `match arg.as_str()`, porque o empréstimo do `as_str()` termina antes de o braço `_` executar (NLL, [Módulo 03](../../modulos/03-borrowing-e-lifetimes/02-borrow-checker-e-nll.md));
- `positional.into_iter()` + `next()` tira o padrão **por valor**, e o resto vira `Vec<Input>` com `map` + `collect`. `PathBuf::from(String)` reaproveita o buffer da `String`.

O teste `pattern_is_moved_not_copied` prova que não há cópia: o ponteiro do `cfg.pattern` é o mesmo da `String` passada. O `main` passa `env::args().skip(1)` direto.

Por que `impl IntoIterator` e não `impl Iterator`, como no Book (13.3)? O Book usa `mut args: impl Iterator<Item = String>` e chama `args.next()` para pular o nome do programa. Aqui, o nome do programa já sai no `skip(1)` do `main`, o parser percorre os argumentos com `for`, e os testes passam `Vec<String>`. Com `IntoIterator`, o `Vec`, o array e o iterador de `env::args()` servem sem mudança na chamada.

**(c) `search_iter`.** A versão preguiçosa está em `search.rs`, com dois testes (mesmo resultado que `search` com três combinações de opções, e parada na primeira linha com `.next()`). A resposta à pergunta: a closure do `filter` usa `pattern` a cada `next`, e por isso o iterador precisa guardar `pattern` (o `move` copia o `&str` para dentro da closure). Então `pattern` tem de viver tanto quanto o iterador, e a assinatura usa o mesmo `'a` nos dois parâmetros. Em `search`, a cadeia é consumida dentro da função, e o resultado só guarda pedaços de `contents`.
