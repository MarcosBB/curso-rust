//! Solução do exercício 2: quem pode ser `Copy`?

// Todos os campos são `Copy` (f64): pode ser Copy.
#[derive(Clone, Copy)]
struct Ponto {
    x: f64,
    y: f64,
}

// Três u8: pode ser Copy.
#[derive(Clone, Copy)]
struct Cor(u8, u8, u8);

// `String` não é Copy (E0204): fica só com Clone.
#[derive(Clone)]
struct Usuario {
    id: u32,
    nome: String,
}

// Array de i32 é Copy, porque i32 é Copy.
#[derive(Clone, Copy)]
struct Grade([i32; 4]);

// Tem destrutor (Drop): não pode ser Copy (E0184). Fica só com Clone.
#[derive(Clone)]
struct Arquivo(&'static str);

impl Drop for Arquivo {
    fn drop(&mut self) {
        println!("fechando {}", self.0);
    }
}

fn main() {
    let p1 = Ponto { x: 1.0, y: 2.0 };
    let p2 = p1;
    println!("ponto: ({}, {}) e ({}, {})", p1.x, p1.y, p2.x, p2.y);

    let c1 = Cor(255, 128, 0);
    let c2 = c1;
    println!("cor: {} {} {} / {}", c1.0, c1.1, c1.2, c2.0);

    let u1 = Usuario {
        id: 1,
        nome: String::from("Ana"),
    };
    let u2 = u1.clone(); // cópia explícita: aloca outra String
    println!("usuário: {} {} / {} {}", u1.id, u1.nome, u2.id, u2.nome);

    let g1 = Grade([1, 2, 3, 4]);
    let g2 = g1;
    println!("grade: {:?} / {:?}", g1.0, g2.0);

    let a1 = Arquivo("dados.txt");
    let a2 = a1.clone(); // agora existem dois valores, e cada um será descartado
    println!("arquivo: {} / {}", a1.0, a2.0);
}
