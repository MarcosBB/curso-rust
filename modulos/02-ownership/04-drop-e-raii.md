# 04 — Drop e RAII

**Objetivos.** Ao final, você vai saber:

- em que momento exato cada valor é descartado, e em que ordem;
- descartar um valor antes do fim do escopo com `drop()`;
- a diferença entre `let _ =` e `let _x =`, que surpreende muita gente;
- como o `Drop` do Rust se compara a destrutores do C++, ao `with` do Python e a coletores de lixo.

## Um instrumento de laboratório

Para *ver* quando um valor é descartado, vamos usar um tipo que imprime uma linha no momento do descarte:

```rust
/// Tipo de demonstração: imprime uma linha quando é descartado.
struct Barulhento(&'static str);

impl Drop for Barulhento {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}
```

Use-o como um instrumento pronto. `Barulhento("a")` cria um valor com o nome `"a"` (o `&'static str` é um texto literal, assunto da [lição 05](05-string-e-str.md)), e o bloco `impl Drop` diz o que fazer no descarte. Escrever o seu próprio `impl Drop`, e saber quando isso vale a pena, fica para o Módulo 10. Neste módulo, a gente só **usa** o `Drop` e observa.

Tipos como `String`, `Vec` e `File` têm um `Drop` parecido na biblioteca padrão. O deles libera o heap ou fecha o arquivo, em vez de imprimir.

## A ordem dos descartes

O programa [`codigo/ordem-do-drop`](codigo/ordem-do-drop/) exercita cada situação:

```rust
fn cria(nome: &'static str) -> Barulhento {
    Barulhento(nome)
}

fn consome(b: Barulhento) {
    println!("consome recebeu {}", b.0);
} // `b` é descartado aqui, no fim da função que virou dona

fn escopo() {
    let a = Barulhento("a");
    let b = Barulhento("b");
    println!("criei {} e {}", a.0, b.0);

    {
        let c = Barulhento("c");
        println!("dentro do bloco: {}", c.0);
    } // `c` é descartado aqui
    println!("depois do bloco");

    let mut d = Barulhento("d1");
    println!("d = {}", d.0);
    d = Barulhento("d2"); // o valor antigo ("d1") é descartado na hora
    println!("d = {}", d.0);

    drop(b); // move `b` para `drop`, que o descarta
    println!("depois do drop(b)");

    consome(Barulhento("f"));
    println!("depois de consome");

    let _ = cria("temporario"); // `_` não liga o valor: descarte imediato
    let _g = cria("g"); // `_g` é uma variável de verdade: vive até o fim
    println!("fim de escopo()");
} // descarta o que sobrou, em ordem inversa: g, d2, a
```

Saída real:

```text
criei a e b
dentro do bloco: c
  drop c
depois do bloco
d = d1
  drop d1
d = d2
  drop b
depois do drop(b)
consome recebeu f
  drop f
depois de consome
  drop temporario
fim de escopo()
  drop g
  drop d2
  drop a
```

Cada linha `drop` tem uma regra por trás:

| Momento | Regra |
|---|---|
| `drop c` logo depois de "dentro do bloco" | um bloco interno descarta as suas variáveis no próprio `}` |
| `drop d1` entre os dois `println!` de `d` | **reatribuir** descarta o valor antigo na hora |
| `drop b` antes de "depois do drop(b)" | `drop(b)` descarta antes do fim do escopo |
| `drop f` dentro de `consome` | quem recebe por valor vira dono, e descarta no fim **da função** |
| `drop temporario` logo após a criação | `let _ =` não guarda o valor, que é descartado no fim da instrução |
| `drop g`, `drop d2`, `drop a` no fim | as variáveis restantes saem em **ordem inversa** da declaração |

E repare no que **não** aparece: não há um segundo `drop b` no fim do escopo. `b` foi movida para `drop()`, e um valor movido não é descartado na origem. É a regra 2 do ownership em ação: um dono só, um descarte só.

A ordem inversa tem um motivo: uma variável declarada depois pode depender de uma declarada antes (por exemplo, guardar uma referência para ela). Descartando de trás para frente, nada é destruído enquanto alguém ainda pode precisar.

