# Exercícios — Módulo 06

Soluções em [`solucoes/06-traits-e-generics`](../../solucoes/06-traits-e-generics/).

Em todos os exercícios (menos o 6), crie um projeto com `cargo new --lib nome`, acrescente `rust-version = "1.96"` no `Cargo.toml` e cole o código inicial no `src/lib.rs`. O objetivo é sempre o mesmo: `cargo test` passando, e `cargo clippy` sem avisos.

## 1. Faça compilar: generics sem bounds (fácil)

Lições: [01](01-generics.md) e [03](03-trait-bounds-where-e-impl-trait.md).

Todas as funções abaixo estão certas, menos as assinaturas. Faça o código compilar mexendo **só nas assinaturas** e nas linhas `impl` (não mude nenhum corpo). Depois, deixe o `cargo clippy` limpo.

```rust
// ❌ NÃO COMPILA
use std::cmp::Ordering;

/// O maior elemento, ou `None` se a lista estiver vazia.
pub fn maior<T>(lista: &[T]) -> Option<&T> {
    let mut maior = lista.first()?;
    for item in lista {
        if item > maior {
            maior = item;
        }
    }
    Some(maior)
}

/// O menor dos dois (o primeiro, se forem iguais).
pub fn menor_de<T>(a: T, b: T) -> T {
    match a.cmp(&b) {
        Ordering::Greater => b,
        Ordering::Less | Ordering::Equal => a,
    }
}

/// Duas cópias de um valor emprestado.
pub fn duplicar<T>(x: &T) -> (T, T) {
    (x.clone(), x.clone())
}

/// `["a", "b"]` vira `["1. a", "2. b"]`.
pub fn numerar<T>(itens: &[T]) -> Vec<String> {
    let mut saida = Vec::new();
    for (i, item) in itens.iter().enumerate() {
        saida.push(format!("{}. {item}", i + 1));
    }
    saida
}

/// Uma pilha genérica (LIFO).
#[derive(Debug)]
pub struct Pilha<T> {
    itens: Vec<T>,
}

impl Pilha<T> {
    pub fn new() -> Self {
        Pilha { itens: Vec::new() }
    }

    pub fn empilhar(&mut self, item: T) {
        self.itens.push(item);
    }

    pub fn desempilhar(&mut self) -> Option<T> {
        self.itens.pop()
    }

    pub fn topo(&self) -> Option<&T> {
        self.itens.last()
    }

    pub fn len(&self) -> usize {
        self.itens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.itens.is_empty()
    }
}
```

Cole os testes no fim do arquivo:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maior_funciona_com_varios_tipos() {
        assert_eq!(maior(&[3, 7, 2]), Some(&7));
        assert_eq!(maior(&['r', 'u', 's', 't']), Some(&'u'));
        assert_eq!(maior(&["pera", "uva", "maçã"]), Some(&"uva"));
        assert_eq!(maior(&[1.5, -2.0]), Some(&1.5));
        assert_eq!(maior::<i32>(&[]), None);
    }

    #[test]
    fn menor_de_dois() {
        assert_eq!(menor_de(3, 2), 2);
        assert_eq!(menor_de("b", "a"), "a");
        assert_eq!(menor_de(String::from("x"), String::from("x")), "x");
    }

    #[test]
    fn duplicar_clona() {
        let s = String::from("oi");
        assert_eq!(duplicar(&s), (String::from("oi"), String::from("oi")));
        assert_eq!(duplicar(&5), (5, 5));
    }

    #[test]
    fn numerar_usa_display() {
        assert_eq!(numerar(&["a", "b"]), ["1. a", "2. b"]);
        assert_eq!(numerar(&[10, 20]), ["1. 10", "2. 20"]);
    }

    #[test]
    fn pilha() {
        let mut p = Pilha::new();
        assert!(p.is_empty());
        p.empilhar("a");
        p.empilhar("b");
        assert_eq!(p.topo(), Some(&"b"));
        assert_eq!(p.desempilhar(), Some("b"));
        assert_eq!(p.len(), 1);
        // `File` não implementa `Default`, e a pilha vazia funciona mesmo assim.
        let vazia: Pilha<std::fs::File> = Pilha::default();
        assert_eq!(vazia.len(), 0);
    }
}
```

<details>
<summary>Dicas</summary>

- São nove erros, mas só cinco causas. Comece pelo `impl` da `Pilha`: os quatro E0425 somem juntos.
- Para cada função, leia o corpo e pergunte "o que ele faz com `T`?": compara com `>`, chama `cmp`, chama `clone`, usa `{}`. Cada resposta é uma trait.
- Um dos testes guarda um `std::fs::File` na pilha. O clippy vai pedir um `Default` para `Pilha`: pense se `#[derive(Default)]` serve.

