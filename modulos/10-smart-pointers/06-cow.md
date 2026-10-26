# 06 — `Cow`: copiar só quando precisa

**Objetivos.** Ao final, você vai saber:

- usar `Cow<'a, str>` para devolver um empréstimo quando possível e um valor próprio quando necessário;
- trabalhar com `Cow`: `to_mut`, `into_owned`, e o `Deref` que o faz parecer um `&str`;
- ler o `Cow` que a std devolve, como o de `String::from_utf8_lossy`;
- tirar as alocações do parser de CSV do Projeto 2, e medir a diferença.

## O problema: alocar sempre por causa do caso raro

Uma função que troca tabulações por espaços. Na maioria das linhas não há tabulação nenhuma, mas a assinatura óbvia obriga a alocar sempre:

```rust
fn sem_tabs(s: &str) -> String {
    s.replace('\t', "    ") // aloca uma String nova mesmo quando não há '\t'
}
```

Devolver `&str` não resolve: quando há tabulação, o texto novo precisa morar em algum lugar, e uma referência para uma variável local não pode sair da função (Módulo 03). O tipo de retorno precisa dizer "às vezes emprestado, às vezes meu".

É o que `std::borrow::Cow` (*clone on write*, clonar ao escrever) faz. É um enum com as duas possibilidades:

