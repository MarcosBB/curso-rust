# Soluções — Módulo 13

Enunciados em [`modulos/13-tipos-e-traits-avancados/exercicios.md`](../../modulos/13-tipos-e-traits-avancados/exercicios.md).

Cada solução é um projeto Cargo com os testes do enunciado. Para conferir:

```bash
cd nome-do-projeto
cargo test    # roda os testes e os doctests (inclusive os `compile_fail`)
```

## 1. Dinheiro com operadores

Projeto: [`ex1-dinheiro/`](ex1-dinheiro/src/lib.rs).

Com o código inicial, os testes nem compilam, e os nove erros resumem a lição 01: E0369 para `a + b`, `a - b` e `preco * 3` (*"cannot add `Dinheiro` to `Dinheiro`"*), E0600 para `-a` (*"cannot apply unary operator `-`"*), E0368 para `+=`, E0277 *"cannot multiply `{integer}` by `Dinheiro`"* para `3 * preco`, e E0277 *"a value of type `Dinheiro` cannot be made by summing an iterator over elements of type `&Dinheiro`"* para o `iter().sum()`.

O que merece atenção:

- `3 * preco` é `impl Mul<Dinheiro> for i64`, que delega para `Dinheiro * i64`. A orphan rule permite, porque `Dinheiro` é do crate.
- São duas `impl Sum`: uma para itens `Dinheiro` (o `copied()`) e outra para `&'a Dinheiro`, que só repassa: `iter.copied().sum()`.
- `Dinheiro` é `Copy`, então os operadores por valor bastam. Com um tipo dono de memória, seria a vez de `impl Add<&T> for &T`.
- No `Display`, `self.0.unsigned_abs()` evita o caso de `i64::MIN`, cujo `abs()` estoura. O sinal vai à parte, para `-5` virar `-R$ 0,05` (e não `R$ 0,-5`).

```rust
impl fmt::Display for Dinheiro {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sinal = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        write!(f, "{sinal}R$ {},{:02}", abs / 100, abs % 100)
    }
}
```

## 2. Conexão com typestate

Projeto: [`ex2-conexao/`](ex2-conexao/src/lib.rs).

Os estados são structs vazias, e a conexão carrega o estado num `PhantomData<E>`. O truque que encurta tudo é uma transição genérica privada, que só troca o parâmetro de tipo:

```rust
impl<E> Conexao<E> {
    fn para<F>(self) -> Conexao<F> {
        Conexao {
            endereco: self.endereco,
            tentativas: self.tentativas,
            enviadas: self.enviadas,
            _estado: PhantomData,
        }
    }
}

impl Conexao<Conectando> {
    pub fn confirmar(self) -> Conexao<Conectada> {
        self.para()
    }

    pub fn falhar(self) -> Conexao<Desconectada> {
        self.para()
    }
}
```

O `para` é privado: de fora, ninguém pula de `Desconectada` para `Conectada`. Os métodos comuns (`estado`, `endereco`, `tentativas`) ficam num `impl<E: Estado> Conexao<E>`, com o nome do estado numa constante associada.

Os três doctests `compile_fail` confirmam o que o tipo impede: E0599 para `enviar` numa `Conexao<Desconectada>` e para `desconectar` numa `Conexao<Conectando>`, e E0382 para confirmar duas vezes a mesma tentativa. O `,EXXXX` importa: sem ele, um doctest com um erro de digitação passaria do mesmo jeito.

Compare com o código inicial: lá, `enviar` devolvia `Result` e todo chamador tinha de tratar um erro que, numa sequência bem escrita, nunca acontece. Com typestate, `enviar` não falha, porque só existe quando faz sentido.

**Desafio extra (builder).** O mesmo padrão da lição 04: `ConexaoBuilder<SemEndereco>` com `endereco(self, e) -> ConexaoBuilder<ComEndereco>`, opcionais num `impl<U>` e `build` só em `ConexaoBuilder<ComEndereco>`, devolvendo `Conexao<Desconectada>`.

## 3. Extension traits e uma trait selada

Projeto: [`ex3-extensoes/`](ex3-extensoes/src/lib.rs).

Os bounds ficam nos **métodos**, e não na trait:

```rust
pub trait SliceExt<T> {
    fn segundo_maior(&self) -> Option<&T>
    where
        T: Ord;

    fn contagem_de(&self, x: &T) -> usize
    where
        T: PartialEq;
}

impl<T> SliceExt<T> for [T] { /* ... */ }
```

Assim, `[0.5, 0.5].contagem_de(&0.5)` funciona (`f64` é `PartialEq`), mesmo `f64` não sendo `Ord`. Com `trait SliceExt<T: Ord>`, esse teste não compilaria.

`segundo_maior` acha o maior e depois o maior dos que são **estritamente menores**, o que trata repetições (`[3, 9, 4, 9]` dá `4`) e devolve `None` para `[7, 7]`. Implementar para `[T]` cobre arrays e `Vec` pela coerção.