</details>

## 2. Passe nos testes: a trait `Forma` (médio)

Lições: [02](02-definindo-e-implementando-traits.md), [04](04-traits-da-std-e-derive.md) e [06](06-trait-objects-e-dyn.md).

Escreva, no `src/lib.rs`:

- uma trait `Forma` que exige `Display` (supertrait), com dois métodos obrigatórios, `area(&self) -> f64` e `perimetro(&self) -> f64`, e um método padrão `descricao(&self) -> String` que devolve `"<display>: área A, perímetro P"`, com duas casas decimais;
- `pub struct Retangulo { pub largura: f64, pub altura: f64 }` e `pub struct Circulo { pub raio: f64 }`, com `Display` à mão (`retângulo 2x3.5` e `círculo de raio 1`) e `Forma`. O `Circulo` substitui `descricao` para mostrar só a área;
- `pub fn maior_area(formas: &[Box<dyn Forma>]) -> Option<&dyn Forma>`;
- `pub fn area_total<T: Forma>(formas: &[T]) -> f64`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn exemplos() -> Vec<Box<dyn Forma>> {
        vec![
            Box::new(Retangulo {
                largura: 2.0,
                altura: 3.0,
            }),
            Box::new(Circulo { raio: 1.5 }),
            Box::new(Retangulo {
                largura: 1.0,
                altura: 1.0,
            }),
        ]
    }

    #[test]
    fn display() {
        let r = Retangulo {
            largura: 2.0,
            altura: 3.5,
        };
        assert_eq!(r.to_string(), "retângulo 2x3.5");
        assert_eq!(Circulo { raio: 1.0 }.to_string(), "círculo de raio 1");
    }

    #[test]
    fn descricao_padrao_e_substituida() {
        let formas = exemplos();
        assert_eq!(
            formas[0].descricao(),
            "retângulo 2x3: área 6.00, perímetro 10.00"
        );
        assert_eq!(formas[1].descricao(), "círculo de raio 1.5: área 7.07");
    }

    #[test]
    fn maior_area_entre_tipos_diferentes() {
        let formas = exemplos();
        let maior = maior_area(&formas).unwrap();
        assert_eq!(maior.to_string(), "círculo de raio 1.5");
        assert!(maior_area(&[]).is_none());
    }

    #[test]
    fn area_total_generica() {
        let quadrados = [
            Retangulo {
                largura: 1.0,
                altura: 1.0,
            },
            Retangulo {
                largura: 2.0,
                altura: 2.0,
            },
        ];
        assert_eq!(area_total(&quadrados), 5.0);
    }
}
```

<details>
<summary>Dicas</summary>

- Com `trait Forma: fmt::Display`, o método padrão pode usar `{self}` no `format!`.
- `f64` formata `2.0` como `2` e `3.5` como `3.5` com `{}`, que é o que os testes esperam.
- Em `maior_area`, guarde um `Option<&dyn Forma>`. Para tirar um `&dyn Forma` de um `&Box<dyn Forma>`, use `f.as_ref()` ou `&**f`.

</details>

## 3. Um logger decorador (médio)

Lição: [07](07-estatico-x-dinamico-e-composicao.md) (adaptado do exercício *Logger Trait* do Comprehensive Rust).

Escreva:

- `pub enum Nivel { Debug, Info, Aviso, Erro }`, ordenado por gravidade, com `Display` (`DEBUG`, `INFO`, `AVISO`, `ERRO`);
- `pub trait Logger` com o método obrigatório `log(&mut self, nivel: Nivel, mensagem: &str)` e os métodos padrão `info(&mut self, mensagem: &str)` e `erro(&mut self, mensagem: &str)`;
- `pub struct StderrLogger;`, que imprime `[NIVEL] mensagem` no stderr;
- `pub struct MemoriaLogger { pub mensagens: Vec<String> }` (com `Default`), que guarda `[NIVEL] mensagem`;
- `pub struct FiltroNivel<L: Logger> { pub minimo: Nivel, pub interno: L }`, que **também é um `Logger`** e só repassa para `interno` as mensagens com nível maior ou igual a `minimo`.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memoria_formata() {
        let mut l = MemoriaLogger::default();
        l.log(Nivel::Aviso, "disco quase cheio");
        l.info("pronto");
        assert_eq!(l.mensagens, ["[AVISO] disco quase cheio", "[INFO] pronto"]);
    }

    #[test]
    fn filtro_descarta_abaixo_do_minimo() {
        let mut l = FiltroNivel {
            minimo: Nivel::Aviso,
            interno: MemoriaLogger::default(),
        };
        l.log(Nivel::Debug, "detalhe");
        l.info("normal");
        l.log(Nivel::Aviso, "cuidado");
        l.erro("falhou");
        assert_eq!(l.interno.mensagens, ["[AVISO] cuidado", "[ERRO] falhou"]);
    }

    #[test]
    fn filtros_se_empilham() {
        let mut l = FiltroNivel {
            minimo: Nivel::Info,
            interno: FiltroNivel {
                minimo: Nivel::Erro,
                interno: MemoriaLogger::default(),
            },
        };
        l.info("some no segundo filtro");
        l.erro("passa pelos dois");
        assert_eq!(l.interno.interno.mensagens, ["[ERRO] passa pelos dois"]);
    }

    #[test]
    fn funciona_como_dyn() {
        let mut loggers: Vec<Box<dyn Logger>> = vec![
            Box::new(StderrLogger),
            Box::new(FiltroNivel {
                minimo: Nivel::Erro,
                interno: StderrLogger,
            }),
        ];
        for l in &mut loggers {
            l.info("só o primeiro mostra isto");
        }
    }
}
```

