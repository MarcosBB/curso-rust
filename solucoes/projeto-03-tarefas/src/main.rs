//! `tarefas`: o executável. Lê o ambiente, chama a biblioteca e traduz o
//! resultado em mensagem e código de saída.

use std::env;
use std::error::Error as _;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use tarefas::Erro;
use tarefas::app::executar;
use tarefas::armazenamento::ArquivoJson;
use tarefas::comando::{Comando, USO};

/// Onde fica o arquivo: `TAREFAS_ARQUIVO` ou `~/.tarefas.json`.
/// (O `--arquivo` é tratado antes, em `rodar`.)
fn caminho_padrao() -> Result<PathBuf, Erro> {
    if let Some(caminho) = env::var_os("TAREFAS_ARQUIVO") {
        return Ok(PathBuf::from(caminho));
    }
    env::home_dir()
        .map(|pasta| pasta.join(".tarefas.json"))
        .ok_or_else(|| Erro::Uso("não achei a pasta pessoal; use --arquivo".into()))
}

/// Segundos desde 1970. Um relógio antes de 1970 vira 0.
fn agora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn rodar(args: &[&str]) -> Result<(), Erro> {
    let (caminho, resto) = match args {
        ["--arquivo", caminho, resto @ ..] => (Some(PathBuf::from(caminho)), resto),
        _ => (None, args),
    };
    // Primeiro o comando: um erro de uso aparece antes de qualquer problema
    // com o arquivo.
    let cmd = Comando::de_args(resto)?;
    let caminho = match caminho {
        Some(caminho) => caminho,
        None => caminho_padrao()?,
    };
    let mut arquivo = ArquivoJson::new(caminho);
    executar(cmd, &mut arquivo, &mut io::stdout().lock(), agora())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    match rodar(&args) {
        Ok(()) => ExitCode::SUCCESS,
        // Pipe fechado (`tarefas list | head -1`) não é erro.
        Err(Erro::Saida(e)) if e.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(e) => {
            // A mensagem e, depois, cada causa: "contexto: causa: causa".
            let mut mensagem = format!("tarefas: {e}");
            let mut causa = e.source();
            while let Some(c) = causa {
                mensagem.push_str(&format!(": {c}"));
                causa = c.source();
            }
            eprintln!("{mensagem}");
            match e {
                Erro::Uso(_) => {
                    eprintln!("{USO}");
                    ExitCode::from(2)
                }
                _ => ExitCode::from(1),
            }
        }
    }
}