O selo é o da lição 05: `mod selo { pub trait Selado {} }`, privado, e `pub trait Moeda: selo::Selado`. Os três tipos ganham `impl selo::Selado`. O doctest roda como **outro crate**, então ali o selo vale, e `impl Moeda for Iene` (um `Iene` do doctest) dá E0277 com a nota *"`Moeda` is a "sealed trait""*. Dentro de `ex3-extensoes`, nada impede implementar: o selo protege contra os outros crates.

Em `formatar`, as constantes associadas fazem o trabalho: `10_i64.pow(M::CASAS)` é a base, e `{:0largura$}` com `largura = M::CASAS as usize` completa os zeros. O `Iene`, sem casas decimais, sai antes.

## 4. Quatro erros de lifetime

Projeto: [`ex4-lifetimes/`](ex4-lifetimes/src/lib.rs).

Com o código inicial, são nove erros na lib e mais um nos testes. As quatro correções, todas na assinatura:

```rust
// (a) o iterador não captura o empréstimo de `v`
pub fn indices_do_meio(v: &[u8]) -> impl Iterator<Item = usize> + use<> {

// (b) T vive pelo menos 'a
pub fn guardar<'a, T: Debug + 'a>(caixas: &mut Vec<Box<dyn Debug + 'a>>, x: T) {

// (c) para qualquer empréstimo, não um 'a escolhido por quem chama
pub fn extremos<C>(c: C) -> Option<(i32, i32)>
where
    for<'a> &'a C: IntoIterator<Item = &'a i32>,

// (d) o log guarda referências de lifetime 'a, e o evento vive 'a
pub fn registrar<'a>(log: &mut Vec<&'a str>, evento: &'a str) {
```

- **(a)** O E0502 do teste traz a nota *"this call may capture more lifetimes than intended, because Rust 2024 has adjusted the `impl Trait` lifetime capture rules"*. `1..v.len()-1` só guarda números, então `use<>` é verdade, e o compilador confere.
- **(b)** E0309, com o `help:` certo. Sem o `T: 'a`, um `T = &'curto str` quebraria a promessa do `dyn Debug + 'a`. O teste guarda um `i32` e um `&String`, e os dois cabem.
- **(c)** Os E0597 dizem *"argument requires that `c` is borrowed for `'a`"*: o `'a` é escolhido por quem chama, mas `&c` é um empréstimo local. O `help:` `C: 'a` silencia os E0309 e deixa os E0597. A correção é o HRTB da lição 08.
- **(d)** O erro é E0521, *"borrowed data escapes outside of function"*, com as notas *"requirement occurs because of a mutable reference to `Vec<&str>`"* e *"mutable references are invariant over their type parameter"*: atrás do `&mut`, o `&'static str` do vetor não pode encolher para o lifetime de `evento` (lição 07). Duas saídas: um `'a` comum aos dois (a solução, que mantém o teste como está) ou `Vec<String>` com `evento.to_string()`, que obrigaria a mudar o teste.

## 5. Um emprestador com GAT

Projeto: [`ex5-emprestador/`](ex5-emprestador/src/lib.rs).

```rust
impl<T> Emprestador for JanelasMut<'_, T> {
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&mut [T]> {
        let janela = self.dados.get_mut(self.pos..self.pos + self.n)?;
        self.pos += 1;
        Some(janela)
    }
}
```

A janela devolvida empresta `self` inteiro (`&mut self`), e por isso duas janelas sobrepostas nunca estão vivas ao mesmo tempo. Com `Iterator`, isso seria impossível: o `collect` guardaria todas, e haveria dois `&mut` para o mesmo elemento.

Em `Palavras`, o buffer é limpo e preenchido a cada chamada, e um `loop` pula as palavras que ficam vazias depois do filtro (o `--` do teste). Uma alocação para o texto inteiro.

Em `propagar`, a ordem importa: `let primeiro = janela[0];` copia o valor **antes** de `janela.last_mut()` pegar o empréstimo mutável. Com `n = 3` sobre `[1, 0, 0, 0, 0]`, o 1 "pula" de duas em duas posições: `[1, 0, 1, 0, 1]`.

**Por que não `contar_se` genérico?** A versão natural seria:

```rust
pub fn contar_se<E, F>(mut e: E, mut f: F) -> usize
where
    E: Emprestador,
    F: for<'a> FnMut(E::Item<'a>) -> bool,
```

Ela compila, mas chamá-la com um `JanelasMut` sobre um array local dá E0597 com a nota *"due to a current limitation of the type system, this implies a `'static` lifetime"*. O `for<'a>` junto do `where Self: 'a` do GAT faz o compilador exigir `E: 'static`. O mesmo acontece com um método padrão na trait. É uma limitação conhecida dos GATs; os contornos usam traits auxiliares e fogem do escopo do curso. Por isso o exercício usa o `while let` direto em cada função.
