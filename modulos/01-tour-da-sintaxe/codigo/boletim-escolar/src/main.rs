use boletim_escolar::notas::{conceito, media};
use boletim_escolar::{agrupar, nomes_ordenados};

fn main() {
    let lancamentos = vec![
        ("Ana", 9.5),
        ("Bruno", 6.0),
        ("Ana", 8.5),
        ("Carla", 4.0),
        ("Bruno", 7.5),
        ("Carla", 5.5),
    ];

    let turma = agrupar(lancamentos);

    for nome in nomes_ordenados(&turma) {
        let notas = &turma[&nome];
        let m = media(notas).unwrap_or(0.0);
        println!("{nome:<6} {notas:?} média {m:.2} conceito {}", conceito(m));
    }

    // Uma closure como critério de filtro.
    let aprovado = |m: f64| m >= 5.0;
    let mut aprovados = 0;
    for notas in turma.values() {
        if aprovado(media(notas).unwrap_or(0.0)) {
            aprovados += 1;
        }
    }
    println!("aprovados: {aprovados} de {}", turma.len());
}
