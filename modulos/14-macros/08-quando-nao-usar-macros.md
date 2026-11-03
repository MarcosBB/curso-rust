# 08 — Quando *não* usar macros

**Objetivos.** Ao final, você vai saber:

- o que uma macro custa para quem escreve, para quem usa e para o compilador;
- qual recurso da linguagem resolve o problema sem macro (função, genérico, trait, `const fn`);
- decidir entre `macro_rules!` e macro procedural quando a macro é mesmo necessária.

## O que uma macro custa

As lições anteriores mostraram o que uma macro **consegue** fazer. Agora, a outra metade: cada macro tem custos que uma função não tem.

- **Leitura.** Uma função tem uma assinatura que diz o que entra e o que sai. Uma macro tem padrões; para saber o que ela aceita e gera, é preciso ler as regras ou rodar `cargo expand`.
- **Mensagens de erro.** O erro pode cair no código gerado, com o `this error originates in the macro` no fim. As lições 02 e 06 mostraram quanto trabalho dá deixar essas mensagens boas.
- **Ferramentas.** O rust-analyzer expande macros, mas completar código, renomear e "ir para a definição" funcionam pior dentro de uma chamada de macro do que em código normal. O `rustfmt` não formata o conteúdo de chamadas com sintaxe própria (`comandos!(andar 3; girar;)` fica como você escreveu).
- **Tempo de compilação.** Uma macro procedural traz `syn`, `quote` e `proc-macro2`, que precisam compilar antes de tudo. Neste módulo, a compilação do zero do workspace `macros-procedurais` levou **3,1 s**, quase tudo no `syn` com `"full"`; o projeto `macros-declarativas`, sem dependências, levou **0,14 s**. Em projetos grandes, essas dependências costumam já estar lá (pelo `serde`, por exemplo), mas cada macro ainda roda a cada compilação, e *tt munchers* grandes também pesam.
- **Desconfiança.** Uma macro procedural é código arbitrário rodando na sua máquina durante o `cargo build`. Com uma crate conhecida, tudo bem; com uma desconhecida, é um risco de *supply chain* (Módulo 17 discute como avaliar crates).

Nenhum desses custos impede o uso. Mas eles significam que a macro precisa **ganhar** o lugar dela.

## Antes de escrever uma macro, tente...

### ...uma função

O caso mais comum de macro desnecessária é a que poderia ser uma função:

```rust
// Desnecessária: só calcula um valor a partir de outro.
macro_rules! dobro {
    ($x:expr) => {
        $x * 2
    };
}

// ✅ Melhor: tipos na assinatura, erros normais, avaliada uma vez.
fn dobro(x: i32) -> i32 {
    x * 2
}
```

A função ganha tudo: tipos verificados na assinatura, documentação, autocompletar, e nada de avaliação dupla. E o compilador decide sozinho se vale fazer *inline*; não existe ganho de desempenho em trocar uma função por uma macro, como às vezes se fazia em C.

### ...um genérico ou uma trait

"Preciso da mesma função para vários tipos" é trabalho para genéricos (Módulo 06), não para uma macro que copia o código:

```rust
fn maior<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

A macro faz sentido quando os tipos **não** compartilham uma trait que permita escrever o código uma vez só. Era o caso do `unidades!` da lição 03: cada newtype precisa do seu próprio `impl Display` e `impl Add`, e uma *blanket implementation* (Módulo 13) não resolve, porque os tipos não existem ainda. Gerar `impl`s repetidos para tipos diferentes é um uso legítimo.

### ...uma `const fn`

Calcular ou validar algo na compilação nem sempre exige macro. Uma `const fn` usada numa `const` roda **no compilador**, e um *panic* lá dentro vira erro de compilação. A `cor!` da lição 07 sem macro:

```rust
const fn digito(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("dígito hexadecimal inválido"),
    }
}

const fn cor(hex: &str) -> [u8; 3] {
    let b = hex.as_bytes();
    assert!(b.len() == 7 && b[0] == b'#', "use o formato \"#rrggbb\"");
    [
        digito(b[1]) * 16 + digito(b[2]),
        digito(b[3]) * 16 + digito(b[4]),
        digito(b[5]) * 16 + digito(b[6]),
    ]
}

const LARANJA: [u8; 3] = cor("#f97316");
```

Com uma cor errada (`const ERRADA: [u8; 3] = cor("#f9731");`):

```text
error[E0080]: evaluation panicked: use o formato "#rrggbb"
  --> src/main.rs:21:25
   |
21 | const ERRADA: [u8; 3] = cor("#f9731");
   |                         ^^^^^^^^^^^^^ evaluation of `ERRADA` failed inside this call
   |
note: inside `cor`
  --> src/main.rs:12:5
   |