### Sombreamento, coleções e campos

A segunda parte do mesmo programa mostra mais três casos:

```rust
fn colecoes() {
    let x = Barulhento("x1");
    println!("x = {}", x.0);
    let x = Barulhento("x2"); // sombreamento NÃO descarta "x1"
    println!("x = {}", x.0);

    let mut v = Vec::new();
    for nome in ["v0", "v1", "v2"] {
        v.push(Barulhento(nome)); // o Vec vira dono de cada item
    }
    println!("vetor com {} itens", v.len());

    let p = Par {
        primeiro: Barulhento("p.primeiro"),
        segundo: Barulhento("p.segundo"),
    };
    println!("par: {} e {}", p.primeiro.0, p.segundo.0);
    println!("fim de colecoes()");
}
```

```text
x = x1
x = x2
vetor com 3 itens
par: p.primeiro e p.segundo
fim de colecoes()
  drop p.primeiro
  drop p.segundo
  drop v0
  drop v1
  drop v2
  drop x2
  drop x1
```

- **Sombreamento não é reatribuição.** O segundo `let x` cria uma variável **nova**, e a primeira (`x1`) continua viva, só que sem nome acessível, até o fim do escopo. Compare com `d = ...` acima, que descartou `d1` na hora.
- **Um `Vec` descarta os itens na ordem**, do primeiro ao último, quando ele mesmo é descartado.
- **Os campos de um struct são descartados na ordem de declaração** quando o struct é descartado. (`Par` é um struct com dois campos `Barulhento`; a definição está no projeto.) As variáveis saem em ordem inversa, mas os campos e os itens saem em ordem direta.

## `drop()` é uma função vazia

O `drop` que você chama não tem nada de especial. Ele está no *prelude* (não precisa de `use`), e a própria documentação da biblioteca padrão mostra que a implementação é só isto:

```rust
pub fn drop<T>(_x: T) {}
```

(Na fonte atual a assinatura tem alguns detalhes a mais, um `const` e uma cláusula `where`, que não mudam nada aqui. O `<T>` diz que ela aceita um valor de qualquer tipo: *generics* são o Módulo 06.)

Ela recebe o valor **por valor** (um move) e não faz nada. O descarte acontece porque o parâmetro `_x` sai de escopo no `}` da função, como em qualquer outra função que toma posse. Por isso, depois de `drop(v)`, usar `v` dá o E0382 de sempre:

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = vec![1, 2, 3];
    drop(v);
    println!("{v:?}");
}
```

```text
error[E0382]: borrow of moved value: `v`
 --> src/main.rs:4:16
  |
2 |     let v = vec![1, 2, 3];
  |         - move occurs because `v` has type `Vec<i32>`, which does not implement the `Copy` trait
3 |     drop(v);
  |          - value moved here
4 |     println!("{v:?}");
  |                ^ value borrowed here after move
...
```

Quando usar `drop()`? Quando você quer liberar um recurso **antes** do fim do escopo: fechar um arquivo antes de uma operação demorada, ou soltar um *lock* (Módulo 11) o quanto antes.

## `let _ =` x `let _x =`

As duas formas evitam o aviso de variável não usada, mas fazem coisas diferentes:

- `let _x = valor;` cria uma variável chamada `_x`. O sublinhado só diz ao compilador "eu sei que não vou usar". Ela vive até o fim do escopo, como qualquer variável.
- `let _ = valor;` **não cria variável nenhuma**. O `_` é um *pattern* que aceita qualquer coisa e não guarda nada. Se o valor era um temporário, ele é descartado no fim da instrução.

> [!WARNING]
> A diferença parece detalhe até o dia em que o valor é um *guard*. Com `Mutex` (Módulo 11), `let _ = m.lock().unwrap();` adquire o lock e o solta **na mesma linha**, e o código seguinte roda sem proteção. Já `let _guard = m.lock().unwrap();` segura o lock até o fim do escopo. Quando o valor representa um recurso, dê um nome a ele.

## RAII não é só memória

O mesmo mecanismo serve para qualquer recurso. Um `File` fecha o arquivo no `drop`, e por isso não existe um método `close()` que você precise lembrar de chamar:

```rust
use std::fs::File;
use std::io::Write;

