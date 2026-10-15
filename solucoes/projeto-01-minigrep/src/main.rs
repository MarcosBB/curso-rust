use std::env;
use std::io::{self, Write};
use std::process::ExitCode;

use minigrep::config::Config;
use minigrep::{read_input, render, search};

const USAGE: &str = "uso: minigrep [-i] [-n] [-v] [-c] [--] PADRÃO [ARQUIVO...]";

/// Códigos de saída, na convenção do `grep`.
const FOUND: u8 = 0;
const NOT_FOUND: u8 = 1;
const ERROR: u8 = 2;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let ignore_case_env = env::var("IGNORE_CASE").is_ok();

    let cfg = match Config::from_args(&args, ignore_case_env) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("minigrep: {e}");
            eprintln!("{USAGE}");
            return ExitCode::from(ERROR);
        }
    };

    match run(&cfg) {
        Ok(code) => ExitCode::from(code),
        // `minigrep ... | head` fecha o pipe antes de lermos tudo.
        // Não é erro: quem está do outro lado já tem o que queria.
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::from(FOUND),
        Err(e) => {
            eprintln!("minigrep: erro ao escrever a saída: {e}");
            ExitCode::from(ERROR)
        }
    }
}

/// Processa todas as entradas e devolve o código de saída.
///
/// Só a escrita na saída padrão é fatal (o `?` devolve o `io::Error`).
/// Um arquivo que não abre vira aviso no stderr, e o laço continua.
fn run(cfg: &Config) -> io::Result<u8> {
    // `println!` dá panic se a escrita falhar. Com `writeln!` recebemos
    // o erro como valor e decidimos o que fazer com ele.
    let mut stdout = io::stdout().lock();
    let show_names = cfg.inputs.len() > 1;
    let mut found = false;
    let mut had_error = false;

    for input in &cfg.inputs {
        let contents = match read_input(input) {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("minigrep: {e}");
                had_error = true;
                continue;
            }
        };

        let matches = search::search(&cfg.pattern, &contents, cfg.search);
        found = found || !matches.is_empty();

        let name = input.name();
        let prefix = if show_names {
            Some(name.as_str())
        } else {
            None
        };
        for line in render(&matches, prefix, cfg.output) {
            writeln!(stdout, "{line}")?;
        }
    }

    // Garantia: se a saída tiver buffer, um erro na última escrita só
    // aparece no flush. Sem ele, o erro se perderia em silêncio.
    stdout.flush()?;

    if had_error {
        Ok(ERROR)
    } else if found {
        Ok(FOUND)
    } else {
        Ok(NOT_FOUND)
    }
}
