use linha_csv::{LinhaCsv, para_csv};
use minhas_macros::{cor, cronometrar};

#[derive(LinhaCsv)]
struct Venda {
    produto: String,
    #[csv(renomear = "qtd")]
    quantidade: u32,
    total: f64,
}

/// Uma função lenta de propósito, para o `#[cronometrar]` ter o que medir.
#[cronometrar]
fn soma_dos_quadrados(n: u64) -> u64 {
    (1..=n).map(|x| x * x).sum()
}

fn main() {
    let vendas = vec![
        Venda {
            produto: "Café".into(),
            quantidade: 2,
            total: 37.0,
        },
        Venda {
            produto: "Pão de queijo, congelado".into(),
            quantidade: 1,
            total: 22.9,
        },
    ];
    print!("{}", para_csv(&vendas));

    println!("soma = {}", soma_dos_quadrados(1_000_000));

    const LARANJA: [u8; 3] = cor!("#f97316");
    println!("laranja = {LARANJA:?}");
}
