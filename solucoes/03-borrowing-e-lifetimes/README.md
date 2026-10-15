# Soluções — Módulo 03

Enunciados em [`modulos/03-borrowing-e-lifetimes/exercicios.md`](../../modulos/03-borrowing-e-lifetimes/exercicios.md).

Cada exercício com código tem um projeto aqui. Para conferir:

```bash
cd push-fantasma
cargo test
```

## 1. O `push` fantasma

Projeto: [`push-fantasma/`](push-fantasma/src/lib.rs).

As duas funções davam E0502: uma referência para dentro de `nomes`/`itens` estava viva quando o `push` pediu `&mut`. O programa era **inseguro**: o `push` pode realocar o buffer, e a referência ficaria apontando para memória liberada (o mesmo caso do diagrama da lição 01).

- `registra`: a função só precisa do **tamanho** do maior nome. Guardar um `usize` (que é `Copy`) em vez de `&String` encerra o empréstimo antes do `push`. O `unwrap_or(0)` trata o vetor vazio:

  ```rust
  let maior = nomes.iter().map(|n| n.len()).max().unwrap_or(0);
  nomes.push(novo.to_string());
  maior
  ```

- `resumo`: o total precisa ser contado **depois** do `push`, então não adianta guardar o primeiro item antes. A saída é reordenar: muta primeiro, e só então pega um empréstimo novo para ler:

  ```rust
  itens.push(String::from("fim"));
  format!("primeiro: {}, total: {}", itens[0], itens.len())
  ```

## 2. Assinaturas

Projeto: [`assinaturas/`](assinaturas/src/lib.rs).

| Função | Decisão | Por quê |
|---|---|---|
| `longest` | `<'a>` em `x`, `y` e no retorno | duas referências de entrada, sem `self`: a elisão não decide, e o retorno pode vir de qualquer uma |
| `first_word` | já compilava | regra 2: uma entrada só |
| `keep_first` | `<'a>` só em `x` e no retorno | o retorno vem sempre de `x`. Com `'a` também em `y`, o teste `keep_first_nao_depende_do_segundo` dá E0597, porque `y` morre antes do `assert` |
| `version` | `&'static str` | devolve um literal, que vive o programa todo |
| `greeting` | `String` | o texto é montado em runtime com `format!`. `'static` não funcionaria (seria E0515), porque a `String` local morre no fim da função |
| `split_pair` | já compilava | só `s` é referência (`sep` é `char`), então a regra 2 liga as duas saídas a `s` |

## 3. As palavras mais longas, sem copiar

Projeto: [`palavras-longas/`](palavras-longas/src/lib.rs).

```rust
pub fn palavras_mais_longas(texto: &str, n: usize) -> Vec<&str> {
    let mut palavras: Vec<&str> = texto
        .split_whitespace()
        .map(|p| p.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|p| !p.is_empty())
        .collect();
    palavras.sort_by_key(|p| std::cmp::Reverse(p.chars().count()));
    palavras.truncate(n);
    palavras
}
```

- `split_whitespace` e `trim_matches` só **recortam**: devolvem `&str` que apontam para dentro de `texto`. Por isso o teste de endereços passa.
- `chars().count()` mede em caracteres. Com `len()`, palavras acentuadas contariam a mais: `"ação"` (4 caracteres, 6 bytes) passaria na frente de `"casas"` (5 caracteres, 5 bytes).
- `sort_by_key` é estável: em empate, mantém a ordem original. `Reverse` inverte a comparação para ordenar do maior para o menor.

**Pergunta extra:** só há **uma** referência de entrada (`n` é `usize`). Pela regra 2 da elisão, o lifetime dela vai para as referências da saída. A assinatura completa seria `fn palavras_mais_longas<'a>(texto: &'a str, n: usize) -> Vec<&'a str>`.

## 4. Um tokenizador que não copia

Projeto: [`tokenizer/`](tokenizer/src/lib.rs).

O problema estava no contrato. Com `fn next_token(&mut self) -> Option<&str>`, a regra 3 da elisão prende o token ao empréstimo `&mut self`:

- em `all_tokens`, cada `tok` guardado em `out` mantém `t` emprestado como `&mut`, e a chamada seguinte de `next_token` é um segundo `&mut`: **E0499**;
- em `first_token`, o token devolvido empresta de `t`, que é local: **E0515**.

Os tokens, na verdade, apontam para o texto original, que vive em `'a`. A assinatura certa diz isso:

