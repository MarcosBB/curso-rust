use std::env;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::ExitCode;

use minigrep::config::Config;
use minigrep::{ERROR, FOUND, run};

const USAGE: &str = "uso: minigrep [-i] [-n] [-v] [-c] [-o ARQUIVO] [--] PADRÃO [ARQUIVO...]";

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

    // O destino só é conhecido em tempo de execução: stdout ou um arquivo.
    // São dois tipos diferentes, então o `match` devolve um trait object.
    // `Box<dyn Write>` também implementa `Write`, e por isso serve para o
    // `out: &mut impl Write` de `run`.
    let mut out: Box<dyn Write> = match &cfg.output_file {
        None => Box::new(io::stdout().lock()),
        Some(path) => match File::create(path) {
            Ok(file) => Box::new(BufWriter::new(file)),
            Err(e) => {
                eprintln!("minigrep: {}: {e}", path.display());
                return ExitCode::from(ERROR);
            }
        },
    };

    match run(&cfg, &mut out) {
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
