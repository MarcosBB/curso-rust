use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(texto) = std::env::args().nth(1) else {
        eprintln!("uso: idade <texto>");
        return ExitCode::from(2);
    };
    match idade::idade(&texto) {
        Ok(n) => {
            println!("{n} anos");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("idade inválida: {e:?}");
            ExitCode::FAILURE
        }
    }
}
