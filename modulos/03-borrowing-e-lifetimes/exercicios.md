# Exercícios — Módulo 03

Soluções em [`solucoes/03-borrowing-e-lifetimes`](../../solucoes/03-borrowing-e-lifetimes/).

Nos exercícios com código, crie um projeto de biblioteca (`cargo new --lib nome`), cole o código em `src/lib.rs` e rode `cargo test`. O código inicial **não compila** (ou não passa nos testes) de propósito.

## 1. O `push` fantasma

Faça compilar e passar nos testes, **sem usar `clone()`** e sem mudar os testes. As duas funções têm o mesmo erro, mas pedem correções diferentes: em `registra`, guarde só o que você precisa; em `resumo`, mude a ordem.

```rust
/// Adiciona `novo` ao fim de `nomes` e devolve o tamanho, em bytes,
/// do maior nome que existia ANTES da inserção (0 se não havia nenhum).
pub fn registra(nomes: &mut Vec<String>, novo: &str) -> usize {
    let maior = nomes.iter().max_by_key(|n| n.len()).unwrap();
    nomes.push(novo.to_string());
    maior.len()
}

/// Acrescenta "fim" a `itens` e devolve um resumo com o primeiro item
/// e o total DEPOIS da inserção.
pub fn resumo(itens: &mut Vec<String>) -> String {
    let primeiro = &itens[0];
    itens.push(String::from("fim"));
    format!("primeiro: {primeiro}, total: {}", itens.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registra_devolve_o_maior_anterior() {
        let mut v = vec![String::from("Ana"), String::from("Bartolomeu")];
        assert_eq!(registra(&mut v, "Maximiliano Augusto"), 10);
        assert_eq!(v, ["Ana", "Bartolomeu", "Maximiliano Augusto"]);
    }

    #[test]
    fn registra_com_vetor_vazio() {
        let mut v = Vec::new();
        assert_eq!(registra(&mut v, "Zé"), 0);
        assert_eq!(v, ["Zé"]);
    }

    #[test]
    fn resumo_conta_depois_do_push() {
        let mut v = vec![String::from("a"), String::from("b")];
        assert_eq!(resumo(&mut v), "primeiro: a, total: 3");
        assert_eq!(v, ["a", "b", "fim"]);
    }
}
```

<details>
<summary>Dica</summary>

Leia os três rótulos do E0502: onde nasce a `&`, onde está o `&mut` (o `push`) e onde a `&` é usada depois. Em `registra`, você precisa de um **número**, não de uma referência para o nome. O teste do vetor vazio também vai exigir trocar o `unwrap()`.

</details>

## 2. Assinaturas

Seis funções. Para cada uma, decida: **(a)** já compila (elisão resolve), **(b)** precisa de `<'a>` (e em quais parâmetros), ou **(c)** deveria devolver outra coisa (`String` ou `&'static str`). Corrija só as assinaturas (e o corpo quando o tipo de retorno mudar), sem mexer nos testes. Depois, justifique cada escolha numa frase.

```rust
/// Devolve a string mais longa (a primeira, em caso de empate).
pub fn longest(x: &str, y: &str) -> &str {
    if x.len() >= y.len() { x } else { y }
}

/// Devolve a primeira palavra (separada por espaço).
pub fn first_word(s: &str) -> &str {
    s.split(' ').next().unwrap_or("")
}

/// Devolve sempre o primeiro argumento. O segundo só é registrado em log.
pub fn keep_first(x: &str, y: &str) -> &str {
    println!("descartando {y}");
    x
}

/// Versão do programa.
pub fn version() -> &str {
    "1.0.3"
}

/// Mensagem de saudação conforme a hora.
pub fn greeting(hour: u32) -> &str {
    let period = if hour < 12 { "bom dia" } else { "boa tarde" };
    let msg = format!("{period}! são {hour}h");
    &msg
}

/// Divide em duas partes no primeiro `sep`. Sem `sep`, a segunda parte é vazia.
pub fn split_pair(s: &str, sep: char) -> (&str, &str) {
    match s.find(sep) {
        Some(i) => (&s[..i], &s[i + sep.len_utf8()..]),
        None => (s, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest() {
        assert_eq!(longest("abc", "de"), "abc");
        assert_eq!(longest("ab", "cde"), "cde");
    }

    #[test]
    fn test_first_word() {
        assert_eq!(first_word("olá mundo"), "olá");
    }

    #[test]
    fn keep_first_nao_depende_do_segundo() {
        let x = String::from("fica");
        let r;
        {
            let y = String::from("morre");
            r = keep_first(&x, &y);
        }
        assert_eq!(r, "fica");
    }

    #[test]
    fn test_version() {
        assert_eq!(version(), "1.0.3");
    }

    #[test]
    fn test_greeting() {
        assert_eq!(greeting(9), "bom dia! são 9h");
        assert_eq!(greeting(15), "boa tarde! são 15h");
    }

    #[test]
    fn test_split_pair() {
        assert_eq!(split_pair("chave=valor", '='), ("chave", "valor"));
        assert_eq!(split_pair("sozinho", '='), ("sozinho", ""));
    }
}
```

