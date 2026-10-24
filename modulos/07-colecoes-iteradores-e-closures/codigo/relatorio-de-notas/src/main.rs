use relatorio_de_notas::{aprovados_e_reprovados, ler, media, melhor, por_conceito};

const TURMA: &str = "\
Ana;9.5
Bia;6
Caio;7.25
Davi;4.5
";

fn main() {
    // 1. O modelo "pull": cada item atravessa a cadeia inteira (lição 07).
    println!("== ordem de execução ==");
    let r: Vec<i32> = (1..=3)
        .inspect(|x| println!("gerou {x}"))
        .map(|x| {
            println!("  map {x} -> {}", x * x);
            x * x
        })
        .filter(|x| {
            println!("    filter {x}");
            x % 2 == 1
        })
        .collect();
    println!("r = {r:?}");

    // 2. Um relatório só com iteradores.
    println!("\n== relatório ==");
    let alunos = match ler(TURMA) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("erro: {e}");
            return;
        }
    };
    if let Some(m) = media(&alunos) {
        println!("média: {m:.2}");
    }
    if let Some(a) = melhor(&alunos) {
        println!("melhor: {} ({})", a.nome, a.nota);
    }
    let (sim, nao) = aprovados_e_reprovados(&alunos, 6.0);
    println!("aprovados: {}", sim.join(", "));
    println!("reprovados: {}", nao.join(", "));
    for (conceito, nomes) in por_conceito(&alunos) {
        println!("{conceito}: {}", nomes.join(", "));
    }
}
