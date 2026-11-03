use std::env;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use contapal::config::{Config, Modo, USO};
use contapal::processar;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = match Config::de_args(&args) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("contapal: {e}");
            eprintln!("{USO}");
            return ExitCode::from(2);
        }
    };

    let inicio = Instant::now();
    let relatorio = match cfg.modo {
        Modo::Sequencial => processar::sequencial(&cfg.caminhos),
        Modo::Canais => processar::com_canais(&cfg.caminhos, cfg.threads),
        Modo::Rayon => processar::com_rayon(&cfg.caminhos, cfg.threads),
    };
    let duracao = inicio.elapsed();

    // Um arquivo ruim é aviso, não erro fatal: o relatório sai do mesmo jeito.
    for e in &relatorio.erros {
        eprintln!("contapal: {e}");
    }

    let mut stdout = io::stdout().lock();
    let escrita = relatorio
        .escrever(&mut stdout, cfg.top, cfg.por_arquivo)
        .and_then(|()| stdout.flush());
    match escrita {
        Ok(()) => {}
        // `contapal ... | head` fecha o pipe cedo. Não é erro (Projeto 1).
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("contapal: erro ao escrever a saída: {e}");
            return ExitCode::from(2);
        }
    }

    if cfg.tempo {
        eprintln!("tempo: {duracao:.1?} (modo {:?})", cfg.modo);
    }
    if relatorio.erros.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