<details>
<summary>Dicas</summary>

- `#[derive(PartialOrd)]` num enum ordena pelas variantes, na ordem de declaração. Com isso, `nivel >= self.minimo` funciona.
- O `log` do filtro tem duas linhas: um `if` e uma chamada ao `log` de `self.interno`.
- Por que `&mut self` e não `&self`? Porque o `MemoriaLogger` precisa alterar o `Vec`. Com `&self`, seria preciso mutabilidade interior (`RefCell`, Módulo 10).

</details>

## 4. Conversões de `Porta` (médio)

Lição: [05](05-conversoes-from-into-tryfrom.md).

Uma `Porta` é um número de 1024 a 65535. Escreva:

- `pub struct Porta(u16);`, com o campo **privado**;
- `pub enum ErroPorta { Zero, Reservada(u16), NaoNumerica(ParseIntError) }`, com `Display` e `Error` (o `source()` devolve o `ParseIntError` quando houver);
- `TryFrom<u16> for Porta` (0 é `Zero`, 1 a 1023 é `Reservada`);
- `FromStr for Porta`, que aceita espaços em volta (`" 3000\n"`) e usa o `?` para converter o `ParseIntError`;
- `From<Porta> for u16`;
- `Display` para `Porta`, que mostra `:8080`.

As mensagens de erro esperadas estão nos testes:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_valido_e_invalido() {
        assert_eq!(u16::from(Porta::try_from(8080).unwrap()), 8080);
        assert_eq!(Porta::try_from(0), Err(ErroPorta::Zero));
        assert_eq!(Porta::try_from(80), Err(ErroPorta::Reservada(80)));
        let p: Result<Porta, _> = 1024_u16.try_into();
        assert!(p.is_ok());
    }

    #[test]
    fn parse() {
        assert_eq!("8080".parse::<Porta>(), Porta::try_from(8080));
        assert_eq!(" 3000\n".parse::<Porta>(), Porta::try_from(3000));
        assert_eq!("22".parse::<Porta>(), Err(ErroPorta::Reservada(22)));
        assert!(matches!(
            "70000".parse::<Porta>(),
            Err(ErroPorta::NaoNumerica(_))
        ));
        assert!(matches!(
            "http".parse::<Porta>(),
            Err(ErroPorta::NaoNumerica(_))
        ));
    }

    #[test]
    fn into_u16() {
        let p: Porta = "5432".parse().unwrap();
        let n: u16 = p.into();
        assert_eq!(n, 5432);
    }

    #[test]
    fn mensagens_e_causa() {
        assert_eq!(Porta::try_from(9000).unwrap().to_string(), ":9000");
        let e = "abc".parse::<Porta>().unwrap_err();
        assert_eq!(e.to_string(), "a porta precisa ser um número de 0 a 65535");
        assert!(e.source().is_some());
        assert!(ErroPorta::Zero.source().is_none());
    }

    #[test]
    fn interrogacao_com_box_dyn_error() {
        fn ler(texto: &str) -> Result<u16, Box<dyn Error>> {
            let p: Porta = texto.parse()?; // ErroPorta -> Box<dyn Error>
            Ok(p.into())
        }
        assert_eq!(ler("8443").unwrap(), 8443);
        assert_eq!(
            ler("443").unwrap_err().to_string(),
            "a porta 443 é reservada (use 1024 ou mais)"
        );
    }
}
```

<details>
<summary>Dicas</summary>

- Para o `?` do `parse` funcionar dentro de `from_str`, falta um `impl From<ParseIntError> for ErroPorta`.
- `"70000".parse::<u16>()` já falha com `ParseIntError` (não cabe em `u16`), então ele cai em `NaoNumerica`.
- O último teste não precisa de código novo: `Box<dyn Error>` aceita qualquer `Error` no `?`.

</details>

## 5. Faça compilar: *dyn compatibility* (médio)

Lição: [06](06-trait-objects-e-dyn.md).

Este programa de comandos de uma calculadora de pilha não compila:

```rust
// ❌ NÃO COMPILA
pub trait Comando {
    fn nome(&self) -> String;
    fn executar(&self, pilha: &mut Vec<i64>);
    fn clonar(&self) -> Self;
}