<details>
<summary>Dica</summary>

Aplique as três regras de elisão da lição 05 a cada assinatura. Em `keep_first`, o teste `keep_first_nao_depende_do_segundo` só compila se o contrato for o mais preciso possível. Em `version` e `greeting`, o E0106 sugere duas saídas, e cada função pede uma diferente.

</details>

## 3. As palavras mais longas, sem copiar

Implemente `palavras_mais_longas` para passar nos testes. Regras:

- as palavras são separadas por espaço em branco, e a pontuação nas pontas é ignorada (`"(olá),"` vira `"olá"`);
- o tamanho é medido em **caracteres**, e não em bytes;
- o resultado vem da mais longa para a mais curta, e em empate vale a ordem do texto;
- **nenhuma `String` nova**: cada `&str` devolvido aponta para dentro de `texto` (o último teste confere isso pelos endereços).

```rust
/// Devolve as `n` palavras mais longas de `texto` (em número de `char`s),
/// da mais longa para a mais curta. Em caso de empate, vem primeiro a que
/// aparece antes no texto. Pontuação nas pontas da palavra é ignorada.
pub fn palavras_mais_longas(texto: &str, n: usize) -> Vec<&str> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `p` está inteiramente dentro da memória de `texto`?
    fn aponta_para_dentro(texto: &str, p: &str) -> bool {
        let ini = texto.as_ptr() as usize;
        let fim = ini + texto.len();
        let q = p.as_ptr() as usize;
        q >= ini && q + p.len() <= fim
    }

    #[test]
    fn as_tres_maiores() {
        let texto = "O borrow checker verifica empréstimos em tempo de compilação.";
        assert_eq!(
            palavras_mais_longas(texto, 3),
            ["empréstimos", "compilação", "verifica"]
        );
    }

    #[test]
    fn empate_mantem_a_ordem_do_texto() {
        assert_eq!(palavras_mais_longas("ab cd ef g", 2), ["ab", "cd"]);
    }

    #[test]
    fn ignora_pontuacao_nas_pontas() {
        assert_eq!(palavras_mais_longas("(olá), mundo!", 2), ["mundo", "olá"]);
    }

    #[test]
    fn n_maior_que_o_total() {
        assert_eq!(palavras_mais_longas("um dois", 10), ["dois", "um"]);
        assert!(palavras_mais_longas("", 3).is_empty());
    }

    #[test]
    fn nao_aloca_strings_novas() {
        let texto = String::from("slices apontam para o texto original");
        for p in palavras_mais_longas(&texto, 4) {
            assert!(
                aponta_para_dentro(&texto, p),
                "{p:?} não aponta para o texto"
            );
        }
    }
}
```

Pergunta extra: por que a assinatura não precisa de `<'a>`?

<details>
<summary>Dica</summary>

`split_whitespace()` e `trim_matches(...)` devolvem `&str` que apontam para dentro do texto original. Junte tudo num `Vec<&str>`, ordene com `sort_by_key` (que é **estável**: preserva a ordem dos empates) usando `std::cmp::Reverse` para inverter a ordem, e corte com `truncate(n)`.

</details>

## 4. Um tokenizador que não copia

Implemente `next_token` para passar nos testes. Um token é uma sequência de letras, dígitos ou `_` (use `char::is_alphanumeric`), **ou** um único caractere de pontuação. Espaços são ignorados.

