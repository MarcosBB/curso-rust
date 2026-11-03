//! Lição 06: constantes associadas, bounds em tipos associados, RPITIT e GATs.

use std::fmt::Display;

// ---------------------------------------------------------------------------
// Constantes associadas.
// ---------------------------------------------------------------------------

trait Unidade {
    const SIMBOLO: &'static str;
    const POR_METRO: f64;

    fn formatar(metros: f64) -> String {
        format!("{:.2} {}", metros * Self::POR_METRO, Self::SIMBOLO)
    }
}

struct Quilometro;
struct Milha;

impl Unidade for Quilometro {
    const SIMBOLO: &'static str = "km";
    const POR_METRO: f64 = 0.001;
}

impl Unidade for Milha {
    const SIMBOLO: &'static str = "mi";
    const POR_METRO: f64 = 1.0 / 1609.344;
}

// ---------------------------------------------------------------------------
// Bound direto no tipo associado: `Item: Display` (1.79).
// ---------------------------------------------------------------------------

fn juntar(itens: impl IntoIterator<Item: Display>, sep: &str) -> String {
    itens
        .into_iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(sep)
}

// ---------------------------------------------------------------------------
// `impl Trait` no retorno de um método de trait (RPITIT, 1.75).
// ---------------------------------------------------------------------------

trait Colecao {
    fn itens(&self) -> impl Iterator<Item = &str>;

    fn contar_com(&self, prefixo: &str) -> usize {
        self.itens().filter(|i| i.starts_with(prefixo)).count()
    }
}

struct Lista(Vec<String>);

impl Colecao for Lista {
    fn itens(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(String::as_str)
    }
}

struct Csv(String);

impl Colecao for Csv {
    fn itens(&self) -> impl Iterator<Item = &str> {
        self.0.split(',').map(str::trim)
    }
}

// ---------------------------------------------------------------------------
// GAT: um "iterador que empresta de si mesmo".
// ---------------------------------------------------------------------------

trait Emprestador {
    type Item<'a>
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<Self::Item<'_>>;
}

/// Lê linhas e devolve cada uma, normalizada, num buffer reaproveitado.
struct Linhas<'t> {
    resto: &'t str,
    buf: String,
}

impl<'t> Linhas<'t> {
    fn new(texto: &'t str) -> Self {
        Linhas {
            resto: texto,
            buf: String::new(),
        }
    }
}

impl Emprestador for Linhas<'_> {
    type Item<'a>
        = &'a str
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&str> {
        let (linha, resto) = self.resto.split_once('\n')?;
        self.resto = resto;
        self.buf.clear();
        self.buf
            .extend(linha.trim().chars().map(|c| c.to_ascii_uppercase()));
        Some(&self.buf)
    }
}

/// Janelas sobrepostas e mutáveis de 2 elementos: `[a, b]`, `[b, c]`, ...
struct JanelasMut<'d, T> {
    dados: &'d mut [T],
    pos: usize,
}

impl<T> Emprestador for JanelasMut<'_, T> {
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&mut [T]> {
        let janela = self.dados.get_mut(self.pos..self.pos + 2)?;
        self.pos += 1;
        Some(janela)
    }
}

/// Funções genéricas funcionam, mas não há `for`, `map` nem `collect`.
fn contar<E: Emprestador>(mut e: E) -> usize {
    let mut n = 0;
    while e.proximo().is_some() {
        n += 1;
    }
    n
}

/// Soma acumulada no lugar: `[1, 2, 3, 4]` vira `[1, 3, 6, 10]`.
fn acumular(v: &mut [i64]) {
    let mut j = JanelasMut { dados: v, pos: 0 };
    while let Some(par) = j.proximo() {
        par[1] += par[0];
    }
}

fn main() {
    println!("{}", Quilometro::formatar(42_195.0));
    println!("{}", Milha::formatar(42_195.0));

    println!("{}", juntar([1, 2, 3], " + "));
    println!("{}", juntar(vec!['a', 'b'], ""));

    let l = Lista(vec!["rust".into(), "ruby".into(), "go".into()]);
    let c = Csv(String::from("rato, roupa, rei, Roma"));
    println!("começam com 'ru': {}", l.contar_com("ru"));
    println!("itens do csv: {:?}", c.itens().collect::<Vec<_>>());

    let texto = "  um\n dois  \ntres\n";
    let mut linhas = Linhas::new(texto);
    while let Some(l) = linhas.proximo() {
        println!("{l:?}");
    }
    println!("{} linhas", contar(Linhas::new(texto)));

    let mut v = [1, 2, 3, 4];
    acumular(&mut v);
    println!("{v:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constantes() {
        assert_eq!(Quilometro::SIMBOLO, "km");
        assert_eq!(Quilometro::formatar(1500.0), "1.50 km");
    }

    #[test]
    fn juntar_aceita_qualquer_display() {
        assert_eq!(juntar([1.5, 2.0], ";"), "1.5;2");
        assert_eq!(juntar(Vec::<String>::new(), ","), "");
    }

    #[test]
    fn rpitit() {
        let c = Csv(String::from("a, b ,c"));
        assert_eq!(c.itens().collect::<Vec<_>>(), ["a", "b", "c"]);
        assert_eq!(c.contar_com("b"), 1);
    }

    #[test]
    fn linhas_reaproveitam_o_buffer() {
        let mut l = Linhas::new("a\n b \n");
        assert_eq!(l.proximo(), Some("A"));
        assert_eq!(l.proximo(), Some("B"));
        assert_eq!(l.proximo(), None);
        assert_eq!(contar(Linhas::new("x\ny\nz\n")), 3);
    }

    #[test]
    fn janelas_sobrepostas() {
        let mut v = [1, 2, 3, 4];
        acumular(&mut v);
        assert_eq!(v, [1, 3, 6, 10]);
        let mut vazio: [i64; 1] = [5];
        acumular(&mut vazio);
        assert_eq!(vazio, [5]);
    }
}
