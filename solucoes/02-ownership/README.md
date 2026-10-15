# Soluções — Módulo 02

Enunciados em [`modulos/02-ownership/exercicios.md`](../../modulos/02-ownership/exercicios.md).

## 1. Três E0382, três correções

Projeto: [`tres-correcoes/`](tres-correcoes/src/main.rs).

| Parte | Correção | Por quê |
|---|---|---|
| (a) | imprimir `nome` **antes** de `nomes.push(nome)` | o vetor precisa ser dono da `String`. Não há motivo para copiar, só para usar antes de entregar |
| (b) | `fn contar_letras(texto: &str)` e `contar_letras(&frase)` | a função só lê. Ela não precisa da posse, e um empréstimo não custa nada |
| (c) | `let mut copia = original.clone();` | o enunciado pede duas listas **independentes**: uma muda e a outra não. Aqui o custo do `clone` é o que se quer |

Repare que, nas três, o compilador sugeriu `clone()`. Só em (c) ele era a resposta certa. Em (b), a `note:` do compilador ("consider changing this parameter type ... to borrow") apontava a correção melhor.

## 2. Copy ou não?

Projeto: [`copy-ou-nao/`](copy-ou-nao/src/main.rs).

| Struct | Pode ser `Copy`? | Motivo |
|---|---|---|
| `Ponto { x: f64, y: f64 }` | sim | todos os campos são `Copy` |
| `Cor(u8, u8, u8)` | sim | idem |
| `Usuario { id: u32, nome: String }` | **não**, E0204 | `nome: String` não é `Copy` (é dona de memória no heap) |
| `Grade([i32; 4])` | sim | um array de `Copy` é `Copy` |
| `Arquivo(&'static str)` com `Drop` | **não**, E0184 | tipos com destrutor não podem ser `Copy`, mesmo com campos `Copy` (`&str` é `Copy`) |

Depois de trocar `#[derive(Clone, Copy)]` por `#[derive(Clone)]` em `Usuario` e `Arquivo`, o `main` dá dois E0382 novos:

```text
error[E0382]: borrow of moved value: `u1`
  --> src/main.rs:39:47
...
error[E0382]: borrow of moved value: `a1`
  --> src/main.rs:47:34
```

Agora `let u2 = u1;` e `let a2 = a1;` são **moves**. A correção é `u1.clone()` e `a1.clone()`.

Respostas:

- **(a)** Os 16 bytes de `[i32; 4]` *são* o valor inteiro: duplicá-los dá uma cópia independente. Um `Vec<i32>` guarda um ponteiro para o heap. Duplicar os bytes dele daria dois donos do mesmo buffer (o *double free* que o move evita). A pergunta certa não é "está na stack?", e sim "duplicar os bytes dá um valor independente?".
- **(b)** Duas linhas `fechando dados.txt`, no fim do `main`: `a1` e o clone `a2` são dois valores, e cada um é descartado uma vez. Isso também mostra por que `Clone` pede cuidado com tipos que representam recursos: clonar um "arquivo" assim não abre outro arquivo, só cria mais um valor que vai "fechar" o mesmo nome.

## 3. Preveja a ordem do drop

Projeto: [`preveja-o-drop/`](preveja-o-drop/src/main.rs) (o mesmo código, com a explicação de cada linha em comentários).

```text
início: a1 b
drop d
bloco: c
drop c
e = e1
drop e1
devolve recebeu b
drop e2
fim: a2 b
drop f
drop b
drop a2
drop a1
```

Os pontos que costumam enganar:

- **`drop d` vem antes de `bloco: c`.** `let _ =` não cria variável, e o temporário é descartado no fim da própria instrução.
- **O segundo `let a` não descarta `a1`.** Sombreamento cria uma variável nova, e `a1` vive até o fim do `main`. Por isso `drop a1` é a última linha.
- **`drop e1` acontece na reatribuição** `e = Barulhento("e2")`, e não no fim.
- **`devolve` não descarta nada.** Ela recebe a posse e a devolve, e o valor vai parar no segundo `b`.
- **No fim, só quatro descartes.** As variáveis são descartadas em ordem inversa da declaração (`_f`, segundo `b`, `e`, `a2`, primeiro `b`, `a1`), mas `e` foi movida para `drop()` e o primeiro `b` foi movido para `devolve`. Valores movidos não são descartados na origem.