```rust
/// Quebra um texto em tokens: sequências de letras, dígitos e `_`, ou um
/// único caractere de pontuação. Espaços são ignorados.
///
/// O `Tokenizer` só guarda uma visão do texto que ainda falta ler. Ele não
/// copia nada.
pub struct Tokenizer<'a> {
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(src: &'a str) -> Self {
        Tokenizer { rest: src }
    }

    pub fn next_token(&mut self) -> Option<&str> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_tokens(src: &str) -> Vec<&str> {
        let mut t = Tokenizer::new(src);
        let mut out = Vec::new();
        while let Some(tok) = t.next_token() {
            out.push(tok);
        }
        out
    }

    #[test]
    fn separa_palavras_e_pontuacao() {
        assert_eq!(all_tokens("let x=42;"), ["let", "x", "=", "42", ";"]);
    }

    #[test]
    fn ignora_espacos_e_aceita_unicode() {
        assert_eq!(all_tokens("  ação  (já)  "), ["ação", "(", "já", ")"]);
        assert!(all_tokens("   ").is_empty());
    }

    #[test]
    fn token_sobrevive_ao_tokenizer() {
        fn first_token(src: &str) -> Option<&str> {
            let mut t = Tokenizer::new(src);
            t.next_token()
        }
        let src = String::from("primeiro segundo");
        assert_eq!(first_token(&src), Some("primeiro"));
    }
}
```

Atenção: com a assinatura como está, `cargo test` não compila nem antes de você escrever o corpo. Leia os dois erros (E0499 e E0515) e descubra o que está errado no **contrato** de `next_token`.

<details>
<summary>Dica</summary>

Pela regra 3 da elisão, `-> Option<&str>` prende o token ao empréstimo `&mut self`. Enquanto o token viver, o `Tokenizer` fica emprestado como `&mut`, então não dá para chamar `next_token` de novo nem devolver o token depois que o `Tokenizer` morre. O token, na verdade, aponta para o texto original (`'a`). Releia a seção sobre `part` x `part_long` na lição 06.

No corpo: `trim_start()`, `chars().next()?`, `find(...)` para achar o fim da palavra, `char::len_utf8()` para o tamanho de um caractere de pontuação, e `split_at` para separar o token do resto.

</details>

## 5. Inventário sem `clone`

Faça compilar e passar nos testes **sem `clone()`**, `to_string()`, `to_owned()`, `Rc` ou `RefCell` dentro do `impl`. Cada um dos três métodos tem um erro diferente (E0506, E0499 e E0502). Para cada um, diga também se o programa original era **inseguro** ou se era uma **limitação** do checker.

