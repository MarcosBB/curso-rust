# Exercícios — Módulo 02

Soluções em [`solucoes/02-ownership`](../../solucoes/02-ownership/).

Para cada exercício, crie um projeto com `cargo new` (ou `cargo new --lib`, quando indicado) e cole o código inicial no arquivo indicado.

## 1. Três E0382, três correções

Cole em `src/main.rs`. O programa tem três erros E0382, um em cada função. Conserte cada um de um jeito **diferente**, seguindo a restrição escrita no comentário, e escreva ao lado da correção um comentário de uma linha justificando a escolha.

```rust
fn contar_letras(texto: String) -> usize {
    texto.chars().filter(|c| c.is_alphabetic()).count()
}

// (a) Conserte só mudando a ordem das linhas.
fn parte_a() -> Vec<String> {
    let mut nomes = Vec::new();
    let nome = String::from("Ferris");
    nomes.push(nome);
    println!("adicionei {nome}");
    nomes
}

// (b) Conserte sem usar clone(): a função só precisa ler o texto.
fn parte_b() {
    let frase = String::from("Olá, mundo!");
    let n = contar_letras(frase);
    println!("{frase:?} tem {n} letras");
}

// (c) As duas listas precisam ser independentes.
fn parte_c() {
    let original = vec![String::from("a"), String::from("b")];
    let mut copia = original;
    copia.push(String::from("c"));
    println!("original: {original:?}");
    println!("cópia:    {copia:?}");
}

fn main() {
    let nomes = parte_a();
    println!("{nomes:?}");
    parte_b();
    parte_c();
}
```

**Saída esperada:**

```text
adicionei Ferris
["Ferris"]
"Olá, mundo!" tem 8 letras
original: ["a", "b"]
cópia:    ["a", "b", "c"]
```

<details>
<summary>Dica</summary>