```rust
// definição simplificada da std
enum Cow<'a, B: ?Sized + ToOwned> {
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

Para `B = str`, as variantes são `Borrowed(&'a str)` e `Owned(String)`. `ToOwned` é a trait que liga um tipo emprestado à sua versão própria: `str` → `String`, `[T]` → `Vec<T>`, `Path` → `PathBuf`. O `?Sized` é o da lição 06 do Módulo 06: permite que `B` seja `str`, que não tem tamanho fixo.

```rust
use std::borrow::Cow;

fn sem_tabs(s: &str) -> Cow<'_, str> {
    if s.contains('\t') {
        Cow::Owned(s.replace('\t', "    "))
    } else {
        Cow::Borrowed(s)
    }
}

for s in ["sem tab", "com\ttab"] {
    let r = sem_tabs(s);
    let tipo = match r {
        Cow::Borrowed(_) => "emprestado",
        Cow::Owned(_) => "alocado",
    };
    println!("{r:?} ({tipo}, {} bytes)", r.len());
}
```

```text
"sem tab" (emprestado, 7 bytes)
"com    tab" (alocado, 10 bytes)
```

Quem chama não precisa saber qual variante recebeu. `Cow<str>` implementa `Deref<Target = str>` (lição 02), então `r.len()`, `&*r`, a passagem para uma função que pede `&str` e o `Display` funcionam nos dois casos. Também implementa `PartialEq` com `&str` e `String`, e por isso dá para escrever `assert_eq!(r, "sem tab")`.

O `'_` na assinatura diz que o `Cow` pode emprestar de `s`. Sem ele (`-> Cow<str>`), o código compila, mas a edition 2024 avisa:

```text
warning: hiding a lifetime that's elided elsewhere is confusing
 --> src/main.rs:3:16
  |
3 | fn sem_tabs(s: &str) -> Cow<str> {
  |                ^^^^     ^^^^^^^^ the same lifetime is hidden here
  |                |
  |                the lifetime is elided here
...
help: use `'_` for type paths
  |
3 | fn sem_tabs(s: &str) -> Cow<'_, str> {
  |                             +++
```

É o lint `mismatched_lifetime_syntaxes`, o mesmo da [lição 05 do Módulo 03](../03-borrowing-e-lifetimes/05-elisao-e-lifetime-anonimo.md).

## Trabalhando com um `Cow`

- `cow.to_mut()` devolve `&mut String`. Se o `Cow` era `Borrowed`, ele **clona agora** e vira `Owned`; se já era `Owned`, só empresta. É o "clonar ao escrever" do nome.
- `cow.into_owned()` consome o `Cow` e devolve a `String`, clonando só se era `Borrowed`.
- `Cow::from("texto")` e `Cow::from(String::from("texto"))` criam as duas variantes (e `.into()` funciona onde o tipo é conhecido).

```rust
let mut nome: Cow<str> = Cow::Borrowed("ana");
nome.to_mut().make_ascii_uppercase(); // clona aqui, e altera a cópia
println!("{nome}");
let dono: String = nome.into_owned(); // já era Owned: nenhuma cópia nova
```

```text
ANA
```

O `Rc::make_mut` da lição 03 é a mesma ideia: copiar só no momento de alterar, e só se for preciso.

## O `Cow` que a std devolve

`String::from_utf8_lossy(&bytes)` converte bytes em texto, trocando sequências inválidas por `�` (U+FFFD). Se os bytes já são UTF-8 válido, não há o que trocar, e ele devolve `Cow::Borrowed`, sem copiar. Só aloca quando precisa substituir algo:

```rust
assert!(matches!(String::from_utf8_lossy(b"ola"), Cow::Borrowed("ola")));
let t = String::from_utf8_lossy(b"ol\xffa");
assert_eq!(t, "ol\u{FFFD}a");
assert!(matches!(t, Cow::Owned(_)));
```

Isso responde à pergunta do item 6 de "Para ir além" do [Projeto 1](../../projetos/01-minigrep/README.md). Se o minigrep lê o arquivo com `fs::read` e converte com `from_utf8_lossy`, o texto passa a ser um `Cow<str>` guardado numa variável local, e as `Match<'a>` passam a emprestar **desse `Cow`** (via `&contents`, que vira `&str` pela *deref coercion*), e não mais da `String` que o `read_input` devolvia. Na prática (com os nomes da solução do projeto):

```rust
let bytes = fs::read(path)?;
let contents = String::from_utf8_lossy(&bytes); // Cow<'_, str>, empresta de `bytes`
let matches = search(pattern, &contents, opts); // Vec<Match<'_>> empresta de `contents`
```

O lifetime das `Match` fica preso aos dois, `contents` e `bytes`: nenhum deles pode ser descartado enquanto os resultados forem usados. Para um arquivo UTF-8 válido, o custo extra é zero.

Outras funções da std com `Cow` no retorno: `Path::to_string_lossy` e `OsStr::to_string_lossy`, pelo mesmo motivo.

## O csvlite sem cópias

O benchmark do Projeto 2 mostrou que ler 10 mil registros custa cerca de 2,4 ms, e que o custo está nas 30 mil `String` alocadas, uma por campo. Mas um campo sem aspas é um pedaço da entrada, igualzinho. E um campo entre aspas também, desde que não tenha `""` dentro: o conteúdo entre as aspas é uma fatia contínua. Só o campo com aspa escapada precisa de texto novo, porque `""` vira `"`.

A função do projeto [`codigo/cow-csv`](codigo/cow-csv/src/lib.rs) separa uma linha em `Vec<Cow<'_, str>>`:

```rust
pub fn campos(linha: &str) -> Vec<Cow<'_, str>> {
    let mut saida = Vec::new();
    let mut resto = linha;
    loop {
        let (campo, depois) = match resto.strip_prefix('"') {
            Some(dentro) => campo_com_aspas(dentro),
            None => match resto.split_once(',') {
                Some((campo, depois)) => (Cow::Borrowed(campo), Some(depois)),
                None => (Cow::Borrowed(resto), None),
            },
        };
        saida.push(campo);
        match depois {
            Some(d) => resto = d,
            None => return saida,
        }
    }
}
```

E, dentro de `campo_com_aspas`, a decisão:

```rust
let conteudo = &dentro[..i]; // o texto entre as aspas
let campo = if tem_escape {
    Cow::Owned(conteudo.replace("\"\"", "\""))
} else {
    Cow::Borrowed(conteudo)
};
```

Os testes conferem não só o valor, mas a variante:

```rust
let c = campos(r#"Ana,"Rio, RJ","diz ""oi""""#);
assert_eq!(c, ["Ana", "Rio, RJ", r#"diz "oi""#]);
assert!(matches!(c[1], Cow::Borrowed(_)));
assert!(matches!(c[2], Cow::Owned(_)));
```

Para comparar de forma justa, `campos_string` faz exatamente o mesmo trabalho e converte cada campo em `String` no final (`Cow::into_owned`), como o `Record` do Projeto 2. O benchmark usa as mesmas linhas do Projeto 2 (`{i},nome {i},"texto, com vírgula {i}"`, 10 mil vezes):

```bash
cd codigo/cow-csv
cargo bench -- --warm-up-time 1 --measurement-time 3
```

```text
campos_string           time:   [1.3377 ms 1.3408 ms 1.3452 ms]
campos_cow              time:   [531.35 µs 531.90 µs 532.40 µs]
```

Cerca de 2,5 vezes mais rápido, só por não copiar (os números são de um Apple Silicon; os seus vão variar, mas a proporção deve ser parecida). O que sobrou é, em boa parte, o `Vec` alocado por linha. Daria para reaproveitar um `Vec` entre linhas, mas isso fica como desafio.

O preço é o lifetime: um `Vec<Cow<'a, str>>` empresta da linha, e a linha precisa viver enquanto os campos forem usados. No Projeto 2, o `Record` é dono dos seus campos e pode ser guardado à vontade. Com `Cow`, o `Record` viraria `Record<'a>`, e o `Reader` não poderia mais ler de um arquivo aos poucos, descartando as linhas já lidas (o item 2 de "Para ir além"). A crate `csv` resolve de outro jeito: o registro é dono dos dados, mas `read_record(&mut registro)` grava sempre no mesmo `StringRecord`, reaproveitando a memória em vez de alocar a cada linha.

## Armadilhas comuns

**`Cow` em todo lugar.** Na maioria das funções, `&str` na entrada e `String` na saída é o certo e é mais simples. `Cow` vale quando o caso comum dispensa a cópia **e** a função roda muitas vezes. Meça antes, como na seção anterior.

**`Cow` como parâmetro.** `fn f(s: Cow<str>)` raramente faz sentido. Se a função só lê, peça `&str`. Se ela precisa de uma `String`, peça `String` (ou `impl Into<String>`) e deixe o chamador decidir se clona.

**Esquecer o `'_`** no retorno: compila, mas com o aviso `mismatched_lifetime_syntaxes`, e esconde de quem lê que o resultado pode emprestar do argumento.

**Chamar `to_mut()` sem precisar.** Ele clona um `Borrowed` mesmo que você não altere nada depois. Decida antes se vai alterar, como `sem_tabs` faz com o `contains`.

## Resumo

- `Cow<'a, B>` é um enum: `Borrowed(&'a B)` ou `Owned(B::Owned)`. Para texto, `Cow<'a, str>` é "um `&str` ou uma `String`".
- Use-o no retorno de funções que, no caso comum, devolvem a entrada intacta.
- `Cow` implementa `Deref`: quem recebe usa como `&str`. `to_mut()` clona só se for `Borrowed`; `into_owned()` devolve a `String`.
- A std usa `Cow` em `from_utf8_lossy` e `to_string_lossy`: sem cópia quando a entrada já está boa.
- No parser de CSV, trocar `String` por `Cow` deixou a leitura 2,5 vezes mais rápida. O custo é o lifetime: os campos emprestam da linha.

## Exercícios

Veja o exercício 5 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Documentação de `std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) e de [`ToOwned`](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html)
- [Documentação de `String::from_utf8_lossy`](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy)
- [Effective Rust, Item 8: Familiarize yourself with reference and pointer types](https://effective-rust.com/references.html)
- [Documentação da crate `csv`: `Reader::read_record`](https://docs.rs/csv/latest/csv/struct.Reader.html#method.read_record), para comparar com o Projeto 2

---

[← Lição anterior](05-weak-e-ciclos.md) · [Índice do módulo](README.md) · [Exercícios →](exercicios.md)
