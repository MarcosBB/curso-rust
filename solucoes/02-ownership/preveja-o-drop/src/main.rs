//! Solução do exercício 3: o mesmo programa do enunciado, com a explicação
//! de cada linha da saída nos comentários.

struct Barulhento(&'static str);

impl Drop for Barulhento {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn devolve(b: Barulhento) -> Barulhento {
    println!("devolve recebeu {}", b.0);
    b // devolve a posse: nada é descartado aqui
}

fn main() {
    let a = Barulhento("a1");
    let b = Barulhento("b");
    println!("início: {} {}", a.0, b.0); // início: a1 b
    {
        let c = Barulhento("c");
        let _ = Barulhento("d"); // drop d: `_` não guarda o valor
        println!("bloco: {}", c.0); // bloco: c
    } // drop c
    let a = Barulhento("a2"); // sombreia `a`, mas "a1" continua vivo
    let mut e = Barulhento("e1");
    println!("e = {}", e.0); // e = e1
    e = Barulhento("e2"); // drop e1: a reatribuição descarta o valor antigo
    let b = devolve(b); // devolve recebeu b (o valor volta para o novo `b`)
    drop(e); // drop e2
    let _f = Barulhento("f"); // `_f` é uma variável: vive até o fim
    println!("fim: {} {}", a.0, b.0); // fim: a2 b
} // ordem inversa da declaração: drop f, drop b, (e já foi movida), drop a2,
// (o primeiro `b` foi movido), drop a1
