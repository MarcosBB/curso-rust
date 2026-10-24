# Soluções — Módulo 06

Enunciados em [`modulos/06-traits-e-generics/exercicios.md`](../../modulos/06-traits-e-generics/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test
```

## 1. Faça compilar: generics sem bounds

Projeto: [`ex1-generics/`](ex1-generics/src/lib.rs).

| Erro | Causa | Correção |
|---|---|---|
| E0425 (4x) `cannot find type `T`` | `impl Pilha<T>` sem declarar `T` | `impl<T> Pilha<T>` |
| E0369 `>` em `&T` | `maior` compara | `T: PartialOrd` |
| E0599 `cmp` não encontrado | `menor_de` chama `cmp` | `T: Ord` |
| E0308 (2x) `expected T, found &T` | sem `Clone`, `x.clone()` clona a **referência** (`&T` é `Copy`) e devolve `&T` | `T: Clone` |
| E0277 `T` não implementa `Display` | `numerar` usa `{item}` | `T: Display` |

O E0308 é o mais traiçoeiro: o compilador não reclama que falta `clone`, porque `&T` sempre tem `clone` (toda referência compartilhada é `Copy`). Ele reclama do tipo do resultado. Repare também que `maior` só precisa de `PartialOrd` (os testes passam `f64`), enquanto `menor_de` usa `cmp`, que exige a ordem total `Ord`.

Depois disso, o clippy pede um `Default` para `Pilha` (`new_without_default`, lição 04). A tentação é `#[derive(Default)]`, mas o derive num tipo genérico gera `impl<T: Default> Default for Pilha<T>`: ele exige `T: Default`, mesmo sem precisar. O teste com `Pilha<std::fs::File>` mostra o problema, porque `File` não tem `Default`:

```text
error[E0277]: the trait bound `File: Default` is not satisfied
...
help: the trait `Default` is conditionally implemented for `Pilha<T>`
```

A solução escreve à mão, sem bound:

```rust
impl<T> Default for Pilha<T> {
    fn default() -> Self {
        Self::new()
    }
}
```

O mesmo vale para `Clone`, `PartialEq` e companhia: o `derive` sempre acrescenta o bound em cada parâmetro de tipo.

## 2. Passe nos testes: a trait `Forma`

Projeto: [`ex2-formas/`](ex2-formas/src/lib.rs).

```rust
pub trait Forma: fmt::Display {
    fn area(&self) -> f64;
    fn perimetro(&self) -> f64;

    fn descricao(&self) -> String {
        format!(
            "{self}: área {:.2}, perímetro {:.2}",
            self.area(),
            self.perimetro()
        )
    }
}
```

A supertrait `Display` é o que permite ao método padrão usar `{self}` sem saber o tipo concreto, e é o que permite ao teste fazer `maior.to_string()` num `&dyn Forma`: os métodos das supertraits também entram na vtable.

```rust
pub fn maior_area(formas: &[Box<dyn Forma>]) -> Option<&dyn Forma> {
    let mut maior: Option<&dyn Forma> = None;
    for f in formas {
        match maior {
            Some(m) if m.area() >= f.area() => {}
            _ => maior = Some(f.as_ref()),
        }
    }
    maior
}
```

`f` é um `&Box<dyn Forma>`, e `as_ref()` empresta o conteúdo da caixa como `&dyn Forma`. O lifetime do retorno é o do slice (elisão, Módulo 03): a forma devolvida aponta para dentro de `formas`. Com iteradores (Módulo 07), o corpo vira um `max_by` de uma linha.

`area_total<T: Forma>` mostra a outra face da mesma trait: só aceita formas de um tipo, mas não precisa de `Box`.

## 3. Um logger decorador

Projeto: [`ex3-logger/`](ex3-logger/src/lib.rs).

```rust
impl<L: Logger> Logger for FiltroNivel<L> {
    fn log(&mut self, nivel: Nivel, mensagem: &str) {
        if nivel >= self.minimo {
            self.interno.log(nivel, mensagem);
        }
    }
}
```

O filtro é um `Logger` que **tem** um `Logger`: composição, sem herança. Como ele implementa a mesma trait, os filtros se empilham (`FiltroNivel<FiltroNivel<MemoriaLogger>>`, testado em `filtros_se_empilham`), e podem ir para um `Vec<Box<dyn Logger>>` junto com os outros (`funciona_como_dyn`). Os atalhos `info` e `erro` são métodos padrão: o filtro ganha os dois sem escrever nada, e eles passam pelo `log` filtrado.

`FiltroNivel` é genérico em `L` (e não guarda um `Box<dyn Logger>`) porque o tipo de dentro é conhecido na compilação: zero indireção. Se o logger de dentro fosse escolhido em tempo de execução, você guardaria um `Box<dyn Logger>`. Para ele caber em `FiltroNivel<L>`, falta um `impl Logger for Box<dyn Logger>` que delegue com `(**self).log(nivel, mensagem)`: a std tem esse tipo de implementação para as traits dela (como `Write`), mas não tem como ter para as suas. Fica como exercício extra.

## 4. Conversões de `Porta`

Projeto: [`ex4-porta/`](ex4-porta/src/lib.rs).

```rust
impl FromStr for Porta {
    type Err = ErroPorta;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numero: u16 = s.trim().parse()?; // ParseIntError -> ErroPorta
        Porta::try_from(numero)
    }
}
```

`from_str` reaproveita o `try_from`: as regras de validação ficam num lugar só. O `?` só funciona por causa do `impl From<ParseIntError> for ErroPorta`, e o último teste mostra o encaixe com o Módulo 05: `ErroPorta` implementa `Error`, então vira `Box<dyn Error>` no `?` sem código extra.

Com o campo privado, `Porta` só existe válida: quem recebe uma `Porta` nunca precisa conferir a faixa de novo. O `From<Porta> for u16` dá o `into()` de volta, de graça.

## 5. Faça compilar: *dyn compatibility*

Projeto: [`ex5-comandos/`](ex5-comandos/src/lib.rs). O erro original:

```text
error[E0038]: the trait `Comando` is not dyn compatible
...
   |     fn clonar(&self) -> Self;
   |                         ^^^^ ...because method `clonar` references the `Self` type in its return type
```

**(a) `where Self: Sized`.** O método sai da vtable, e a trait volta a ser dyn compatible:

```rust
fn clonar(&self) -> Self
where
    Self: Sized;
```

`Empilhar(7).clonar()` continua funcionando, mas `programa[0].clonar()` num `Box<dyn Comando>` não compila. Bom quando o método só faz sentido em tipos concretos.

**(b) `-> Box<dyn Comando>`.** O problema era o tamanho desconhecido do retorno; um `Box` tem tamanho fixo (é um fat pointer):

```rust
fn clonar(&self) -> Box<dyn Comando>;

impl Comando for Empilhar {
    fn clonar(&self) -> Box<dyn Comando> {
        Box::new(self.clone())
    }
    // ...
}
```

Agora dá para clonar via `dyn`, e `repetir` funciona. O custo: cada tipo repete a mesma linha `Box::new(self.clone())`. É exatamente o que a crate `dyn-clone` automatiza.

A escolha: (a) quando ninguém precisa clonar um `dyn Comando`; (b) quando precisa.

## 6. O minigrep volta

Projeto: [`ex6-minigrep/`](ex6-minigrep/). É uma cópia da [solução do Projeto 1](../projeto-01-minigrep/) com as três partes:

```bash
cd ex6-minigrep
cargo test                           # 33 unitários + 11 de integração
cargo run -q -- -n -o saida.txt to poem.txt && cat saida.txt
```

**Parte 1.** `run` foi para o [`lib.rs`](ex6-minigrep/src/lib.rs) com `out: &mut impl Write`, e o corpo mudou pouco: `stdout` virou `out`. O módulo `run_tests` tem cinco testes que rodam em memória, sem processo:

```rust
fn run_to_string(cfg: &Config) -> (u8, String) {
    let mut out: Vec<u8> = Vec::new();
    let code = run(cfg, &mut out).unwrap();
    (code, String::from_utf8(out).unwrap())
}
```

Um deles usa um `Write` próprio que sempre falha (`Quebrado`), para testar que o erro de escrita sobe pelo `?`. Antes, esse caminho só era testável fechando um pipe de verdade (o teste `closed_pipe_is_not_a_panic` de `tests/cli.rs`).

**Parte 2.** Em [`config.rs`](ex6-minigrep/src/config.rs), `impl fmt::Display for Input` substitui `name()`. Em `run`, o nome vira só `Some(input)`, um `Option<&Input>`:

```rust
let name = if show_names { Some(input) } else { None };
for line in render(&matches, name, cfg.output) {
```

Sem `String` temporária, não há nada para ser destruído antes da hora, e o E0716 da etapa 6 não tem como acontecer. `render` ficou genérica em `N: Display`, e os testes antigos que passavam `None` precisaram do turbofish, `None::<&str>`: sem nenhum `Some`, o compilador não tem como inferir `N`.

**Parte 3.** `Config` ganhou `output_file: Option<PathBuf>`, e `from_args` virou um `while` com índice, porque `-o` consome o argumento seguinte. No [`main.rs`](ex6-minigrep/src/main.rs), o destino é decidido em tempo de execução:

```rust
let mut out: Box<dyn Write> = match &cfg.output_file {
    None => Box::new(io::stdout().lock()),
    Some(path) => match File::create(path) {
        Ok(file) => Box::new(BufWriter::new(file)),
        Err(e) => { /* mensagem e ExitCode::from(ERROR) */ }
    },
};
```

Os dois braços têm tipos diferentes (`StdoutLock` e `BufWriter<File>`), e é por isso que `impl Write` não serviria aqui (lição 03: `impl Trait` é um tipo só). `run` continua genérica: ela recebe `&mut Box<dyn Write>`, e `Box<dyn Write>` implementa `Write` pelo blanket impl da std. As duas formas convivem: generics na biblioteca, `dyn` só no ponto em que a escolha é feita.