Use o fluxograma "As três saídas para o E0382" da [lição 02](02-regras-de-ownership-e-move.md#as-três-saídas-para-o-e0382). Em (b), a mudança é na assinatura de `contar_letras` **e** na chamada.

</details>

## 2. Copy ou não?

Cole em `src/main.rs`. Todos os structs pedem `Copy`, mas nem todos podem ter. Faça o programa compilar:

1. remova `Copy` **só** onde o compilador recusar, e anote qual erro (E0204 ou E0184) cada um deu e por quê;
2. depois disso, o `main` vai dar novos erros. Conserte-os com `.clone()`, sem mudar o que é impresso.

```rust
#[derive(Clone, Copy)]
struct Ponto {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Cor(u8, u8, u8);

#[derive(Clone, Copy)]
struct Usuario {
    id: u32,
    nome: String,
}

#[derive(Clone, Copy)]
struct Grade([i32; 4]);

#[derive(Clone, Copy)]
struct Arquivo(&'static str);

impl Drop for Arquivo {
    fn drop(&mut self) {
        println!("fechando {}", self.0);
    }
}

fn main() {
    let p1 = Ponto { x: 1.0, y: 2.0 };
    let p2 = p1;
    println!("ponto: ({}, {}) e ({}, {})", p1.x, p1.y, p2.x, p2.y);

    let c1 = Cor(255, 128, 0);
    let c2 = c1;
    println!("cor: {} {} {} / {}", c1.0, c1.1, c1.2, c2.0);

    let u1 = Usuario { id: 1, nome: String::from("Ana") };
    let u2 = u1;
    println!("usuário: {} {} / {} {}", u1.id, u1.nome, u2.id, u2.nome);

    let g1 = Grade([1, 2, 3, 4]);
    let g2 = g1;
    println!("grade: {:?} / {:?}", g1.0, g2.0);

    let a1 = Arquivo("dados.txt");
    let a2 = a1;
    println!("arquivo: {} / {}", a1.0, a2.0);
}
```

Depois de compilar, responda: (a) por que `Grade([i32; 4])` pode ser `Copy` e um struct com um `Vec<i32>` não pode? (b) Quantas linhas "fechando" aparecem na saída, e por quê?

<details>
<summary>Dica</summary>

Releia a [lição 03](03-copy-e-clone.md) (o que define um tipo `Copy`) e a seção "Tipo com `Drop` não pode ser `Copy`" da [lição 04](04-drop-e-raii.md).

</details>

## 3. Preveja a ordem do drop

**Sem rodar**, escreva num papel (ou num comentário) a saída completa deste programa, linha por linha. Depois rode com `cargo run` e compare. Para cada linha que você errou, identifique qual regra da [lição 04](04-drop-e-raii.md) explica o resultado.

```rust
struct Barulhento(&'static str);

impl Drop for Barulhento {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn devolve(b: Barulhento) -> Barulhento {
    println!("devolve recebeu {}", b.0);
    b
}

fn main() {
    let a = Barulhento("a1");
    let b = Barulhento("b");
    println!("início: {} {}", a.0, b.0);
    {
        let c = Barulhento("c");
        let _ = Barulhento("d");
        println!("bloco: {}", c.0);
    }
    let a = Barulhento("a2");
    let mut e = Barulhento("e1");
    println!("e = {}", e.0);
    e = Barulhento("e2");
    let b = devolve(b);
    drop(e);
    let _f = Barulhento("f");
    println!("fim: {} {}", a.0, b.0);
}
```

A saída tem 13 linhas.

<details>
<summary>Dica</summary>

Preste atenção em quatro pontos: `let _ =`, o segundo `let a` (sombreamento), a reatribuição de `e` e o que acontece com o primeiro `b` depois de `devolve`.

</details>

## 4. Assinaturas certas

Crie um projeto com `cargo new --lib` e cole em `src/lib.rs`. A biblioteca compila, mas `cargo test` não. Faça os testes passarem **mudando só as assinaturas** (e o mínimo do corpo que a mudança exigir). Não altere os testes.

```rust
pub fn contar_vogais(s: String) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

pub fn saudacao(nome: String) -> String {
    format!("Olá, {nome}!")
}

pub fn gritar(s: String) -> String {
    format!("{}!", s.to_uppercase())
}

pub fn total_de_letras(palavras: Vec<String>) -> usize {
    let mut total = 0;
    for p in &palavras {
        total += p.chars().count();
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vogais_com_literal() {
        assert_eq!(contar_vogais("banana"), 3);
    }

    #[test]
    fn vogais_sem_perder_a_string() {
        let s = String::from("Ferris");
        assert_eq!(contar_vogais(&s), 2);
        assert_eq!(s, "Ferris");
    }

    #[test]
    fn saudacao_com_literal() {
        assert_eq!(saudacao("Ana"), "Olá, Ana!");
    }

    #[test]
    fn saudacao_com_string() {
        let nome = String::from("Bia");
        let s = saudacao(&nome);
        assert_eq!(s, "Olá, Bia!");
        assert_eq!(nome, "Bia");
    }

    #[test]
    fn gritar_com_literal() {
        assert_eq!(gritar("oi"), "OI!");
    }

    #[test]
    fn total_sem_consumir_o_vetor() {
        let palavras = vec![String::from("olá"), String::from("mundo")];
        assert_eq!(total_de_letras(&palavras), 8);
        assert_eq!(palavras.len(), 2);
    }
}
```

Responda também: por que o **retorno** de `saudacao` e `gritar` deve continuar `String`?

<details>
<summary>Dica</summary>

Veja a tabela "Qual usar onde" da [lição 05](05-string-e-str.md). Para `total_de_letras`, `&Vec<String>` funciona, mas existe um tipo de fatia mais geral, assim como `&str` é mais geral que `&String`.

</details>

## 5. Strings UTF-8 (passe nos testes)

Crie um projeto com `cargo new --lib` e cole em `src/lib.rs`. Implemente as quatro funções até `cargo test` passar. Nenhuma delas pode dar *panic*, com nenhuma entrada.

```rust
/// O primeiro `char` do texto, se houver.
pub fn primeiro_char(s: &str) -> Option<char> {
    todo!()
}

/// Inverte a ordem dos `char`s.
pub fn inverter(s: &str) -> String {
    todo!()
}

/// Os primeiros `max_chars` caracteres (chars, não bytes) de `s`,
/// como uma fatia do próprio `s`, sem copiar.
pub fn truncar(s: &str, max_chars: usize) -> &str {
    todo!()
}

/// A posição de `alvo` contada em chars (e não em bytes, como `find`).
pub fn posicao_em_chars(s: &str, alvo: char) -> Option<usize> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primeiro_char_funciona_com_acentos_e_vazio() {
        assert_eq!(primeiro_char("olá"), Some('o'));
        assert_eq!(primeiro_char("ação"), Some('a'));
        assert_eq!(primeiro_char("éramos"), Some('é'));
        assert_eq!(primeiro_char(""), None);
    }

    #[test]
    fn inverter_por_char() {
        assert_eq!(inverter("olá"), "álo");
        assert_eq!(inverter("çãé"), "éãç");
        assert_eq!(inverter(""), "");
    }

    #[test]
    fn inverter_separa_o_modificador_de_tom() {
        // "👍🏽" são dois chars: o polegar e o modificador de tom de pele.
        assert_eq!(inverter("👍🏽"), "\u{1F3FD}\u{1F44D}");
    }

    #[test]
    fn truncar_conta_chars_e_nao_bytes() {
        assert_eq!(truncar("olá", 2), "ol");
        assert_eq!(truncar("olá", 3), "olá");
        assert_eq!(truncar("çãé", 2), "çã");
        assert_eq!(truncar("👍🏽", 1), "👍");
    }

    #[test]
    fn truncar_nos_extremos() {
        assert_eq!(truncar("olá", 0), "");
        assert_eq!(truncar("olá", 10), "olá");
        assert_eq!(truncar("", 3), "");
    }

    #[test]
    fn posicao_em_chars_x_find() {
        let s = "pão de queijo";
        assert_eq!(s.find('d'), Some(5)); // bytes: "ã" ocupa 2
        assert_eq!(posicao_em_chars(s, 'd'), Some(4));
        assert_eq!(posicao_em_chars(s, 'z'), None);
    }
}
```

O teste `inverter_separa_o_modificador_de_tom` documenta um limite: inverter por `char` não é o mesmo que inverter o que uma pessoa vê. Por quê?

> [!NOTE]
> `truncar` devolve uma `&str` que aponta para dentro do parâmetro `s`. O compilador entende essa relação sozinho, sem anotação. Como ele faz isso é assunto do Módulo 03.

<details>
<summary>Dica</summary>

`chars()` tem `next()`, `rev()` e `position()`. Para `truncar`, `s.char_indices().nth(max_chars)` diz em que **byte** começa o primeiro char que deve ficar de fora (ou `None`, se o texto for curto). Use um `match` nesse resultado.

</details>

## 6. Do Python para o Rust (desafio)

Este código Python tem uma surpresa:

```python
class Documento:
    def __init__(self, palavras):
        self.palavras = palavras

d = Documento(["olá"])
d2 = Documento(d.palavras)   # a intenção era copiar
d2.palavras.append("mundo")
print(d.palavras)            # ['olá', 'mundo']: o original mudou
```

Crie um projeto com `cargo new --lib` e implemente a versão Rust em `src/lib.rs`, de modo que a surpresa seja impossível sem você pedir:

```rust
pub struct Documento {
    pub palavras: Vec<String>,
}

/// Cria um documento que passa a ser o dono das palavras.
pub fn novo(palavras: Vec<String>) -> Documento {
    todo!()
}

/// Consome o documento, acrescenta a palavra no fim e o devolve.
pub fn com_palavra(doc: Documento, palavra: &str) -> Documento {
    todo!()
}

/// Cria um documento independente do original, sem consumi-lo.
pub fn copiar(doc: &Documento) -> Documento {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novo_guarda_as_palavras() {
        let d = novo(vec![String::from("olá")]);
        assert_eq!(d.palavras, ["olá"]);
    }

    #[test]
    fn com_palavra_acrescenta_no_fim() {
        let d = novo(vec![String::from("olá")]);
        let d = com_palavra(d, "mundo");
        assert_eq!(d.palavras, ["olá", "mundo"]);
    }

    #[test]
    fn copia_e_independente_do_original() {
        let d = novo(vec![String::from("olá")]);
        let d2 = copiar(&d);
        let d2 = com_palavra(d2, "mundo");
        assert_eq!(d.palavras, ["olá"]);
        assert_eq!(d2.palavras, ["olá", "mundo"]);
    }
}
```

Depois que os testes passarem:

1. No teste `com_palavra_acrescenta_no_fim`, tente usar o primeiro `d` depois da chamada a `com_palavra` (por exemplo, trocando `let d = com_palavra(d, ...)` por `let d3 = com_palavra(d, ...)` e lendo `d.palavras` em seguida). Leia o erro. Que bug do Python ele impede?
2. Como você corrigiria a versão Python? Uma cópia rasa (`list(...)`) basta aqui, ou precisa de `copy.deepcopy`?

<details>
<summary>Dica</summary>

Para alterar o documento recebido em `com_palavra`, o parâmetro precisa ser `mut doc: Documento`. Em `copiar`, você só tem uma referência: precisa clonar o vetor.

</details>

---

[Índice do módulo](README.md)