```rust
#[derive(Debug, PartialEq)]
pub struct Item {
    pub nome: String,
    pub preco: u32,
}

pub struct Inventario {
    pub itens: Vec<Item>,
    pub ouro: u32,
}

impl Inventario {
    pub fn mais_caro(&self) -> Option<&Item> {
        self.itens.iter().max_by_key(|i| i.preco)
    }

    pub fn media(&self) -> u32 {
        if self.itens.is_empty() {
            return 0;
        }
        let total: u32 = self.itens.iter().map(|i| i.preco).sum();
        total / self.itens.len() as u32
    }

    /// Compra uma réplica do item mais caro: custa 1/10 do preço, tem esse
    /// mesmo preço e o nome "<nome> (réplica)". Devolve `false` se não há
    /// itens ou se falta ouro.
    pub fn compra_replica(&mut self) -> bool {
        if let Some(caro) = self.mais_caro() {
            let preco = caro.preco / 10;
            if self.ouro < preco {
                return false;
            }
            self.ouro -= preco;
            self.itens.push(Item {
                nome: format!("{} (réplica)", caro.nome),
                preco,
            });
            return true;
        }
        false
    }

    /// Soma o preço do item `de` no item `para` e zera o `de`. Devolve
    /// `false` (sem mudar nada) se os índices forem iguais ou inválidos.
    pub fn funde(&mut self, de: usize, para: usize) -> bool {
        let origem = &mut self.itens[de];
        let destino = &mut self.itens[para];
        destino.preco += origem.preco;
        origem.preco = 0;
        true
    }

    /// Todo item acima da média passa a custar a média (calculada antes
    /// de qualquer mudança).
    pub fn limita_a_media(&mut self) {
        for item in &mut self.itens {
            if item.preco > self.media() {
                item.preco = self.media();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(nome: &str, preco: u32) -> Item {
        Item {
            nome: nome.to_string(),
            preco,
        }
    }

    fn inventario() -> Inventario {
        Inventario {
            itens: vec![item("espada", 300), item("escudo", 120), item("poção", 30)],
            ouro: 50,
        }
    }

    #[test]
    fn compra_replica_do_mais_caro() {
        let mut inv = inventario();
        assert!(inv.compra_replica());
        assert_eq!(inv.ouro, 20);
        assert_eq!(inv.itens[3], item("espada (réplica)", 30));
    }

    #[test]
    fn compra_replica_sem_ouro() {
        let mut inv = inventario();
        inv.ouro = 10;
        assert!(!inv.compra_replica());
        assert_eq!(inv.itens.len(), 3);
        assert_eq!(inv.ouro, 10);
    }

    #[test]
    fn funde_dois_itens() {
        let mut inv = inventario();
        assert!(inv.funde(2, 0));
        assert_eq!(inv.itens[0].preco, 330);
        assert_eq!(inv.itens[2].preco, 0);
    }

    #[test]
    fn funde_recusa_indices_iguais_ou_invalidos() {
        let mut inv = inventario();
        assert!(!inv.funde(1, 1));
        assert!(!inv.funde(0, 9));
        assert_eq!(inv.itens[1].preco, 120);
    }

    #[test]
    fn limita_pela_media_original() {
        let mut inv = inventario();
        inv.limita_a_media(); // média = 150
        let precos: Vec<u32> = inv.itens.iter().map(|i| i.preco).collect();
        assert_eq!(precos, [150, 120, 30]);
    }
}
```

<details>
<summary>Dica</summary>

- `compra_replica`: `caro` empresta `self` **inteiro** (porque veio de um método `&self`). Termine de usar `caro` antes de mexer em `self`: monte o `Item` novo primeiro.
- `funde`: duas `&mut` para o mesmo `Vec`. Existe um método da lição 03 que devolve as duas e ainda recusa índices iguais.
- `limita_a_media`: dentro do laço, `self.itens` está emprestado como `&mut`. Quanto vale a média? Ela muda durante o laço?

</details>

## 6. Quiz: inseguro ou limitação?

Nenhum dos trechos abaixo compila. Para cada um: **(a)** rode e anote o código do erro; **(b)** classifique como **programa inseguro** (se rodasse, faria algo errado) ou **limitação do checker** (seria seguro, mas o checker não consegue provar); **(c)** proponha uma correção idiomática.

**A**

```rust
fn make(prefix: &str) -> &str {
    let s = format!("{prefix}!");
    &s
}
```

**B**

```rust
fn main() {
    let mut v = vec![1, 2, 3];
    let first = &v[0];
    v.push(4);
    println!("{first}");
}
```

**C**

```rust
fn get_first(name: &(String, String)) -> &String {
    &name.0
}

fn main() {
    let mut name = (String::from("Ferris"), String::from("Rustáceo"));
    let first = get_first(&name);
    name.1.push_str(", Esq.");
    println!("{first} {}", name.1);
}
```

**D**

```rust
fn main() {
    let mut a = [1, 2, 3, 4];
    let x = &mut a[1];
    let y = &a[2];
    *x += *y;
    println!("{a:?}");
}
```

**E**

```rust
use std::collections::HashMap;

fn get_or_default(map: &mut HashMap<u32, String>, k: u32) -> &String {
    if let Some(v) = map.get(&k) {
        return v;
    }
    map.insert(k, String::new());
    &map[&k]
}
```

**F**

```rust
fn main() {
    let v = vec![String::from("a"), String::from("b")];
    let s = v[0];
    println!("{s}");
}
```

<details>
<summary>Dica</summary>

Três são inseguros e três são limitações. Para cada um, imagine o programa rodando sem o borrow checker: alguma memória seria lida depois de liberada, ou liberada duas vezes?

</details>

---

[Índice do módulo](README.md)
