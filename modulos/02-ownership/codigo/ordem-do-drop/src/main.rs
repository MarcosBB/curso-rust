//! Módulo 02, lição 04: em que momento cada valor é descartado.

/// Tipo de demonstração: imprime uma linha quando é descartado.
///
/// Use como um instrumento de laboratório. Como escrever um `impl Drop`
/// (e quando vale a pena) fica para o Módulo 10.
struct Barulhento(&'static str);

impl Drop for Barulhento {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}

/// Um struct com dois campos que também fazem barulho.
struct Par {
    primeiro: Barulhento,
    segundo: Barulhento,
}

fn cria(nome: &'static str) -> Barulhento {
    Barulhento(nome)
}

fn consome(b: Barulhento) {
    println!("consome recebeu {}", b.0);
} // `b` é descartado aqui, no fim da função que virou dona

fn main() {
    println!("=== escopo, reatribuição, drop() e let _ ===");
    escopo();
    println!();
    println!("=== sombreamento, Vec e campos de struct ===");
    colecoes();
}

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
} // variáveis em ordem inversa (p, v, x2, x1); campos e itens em ordem direta
