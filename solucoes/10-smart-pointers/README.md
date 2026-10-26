# Soluções — Módulo 10

Enunciados em [`modulos/10-smart-pointers/exercicios.md`](../../modulos/10-smart-pointers/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test
```

## 1. Árvore de busca com `Box`

Projeto: [`ex1-arvore-busca/`](ex1-arvore-busca/src/lib.rs).

O código inicial dá o erro da lição 01:

```text
error[E0072]: recursive type `No` has infinite size
  --> src/lib.rs:9:1
   |
 9 | struct No {
   | ^^^^^^^^^
10 |     valor: i32,
11 |     esq: Option<No>,
   |                 -- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
11 |     esq: Option<Box<No>>,
   |                 ++++  +
```

O `Option` não resolve sozinho: um `Option<No>` guarda o `No` inteiro dentro de si. A correção vale para `esq`, `dir` **e** `raiz` (esta última não é recursiva, mas usar o mesmo tipo deixa o código de inserção uniforme): `Option<Box<No>>`, que tem 8 bytes.

A inserção usa um cursor do tipo `&mut Option<Box<No>>`, que aponta para "o lugar onde o valor deve entrar":

```rust
let mut cursor = &mut self.raiz;
while let Some(no) = cursor {
    if valor == no.valor {
        return false;
    }
    cursor = if valor < no.valor { &mut no.esq } else { &mut no.dir };
}
*cursor = Some(Box::new(No { valor, esq: None, dir: None }));
```

Quando o laço termina, `cursor` aponta para um `None`, e a atribuição pendura o nó novo ali. Não há recursão nem caso especial para a árvore vazia. As leituras usam `as_deref()`, que transforma `&Option<Box<No>>` em `Option<&No>`, e funções internas recursivas para `em_ordem` e `altura`. A recursão é segura aqui porque a profundidade é a altura da árvore; numa árvore degenerada com milhões de nós (inserções em ordem), ela estouraria a pilha, e aí a solução seria um laço com uma pilha explícita (`Vec<&No>`).

## 2. Um ponteiro que conta os descartes

Projeto: [`ex2-rastreado/`](ex2-rastreado/src/lib.rs).

```rust
impl<T> Deref for Rastreado<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.valor
    }
}

impl<T> DerefMut for Rastreado<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.valor
    }
}

impl<T> Drop for Rastreado<'_, T> {
    fn drop(&mut self) {
        self.drops.update(|n| n + 1);
    }
}
```

**Por que `&Cell<u32>` e não `&mut u32`?** Os testes criam vários `Rastreado` que apontam para o mesmo contador ao mesmo tempo (três no `Vec`). Com `&mut u32`, seriam três empréstimos exclusivos do mesmo valor, o E0499 do Módulo 03. Com `&Cell<u32>`, são três empréstimos compartilhados, e o `Cell` permite alterar através deles. É o caso de uso mais simples de mutabilidade interior.

**Por que colocar no `Vec` não descarta?** Porque é um *move*: a posse passa da closure para o `Vec`, e o valor continua vivo. O `drop` roda uma vez por valor, quando o último dono o descarta, aqui no `drop(lista)`. O teste `tamanho(&r)` mostra a *deref coercion* em duas etapas: `&Rastreado<String>` → `&String` → `&str`.

## 3. Conserte os pânicos

Projeto: [`ex3-banco/`](ex3-banco/src/lib.rs).

**`aplicar_juros`**: o `for` segura o `Ref` de `self.contas.borrow()` durante o laço inteiro, e `depositar` pede um `RefMut` lá dentro. Mesmo que o C++ deixasse, seria um bug: inserir num `HashMap` durante a iteração pode realocar a tabela. A correção separa a leitura da escrita:

```rust
let juros: Vec<(String, i64)> = self
    .contas
    .borrow()
    .iter()
    .filter(|&(_, &saldo)| saldo > 0)
    .map(|(nome, saldo)| (nome.clone(), saldo * taxa / 100))
    .collect(); // o `Ref` morre aqui, no fim da instrução
for (nome, valor) in juros {
    self.depositar(&nome, valor);
}
```

**`saldo_ou_abrir`**: o temporário criado na expressão de um `match` vive até o fim do `match` inteiro, inclusive no braço `None`. A correção é terminar o empréstimo antes de decidir. `self.saldo(nome)` já faz isso: devolve um `Option<i64>` copiado (com `.copied()`), e o `Ref` morre dentro dele.

**Por que `primeiro_deposito` não dá pânico?** Porque é um `if let`, e na edition 2024 o temporário da condição de um `if let` é descartado antes do `else`. Na edition 2021, ele também daria `RefCell already borrowed`. O `match` não mudou na 2024.

## 4. Organograma com `Rc` e `Weak`

Projeto: [`ex4-organograma/`](ex4-organograma/src/lib.rs).

```rust
pub fn contratar(chefe: &Rc<Funcionario>, nome: &str) -> Rc<Funcionario> {
    let novo = Funcionario::novo(nome);
    *novo.chefe.borrow_mut() = Rc::downgrade(chefe);
    chefe.equipe.borrow_mut().push(Rc::clone(&novo));
    novo
}

pub fn cadeia_de_comando(&self) -> Vec<String> {
    let mut nomes = vec![self.nome.clone()];
    let mut atual = self.chefe();
    while let Some(f) = atual {
        nomes.push(f.nome.clone());
        atual = f.chefe();
    }
    nomes
}
```

Em `cadeia_de_comando`, cada `f` é um `Rc` temporário obtido com `upgrade`: ele mantém o chefe vivo só durante uma volta do laço. `tamanho_da_equipe` soma `1 + f.tamanho_da_equipe()` para cada membro, recursivamente.

Se o campo `chefe` fosse `RefCell<Option<Rc<Funcionario>>>`, o ceo seria dono do dev (pela equipe) e o dev seria dono do ceo (pelo chefe). O teste `sem_vazamento` falharia: depois do `drop(ceo)`, o contador forte do ceo continuaria em 1, e o `observador.upgrade()` devolveria `Some`.

## 5. Normalizar espaços sem alocar à toa

Projeto: [`ex5-espacos/`](ex5-espacos/src/lib.rs).

```rust
pub fn normalizar_espacos(s: &str) -> Cow<'_, str> {
    let aparado = s.trim();
    if ja_normalizado(aparado) {
        return Cow::Borrowed(aparado);
    }
    let mut saida = String::with_capacity(aparado.len());
    for (i, palavra) in aparado.split_whitespace().enumerate() {
        if i > 0 {
            saida.push(' ');
        }
        saida.push_str(palavra);
    }
    Cow::Owned(saida)
}
```

**Por que tirar as pontas ainda é empréstimo?** `trim` não cria texto novo: devolve uma fatia de `s` que começa e termina em outros pontos. Uma fatia é um ponteiro e um comprimento (Módulo 03), e qualquer pedaço contínuo de `s` pode ser representado assim. O que não é contínuo, como "bom" + " " + "dia" tirados de `"bom   dia"`, precisa de memória nova.

`ja_normalizado` percorre os caracteres uma vez, recusando qualquer espaço em branco que não seja `' '` ou que venha logo depois de outro. Uma alternativa mais curta, e que aloca sempre, seria comparar com o resultado de `split_whitespace().collect::<Vec<_>>().join(" ")`. Ela funciona, mas joga fora o motivo de usar `Cow`.
