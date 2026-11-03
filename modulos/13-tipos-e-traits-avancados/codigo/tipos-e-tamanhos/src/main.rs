//! Lição 02: newtype, aliases de tipo, o tipo `!` e tipos sem tamanho (DSTs).

use std::collections::HashMap;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Add;

// ---------------------------------------------------------------------------
// Newtype: unidades que o compilador não deixa misturar.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Metros(pub f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pes(pub f64);

impl From<Pes> for Metros {
    fn from(p: Pes) -> Metros {
        Metros(p.0 * 0.3048)
    }
}

impl Add for Metros {
    type Output = Metros;

    fn add(self, outro: Metros) -> Metros {
        Metros(self.0 + outro.0)
    }
}

pub fn altitude_segura(a: Metros) -> bool {
    a.0 > 300.0
}

// ---------------------------------------------------------------------------
// Alias: só um nome mais curto, sem proteção nenhuma.
// ---------------------------------------------------------------------------

pub type Indice = HashMap<String, Vec<usize>>;
pub type Resultado<T> = Result<T, String>;

pub fn indexar(texto: &str) -> Indice {
    let mut indice = Indice::new();
    for (n, palavra) in texto.split_whitespace().enumerate() {
        indice.entry(palavra.to_lowercase()).or_default().push(n);
    }
    indice
}

pub fn porta(texto: &str) -> Resultado<u16> {
    texto
        .parse()
        .map_err(|e| format!("porta inválida {texto:?}: {e}"))
}

// ---------------------------------------------------------------------------
// O tipo `!`: expressões que nunca produzem valor.
// ---------------------------------------------------------------------------

pub fn abortar(motivo: &str) -> ! {
    panic!("abortado: {motivo}")
}

/// Soma os números válidos; `continue` tem tipo `!` e cabe no braço do `match`.
pub fn soma_validos(entradas: &[&str]) -> i64 {
    let mut total = 0;
    for e in entradas {
        let n: i64 = match e.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        total += n;
    }
    total
}

// ---------------------------------------------------------------------------
// DSTs e `?Sized`.
// ---------------------------------------------------------------------------

/// Sem o `?Sized`, `mostrar("oi")` não compilaria (`T` seria `str`).
pub fn mostrar<T: Display + ?Sized>(x: &T) -> String {
    format!("[{x}]")
}

/// Um tipo próprio pode ter um DST como último campo.
pub struct Rotulado<T: ?Sized> {
    pub rotulo: &'static str,
    pub valor: T,
}

impl<T: fmt::Debug + ?Sized> Display for Rotulado<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {:?}", self.rotulo, &self.valor)
    }
}

fn main() {
    let leitura = Pes(1000.0);
    let m = Metros::from(leitura);
    println!("{leitura:?} = {m:?}, segura? {}", altitude_segura(m));
    println!("soma: {:?}", Metros(1.5) + Metros(2.0));

    let idx = indexar("o rato roeu a roupa do rei de Roma o rato");
    println!("'rato' em {:?}", idx["rato"]);
    println!("{:?} | {:?}", porta("8080"), porta("80a"));

    println!("válidos: {}", soma_validos(&["10", "x", "5", "", "-3"]));

    println!("{} {} {}", mostrar("oi"), mostrar(&42), mostrar(&'z'));

    // Coerção de tamanho: `Box<Rotulado<[i32; 3]>>` vira `Box<Rotulado<[i32]>>`.
    let notas: Box<Rotulado<[i32]>> = Box::new(Rotulado {
        rotulo: "notas",
        valor: [7, 9, 10],
    });
    println!("{notas} ({} notas)", notas.valor.len());

    println!("--- tamanhos (em bytes, num sistema de 64 bits) ---");
    println!("&u8           {}", size_of::<&u8>());
    println!("&str          {}", size_of::<&str>());
    println!("&[u8]         {}", size_of::<&[u8]>());
    println!("&dyn Display  {}", size_of::<&dyn Display>());
    println!("Box<dyn Fn()> {}", size_of::<Box<dyn Fn()>>());
    println!("Option<&u8>   {}", size_of::<Option<&u8>>());
    println!("Metros        {}", size_of::<Metros>());
    println!("PhantomData   {}", size_of::<PhantomData<String>>());
    println!("()            {}", size_of::<()>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversao_de_unidade() {
        let m = Metros::from(Pes(1000.0));
        assert!((m.0 - 304.8).abs() < 1e-9);
    }

    #[test]
    fn alias_e_o_mesmo_tipo() {
        let r: Result<u16, String> = porta("443");
        assert_eq!(r, Ok(443));
        assert!(porta("x").is_err());
    }

    #[test]
    fn continue_tem_tipo_never() {
        assert_eq!(soma_validos(&["1", "a", "2"]), 3);
    }

    #[test]
    #[should_panic(expected = "abortado: teste")]
    fn abortar_nunca_volta() {
        abortar("teste");
    }

    #[test]
    fn mostrar_aceita_dst() {
        assert_eq!(mostrar("oi"), "[oi]");
        let fatia: &[i32] = &[1, 2];
        assert_eq!(fatia.len(), 2);
    }

    #[test]
    fn fat_pointers_tem_dois_usizes() {
        assert_eq!(size_of::<&str>(), 2 * size_of::<usize>());
        assert_eq!(size_of::<&dyn Display>(), 2 * size_of::<usize>());
        assert_eq!(size_of::<PhantomData<String>>(), 0);
    }
}
