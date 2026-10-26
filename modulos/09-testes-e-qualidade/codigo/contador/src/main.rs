//! `contador [arquivo]`: conta linhas, palavras e bytes do arquivo ou da entrada padrão.

use std::io::{self, Read};
use std::process::ExitCode;

fn main() -> ExitCode {
    let texto = match std::env::args().nth(1) {
        Some(caminho) => match std::fs::read_to_string(&caminho) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("erro: não consegui ler {caminho}: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => {
            let mut t = String::new();
            if let Err(e) = io::stdin().read_to_string(&mut t) {
                eprintln!("erro: não consegui ler a entrada padrão: {e}");
                return ExitCode::FAILURE;
            }
            t
        }
    };
    println!("{}", contador::formatar(contador::contar(&texto)));
    ExitCode::SUCCESS
}