fn main() {
    {
        let mut f = File::create("saida.txt").expect("não consegui criar o arquivo");
        writeln!(f, "olá").expect("falha ao escrever");
    } // `f` sai de escopo: o arquivo é fechado aqui
    println!("{}", std::fs::read_to_string("saida.txt").unwrap());
}
```

(O `expect` encerra o programa com a mensagem se a operação falhar. O tratamento de erros de verdade é o Módulo 05.)

Em Python, o equivalente é o `with`:

```python
with open("saida.txt", "w") as f:
    f.write("olá\n")
# o arquivo é fechado aqui
```

A diferença é que, em Python, o `with` é **opcional**: sem ele, o arquivo fecha quando o objeto for coletado, e no CPython isso costuma ser logo, graças à contagem de referências, mas a linguagem não garante. Em Rust, todo dono funciona como se estivesse dentro de um `with`, sempre.

## Comparando com C++, Python e coletores de lixo

**C++.** `impl Drop` corresponde ao destrutor `~T()`. As variáveis também são destruídas em ordem inversa da declaração. A grande diferença está no objeto movido:

```cpp
struct Barulhento {
    std::string nome;
    explicit Barulhento(std::string n) : nome(std::move(n)) {}
    Barulhento(Barulhento&&) = default;
    ~Barulhento() { std::cout << "destrutor [" << nome << "]\n"; }
};

int main() {
    Barulhento a("a");
    Barulhento b = std::move(a);
    std::cout << "fim do main\n";
}
```

```text
fim do main
destrutor [a]
destrutor []
```

São **dois** destrutores: o de `b` e o de `a`, que continua existindo, vazia, depois do move. Todo tipo C++ com move precisa deixar a origem num estado que o destrutor aceite. Em Rust, o valor movido simplesmente não é descartado na origem, e por isso o `Drop` nunca precisa lidar com um objeto "meio morto".

Um detalhe para quem conhece bem C++: lá, os **membros** de uma classe são destruídos em ordem inversa da declaração. Em Rust, os campos de um struct são descartados na ordem da declaração, como você viu acima.

**Python.** Um objeto pode ter `__del__`, que roda quando ele é coletado. No CPython, isso acontece quando a contagem de referências zera, o que costuma ser previsível. Mas com ciclos de referências o objeto espera o coletor de ciclos, e outras implementações (como o PyPy) não usam contagem de referências. Por isso o código Python que gerencia recursos usa `with`, e não `__del__`.

**Coletores de lixo e ARC.** Linguagens com GC (Java, Go, C#) liberam memória em algum momento depois que o objeto fica inalcançável. Swift e Objective-C usam ARC (*Automatic Reference Counting*), em que o compilador insere incrementos e decrementos de um contador. O Rust faz parecido para o dono único, mas sem contador: o compilador sabe **na compilação** onde cada dono morre. Quando você precisa de vários donos, existe `Rc`/`Arc` (Módulo 10), que é contagem de referências explícita.

| | Quando o recurso é liberado | Quem decide |
|---|---|---|
| C++ (RAII) | fim do escopo, em ordem inversa | o compilador, mas objetos movidos ainda são destruídos |
| Python | quando o refcount zera (CPython) ou o GC roda; `with` para garantir | o runtime |
| GC (Java, Go) | em algum momento depois de o objeto ficar inalcançável | o runtime |
| ARC (Swift) | quando o contador zera | o runtime, com o contador inserido pelo compilador |
| Rust | fim do escopo do dono, reatribuição ou `drop()` | o compilador, uma vez por valor |

## `Drop` também não garante tudo

Para ser justo: o Rust garante que um valor não é descartado **duas vezes** e que ninguém o usa **depois** do descarte. Ele não garante que o descarte aconteça. `std::mem::forget(valor)` é uma função segura que consome o valor sem chamar o `drop`, e dois `Rc` que apontam um para o outro nunca chegam a zero (Módulo 10). Vazar é um bug, mas não corrompe memória, e por isso não precisa de `unsafe`.

## Armadilhas comuns

### Chamar o método `.drop()`

```rust
// ❌ NÃO COMPILA
fn main() {
    let v = vec![1, 2, 3];
    v.drop();
}
```

```text
error[E0040]: explicit use of destructor method
 --> src/main.rs:3:7
  |