#[derive(Debug, PartialEq)]
pub struct Empilhar(pub i64);

#[derive(Debug, PartialEq)]
pub struct Somar;

impl Comando for Empilhar {
    fn nome(&self) -> String {
        format!("empilhar {}", self.0)
    }

    fn executar(&self, pilha: &mut Vec<i64>) {
        pilha.push(self.0);
    }

    fn clonar(&self) -> Self {
        Empilhar(self.0)
    }
}

impl Comando for Somar {
    fn nome(&self) -> String {
        String::from("somar")
    }

    fn executar(&self, pilha: &mut Vec<i64>) {
        let b = pilha.pop().unwrap_or(0);
        let a = pilha.pop().unwrap_or(0);
        pilha.push(a + b);
    }

    fn clonar(&self) -> Self {
        Somar
    }
}

pub fn rodar(programa: &[Box<dyn Comando>]) -> Vec<i64> {
    let mut pilha = Vec::new();
    for c in programa {
        c.executar(&mut pilha);
    }
    pilha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soma() {
        let programa: Vec<Box<dyn Comando>> =
            vec![Box::new(Empilhar(2)), Box::new(Empilhar(3)), Box::new(Somar)];
        assert_eq!(rodar(&programa), [5]);
        assert_eq!(programa[2].nome(), "somar");
    }
}
```

Resolva de **duas formas**, em dois módulos (`pub mod parte_a { ... }` e `pub mod parte_b { ... }`), e explique a diferença:

- **(a)** mantendo `fn clonar(&self) -> Self`, sem tirar o método da trait. Acrescente um teste que chama `Empilhar(7).clonar()`.
- **(b)** trocando `clonar` para que funcione **também** num `dyn Comando`. Escreva `pub fn repetir(programa: &[Box<dyn Comando>]) -> Vec<Box<dyn Comando>>`, que devolve o programa seguido de uma cópia dele mesmo, e um teste em que `rodar(&repetir(&programa))` dá `[5, 5]`.

<details>
<summary>Dicas</summary>

- (a): a lição mostra uma cláusula que tira um método da vtable.
- (b): o problema de `-> Self` é o tamanho desconhecido. Que tipo de retorno tem tamanho conhecido e ainda assim guarda "qualquer comando"?

</details>

## 6. O minigrep volta (desafio)

Lições: [02](02-definindo-e-implementando-traits.md), [03](03-trait-bounds-where-e-impl-trait.md) e [07](07-estatico-x-dinamico-e-composicao.md).

O [Projeto 1](../../projetos/01-minigrep/README.md) prometeu: "`run` passa a receber `out: &mut impl Write` e fica testável sem rodar o executável". Copie a sua solução do Projeto 1 (ou a [solução de referência](../../solucoes/projeto-01-minigrep/)) para uma pasta nova e faça três mudanças.

**Parte 1: `run` na biblioteca.** Mova `run` do `main.rs` para o `lib.rs`, com a assinatura

```rust
pub fn run(cfg: &Config, out: &mut impl Write) -> io::Result<u8>
```

As constantes `FOUND`, `NOT_FOUND` e `ERROR` vão junto (públicas). O `main` passa a chamar `run(&cfg, &mut io::stdout().lock())`. Escreva testes **unitários** no `lib.rs` que passam um `Vec<u8>` e conferem a saída e o código, pelo menos para: um arquivo com resultados; nenhum resultado; dois arquivos com `-n` (prefixo e número da linha); um arquivo que não existe seguido de um que existe. Os testes de `tests/cli.rs` continuam passando sem mudança.

**Parte 2: `Display` para `Input`.** Troque `Input::name() -> String` por `impl Display for Input`. Em `run`, o prefixo deixa de precisar de uma `String` temporária, e a armadilha do E0716 da etapa 6 some. Para isso, `render` passa a aceitar qualquer `Display` como nome: `render<N: Display>(matches, name: Option<N>, opts)`. (Os testes antigos de `render` que passam `None` vão precisar de uma ajuda para o compilador saber o que é `N`. Lembre do turbofish.)

**Parte 3 (para ir além): `-o ARQUIVO`.** Acrescente a opção `-o ARQUIVO` (ou `--output ARQUIVO`), que escreve o resultado num arquivo em vez do stdout. Sem `-o` no fim da linha de comando, o erro é `a opção -o precisa de um valor`. No `main`, o destino é stdout **ou** um arquivo, escolhido em tempo de execução: um caso para `Box<dyn Write>`. Acrescente um teste em `tests/cli.rs`.

<details>
<summary>Dicas</summary>

- Um teste unitário que quer testar o erro de escrita pode usar um `Write` próprio que sempre falha: `impl Write for Quebrado { fn write(...) -> io::Result<usize> { Err(...) } fn flush(...) }`.
- `Box<dyn Write>` implementa `Write` (é um blanket impl da std para `Box<W>` com `W: Write + ?Sized`), então `run(&cfg, &mut out)` funciona com `out: Box<dyn Write>`.
- Para o arquivo, embrulhe o `File` num `BufWriter`: sem buffer, cada `writeln!` vira uma chamada ao sistema operacional.
- `-o` consome o argumento seguinte, então o `for arg in args` não serve mais. Um laço com índice (`while i < args.len()`) resolve; o Módulo 07 mostra um jeito mais elegante com iteradores.

</details>

---

[Índice do módulo](README.md)
