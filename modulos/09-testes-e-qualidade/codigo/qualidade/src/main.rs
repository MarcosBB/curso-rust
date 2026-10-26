use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(texto) = std::env::args().nth(1) else {
        eprintln!("uso: qualidade \"7, 8, 10\"");
        return ExitCode::from(2);
    };
    match qualidade::ler_notas(&texto) {
        Ok(notas) => {
            match qualidade::media(&notas) {
                Some(m) => println!("média: {m}"),
                None => println!("sem notas"),
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("nota inválida: {e}");
            ExitCode::FAILURE
        }
    }
}