```rust
pub fn next_token(&mut self) -> Option<&'a str> {
    let s = self.rest.trim_start();
    let first = s.chars().next()?;
    let end = if is_word_char(first) {
        s.find(|c: char| !is_word_char(c)).unwrap_or(s.len())
    } else {
        first.len_utf8()
    };
    let (token, rest) = s.split_at(end);
    self.rest = rest;
    Some(token)
}
```

`self.rest = rest` funciona porque `rest` também é `&'a str`: o tokenizador só avança a janela sobre o texto. `first.len_utf8()` garante que um caractere de pontuação com mais de um byte seja cortado numa fronteira válida. É o mesmo caso de `part` x `part_long` da lição 06. Este padrão (uma struct que é visão de um texto e devolve `&'a str`) aparece em praticamente todo parser escrito em Rust.

## 5. Inventário sem `clone`

Projeto: [`inventario/`](inventario/src/lib.rs).

**`compra_replica` (E0506).** `caro` veio de `self.mais_caro()`, um método `&self`, então empresta `self` **inteiro**. Atribuir a `self.ouro` enquanto `caro` ainda é usado (no `push`, logo abaixo) é proibido. É uma **limitação**: `ouro` e o item são lugares diferentes, mas a assinatura de `mais_caro` não diz isso. A correção é terminar de usar `caro` primeiro, montando o item novo:

```rust
let replica = match self.mais_caro() {
    Some(caro) => Item {
        nome: format!("{} (réplica)", caro.nome),
        preco: caro.preco / 10,
    },
    None => return false,
};
// daqui em diante, nenhum empréstimo de self está vivo
```

O `format!` cria uma `String` nova a partir de `&caro.nome`. Não é um `clone` do item: o nome da réplica é outro texto mesmo.

**`funde` (E0499).** Duas `&mut` para elementos do mesmo `Vec`. Para o checker, `self.itens[de]` e `self.itens[para]` são `itens[_]`. Com índices diferentes seria seguro, mas com `de == para` seriam duas `&mut` para o **mesmo** item: **inseguro** no caso geral, e o checker não sabe os índices. `get_disjoint_mut` verifica isso em runtime e ainda recusa índices inválidos, que é o que os testes pedem:

```rust
match self.itens.get_disjoint_mut([de, para]) {
    Ok([origem, destino]) => {
        destino.preco += origem.preco;
        origem.preco = 0;
        true
    }
    Err(_) => false,
}
```

**`limita_a_media` (E0502).** O `for item in &mut self.itens` mantém `self.itens` emprestado como `&mut` durante o laço, e `self.media()` pede `&self` inteiro, o que inclui ler `self.itens`. É aliasing com mutação: **inseguro**. E, além de não compilar, estaria errado: a média mudaria a cada item alterado. Calcule uma vez, antes, e guarde o `u32`:

```rust
let media = self.media();
for item in &mut self.itens {
    if item.preco > media {
        item.preco = media;
    }
}
```

## 6. Quiz: inseguro ou limitação?

| | Erro | Classificação | Correção |
|---|---|---|---|
| A | E0515 | **inseguro**: `s` é liberada no fim da função e a referência ficaria pendurada | devolver `String`: `fn make(prefix: &str) -> String { format!("{prefix}!") }` |
| B | E0502 | **inseguro**: o `push` pode realocar, e `first` leria memória liberada | usar `first` antes do `push`, ou copiar o valor: `let first = v[0];` (`i32` é `Copy`) |
| C | E0502 | **limitação**: `get_first` só lê `name.0`, e o `push_str` mexe em `name.1`, mas o checker só vê a assinatura, que empresta a tupla inteira | `let first = &name.0;` direto, ou reordenar |
| D | E0502 | **limitação**: `a[1]` e `a[2]` são posições diferentes, mas o checker vê `a[_]` | `let (l, r) = a.split_at_mut(2); l[1] += r[0];`, ou ler antes: `let y = a[2]; a[1] += y;` |
| E | E0502 | **limitação**: no caminho do `return v` o `insert` nunca roda, mas o checker atual (sem Polonius) estende o empréstimo do `get` até o fim da função | `map.entry(k).or_default()` |
| F | E0507 | **inseguro**: `s` e o vetor seriam donos da mesma `String`, liberada duas vezes | emprestar (`&v[0]`), copiar (`v[0].clone()`) ou tirar do vetor (`v.remove(0)`, `std::mem::take(&mut v[0])`, que exige `let mut v`) |