12 |     assert!(b.len() == 7 && b[0] == b'#', "use o formato \"#rrggbb\"");
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the failure occurred here
```

Mesmo resultado, sem crate `proc-macro`, sem `syn`. A diferença: a `const fn` só valida quando é chamada num contexto constante. Usada em `let c = cor(...)` com um valor que só existe em tempo de execução, ela vira uma função comum, e o erro vira um *panic*. A macro valida sempre.

### ...uma closure ou um bloco

Macros que "envolvem" um trecho de código (medir tempo, repetir em caso de erro, abrir e fechar um recurso) muitas vezes cabem numa função que recebe uma closure:

```rust
fn cronometrado<T>(nome: &str, f: impl FnOnce() -> T) -> T {
    let inicio = std::time::Instant::now();
    let r = f();
    eprintln!("[cronometrar] {nome} levou {:?}", inicio.elapsed());
    r
}

let soma: u64 = cronometrado("soma", || (1..=1000u64).sum());
```

O `#[cronometrar]` da lição 07 é mais confortável (não muda a chamada), mas a função acima não precisa de nenhuma crate extra. Para uso pontual, ela basta. E RAII (Módulo 02) resolve o "abrir e fechar" sem macro nenhuma: um guarda com `Drop`.

## Quando a macro ganha

Use uma macro quando o problema é, de fato, um destes:

| Problema | Exemplo | Tipo de macro |
|---|---|---|
| número variável de argumentos, ou sintaxe própria | `vec!`, `hashmap!`, `println!` | `macro_rules!` |
| gerar itens repetidos para vários tipos | `unidades!`, os `impl`s da std para `i8`...`u128` | `macro_rules!` |
| capturar o código-fonte além do valor | `assert!`, `dbg!`, `mostrar!` (`stringify!`) | `macro_rules!` |
| implementar uma trait a partir da **estrutura** de um tipo | `Debug`, `Serialize`, `LinhaCsv` | derive |
| transformar um item inteiro | `#[tokio::main]`, `#[async_trait]`, `#[cronometrar]` | atributo |
| validar uma linguagem embutida na compilação | `sqlx::query!`, `cor!` | function-like |

E, entre os tipos:

- **Comece por `macro_rules!`.** Não precisa de crate separada, compila rápido e tem higiene de graça.
- **Passe para procedural** quando precisar inspecionar a estrutura de um tipo (derive), reescrever um item inteiro (atributo) ou fazer cálculos de verdade com a entrada (validar, ler arquivos, gerar nomes novos).

## Um checklist

Antes de publicar uma macro, confira:

- [ ] Uma função, um genérico ou uma `const fn` resolveriam? Se sim, use-os.
- [ ] A macro tem documentação com um exemplo de chamada (um *doctest*, Módulo 08)?
- [ ] As expressões recebidas são avaliadas **uma vez**?
- [ ] O código gerado usa caminhos absolutos (`$crate::`, `::std::`)?
- [ ] Entradas erradas geram uma mensagem que aponta o problema (`compile_error!`, `syn::Error`)?
- [ ] Há testes do uso real, inclusive com genéricos e com a vírgula final?

## Armadilhas comuns

- **Macro para ganhar desempenho.** Funções pequenas já são *inline*; a macro só deixa o código mais difícil de ler.
- **DSL onde uma API comum basta.** Um *builder* (Módulo 13) ou uma struct de configuração costumam ser mais fáceis de descobrir e documentar do que uma mini-linguagem.
- **Esconder controle de fluxo.** Uma macro que faz `return` ou `?` por dentro surpreende quem lê a chamada. Quando fizer isso (como o antigo `try!`, que virou o operador `?`), deixe claro no nome e na documentação.

## Resumo

- Macros custam leitura, mensagens de erro, suporte de ferramentas e tempo de compilação (3,1 s de `syn` contra 0,14 s neste módulo).
- Antes de uma macro, tente função, genérico, trait, `const fn`, closure ou RAII.
- Macros ganham para argumentos variáveis, sintaxe própria, geração de itens repetidos, derive a partir da estrutura e validação de linguagens embutidas.
- Comece por `macro_rules!`; vá para procedural quando precisar inspecionar tipos ou reescrever itens.

## Exercícios

Volte aos exercícios de [`exercicios.md`](exercicios.md) e pergunte, para cada um: isto poderia ser uma função? (Para `media!` e `maximo!`, quase: uma função com um *slice* resolveria, mas perderia a vírgula final, os tipos mistos e a avaliação na ordem escrita.)

## Leitura complementar

- [The Rust Programming Language, cap. 20.5: The Difference Between Macros and Functions](https://doc.rust-lang.org/stable/book/ch20-05-macros.html#the-difference-between-macros-and-functions)
- [The Rust Reference: Constant evaluation](https://doc.rust-lang.org/reference/const_eval.html)
- [Rust API Guidelines: Macros](https://rust-lang.github.io/api-guidelines/macros.html) (checklist de boas práticas para macros públicas)

---

[← Lição anterior](07-atributos-e-function-like.md) · [Índice do módulo](README.md) · [Próximo: exercícios →](exercicios.md)