3 |     v.drop();
  |       ^^^^ explicit destructor calls not allowed
  |
help: consider using `drop` function
  |
3 -     v.drop();
3 +     drop(v);
  |
```

Se o método pudesse ser chamado, `v` continuaria existindo e seria descartada de novo no fim do escopo: um *double drop*. A função `drop(v)` **move** `v`, e o compilador sabe que não deve descartá-la outra vez.

### Tipo com `Drop` não pode ser `Copy`

```rust
// ❌ NÃO COMPILA
#[derive(Clone, Copy)]
struct Recurso(u32);

impl Drop for Recurso {
    fn drop(&mut self) {
        println!("libera {}", self.0);
    }
}
```

```text
error[E0184]: the trait `Copy` cannot be implemented for this type; the type has a destructor
 --> src/main.rs:2:8
  |
1 | #[derive(Clone, Copy)]
  |                 ---- in this derive macro expansion
2 | struct Recurso(u32);
  |        ^^^^^^^ `Copy` not allowed on types with destructors
  |
note: destructor declared here
 --> src/main.rs:5:5
  |
5 |     fn drop(&mut self) {
  |     ^^^^^^^^^^^^^^^^^^
```

Se `Recurso` fosse `Copy`, cada `let r2 = r1;` criaria mais um valor para descartar, e o mesmo recurso seria "liberado" várias vezes. `Copy` e `Drop` são mutuamente exclusivos. Tire o `Copy` e, se precisar de cópias, use `Clone`.

### Esperar que sombreamento descarte

`let x = ...;` duas vezes no mesmo escopo **não** descarta o primeiro valor. Se a intenção é trocar o valor e liberar o antigo agora, use `let mut x` e reatribua (`x = ...`).

### `let _ =` com recursos

Veja o aviso acima. Na dúvida, dê um nome com sublinhado (`_guard`, `_arquivo`).

## Resumo

- Um valor é descartado quando o dono sai de escopo, quando a variável recebe outro valor ou quando é movido para `drop()`.
- No mesmo escopo, as variáveis são descartadas em ordem inversa da declaração. Itens de um `Vec` e campos de um struct, em ordem direta.
- Um valor movido não é descartado na origem: cada valor é descartado exatamente uma vez.
- `drop()` é uma função vazia que toma posse. Chamar `.drop()` é proibido (E0040).
- `let _ =` descarta na hora; `let _x =` vive até o fim do escopo. Sombrear não descarta.
- RAII vale para qualquer recurso (arquivos, locks, conexões), sem precisar de `with` nem de `close()`.
- `Copy` e `Drop` não se misturam (E0184). Implementar o seu próprio `Drop` fica para o Módulo 10.

## Exercícios

Veja o exercício 3 em [`exercicios.md`](exercicios.md).

## Leitura complementar

- [The Rust Programming Language, 15.3 Running Code on Cleanup with the Drop Trait](https://doc.rust-lang.org/stable/book/ch15-03-drop.html)
- [The Rust Reference: Destructors](https://doc.rust-lang.org/reference/destructors.html) (a ordem exata de descarte, inclusive de temporários)
- [Rust by Example: RAII](https://doc.rust-lang.org/rust-by-example/scope/raii.html)
- [Comprehensive Rust: Drop](https://google.github.io/comprehensive-rust/memory-management/drop.html)
- Documentação de [`std::mem::drop`](https://doc.rust-lang.org/std/mem/fn.drop.html)

---

[← Lição anterior](03-copy-e-clone.md) · [Índice do módulo](README.md) · [Próxima lição →](05-string-e-str.md)