## 4. Assinaturas certas

Projeto: [`assinaturas/`](assinaturas/src/lib.rs).

```rust
pub fn contar_vogais(s: &str) -> usize
pub fn saudacao(nome: &str) -> String
pub fn gritar(s: &str) -> String
pub fn total_de_letras(palavras: &[String]) -> usize
```

O código inicial dava seis E0308 nos testes (literais e referências onde a função pedia `String` ou `Vec<String>`). As quatro funções só **leem** os parâmetros, então todos viram empréstimos: `&str` aceita literal, `&String` e fatia; `&[String]` aceita `&Vec<String>`, arrays e pedaços de vetor. No corpo de `total_de_letras`, `for p in &palavras` vira `for p in palavras`, porque `palavras` já é uma referência.

O **retorno** de `saudacao` e `gritar` continua `String` porque as funções **criam** um texto novo, e alguém precisa ser dono dele. Devolver `&str` exigiria apontar para algo que vive fora da função, e o texto criado lá dentro morre no fim dela (E0515, na [lição 05](../../modulos/02-ownership/05-string-e-str.md)).

## 5. Strings UTF-8

Projeto: [`strings-utf8/`](strings-utf8/src/lib.rs).

```rust
pub fn primeiro_char(s: &str) -> Option<char> {
    s.chars().next()
}

pub fn inverter(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn truncar(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((fim, _)) => &s[..fim],
        None => s,
    }
}

pub fn posicao_em_chars(s: &str, alvo: char) -> Option<usize> {
    s.chars().position(|c| c == alvo)
}
```

- `truncar` nunca dá *panic*: `char_indices` só devolve índices onde um `char` começa, que são sempre fronteiras válidas. Se o texto tem `max_chars` caracteres ou menos, `nth` devolve `None` e o texto inteiro é a resposta. E nada é copiado: o resultado é uma fatia de `s`.
- O erro clássico é `&s[..max_chars]`, que corta por **bytes**: passa com `"abc"` e dá *panic* com `"çãé"`.
- `"👍🏽"` são dois `char`s (o polegar e o modificador de tom de pele) e um grafema. `inverter` troca a ordem dos dois `char`s, e o modificador "solto" aparece antes do polegar. Inverter o que uma pessoa vê exige trabalhar com grafemas (crate `unicode-segmentation`).

## 6. Do Python para o Rust

Projeto: [`documento/`](documento/src/lib.rs).

```rust
pub fn novo(palavras: Vec<String>) -> Documento {
    Documento { palavras }
}

pub fn com_palavra(mut doc: Documento, palavra: &str) -> Documento {
    doc.palavras.push(palavra.to_string());
    doc
}

pub fn copiar(doc: &Documento) -> Documento {
    Documento {
        palavras: doc.palavras.clone(),
    }
}
```

**Pergunta 1.** Usar `d` depois de `com_palavra(d, ...)` dá:

```text
error[E0382]: borrow of moved value: `d`
  --> src/lib.rs:39:9
   |
37 |         let d = novo(vec![String::from("olá")]);
   |             - move occurs because `d` has type `Documento`, which does not implement the `Copy` trait
38 |         let d3 = com_palavra(d, "mundo");
   |                              - value moved here
39 |         assert_eq!(d.palavras, ["olá"]);
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ value borrowed here after move
...
```

É o bug do Python ao contrário: lá, duas variáveis davam acesso à mesma lista, e uma alteração por uma delas aparecia na outra. Em Rust, depois que o documento foi entregue a `com_palavra`, o nome antigo deixa de existir, e não sobra um segundo caminho para o mesmo dado. A única forma de ter dois documentos é pedir uma cópia (`copiar`, com `clone()`), e aí eles são independentes.

**Pergunta 2.** Em Python, basta copiar a lista: `Documento(list(d.palavras))` (ou `d.palavras.copy()`). A cópia rasa é suficiente **neste caso** porque os elementos são `str`, que são imutáveis em Python: compartilhar o mesmo objeto `str` entre as duas listas não causa surpresa. Se os elementos fossem listas, seria preciso `copy.deepcopy`. Em Rust, essa distinção não aparece: o `clone()` do `Vec<String>` copia cada `String`, porque cada uma tem um dono só.
