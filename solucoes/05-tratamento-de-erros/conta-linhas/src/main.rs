use std::fs;
use std::io;
use std::process::ExitCode;

/// Os dois jeitos de o programa falhar, cada um com o seu código de saída.
enum Failure {
    Usage,
    Read { path: String, source: io::Error },
}

fn run() -> Result<usize, Failure> {
    let path = std::env::args().nth(1).ok_or(Failure::Usage)?;
    let text = fs::read_to_string(&path).map_err(|source| Failure::Read { path, source })?;
    Ok(text.lines().count())
}

fn main() -> ExitCode {
    match run() {
        Ok(n) => {
            println!("{n}");
            ExitCode::SUCCESS
        }
        Err(Failure::Usage) => {
            eprintln!("uso: conta-linhas <arquivo>");
            ExitCode::from(2)
        }
        Err(Failure::Read { path, source }) => {
            eprintln!("erro: não consegui ler {path}: {source}");
            ExitCode::FAILURE
        }
    }
}
