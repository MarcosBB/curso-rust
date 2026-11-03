use macros_declarativas::{basico, gerar_codigo, higiene, repeticoes};

fn main() {
    println!("--- básico (lição 02)");
    basico::demonstrar();
    println!("--- repetições (lição 03)");
    repeticoes::demonstrar();
    gerar_codigo::demonstrar();
    println!("--- higiene (lição 04)");
    higiene::demonstrar();
}
