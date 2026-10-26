//! `tarefas`: o executável. Lê o ambiente, chama a biblioteca e traduz o
//! resultado em mensagem e código de saída.

// Só no projeto inicial: apague esta linha na etapa 7.
#![allow(unused)]

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
    // Etapa 7: `env::var_os("TAREFAS_ARQUIVO")`, senão `env::home_dir()`.
    todo!()
}

/// Segundos desde 1970. Um relógio antes de 1970 vira 0.
fn agora() -> u64 {
    // Etapa 7: `SystemTime::now().duration_since(UNIX_EPOCH)`.
    todo!()
}

fn rodar(args: &[&str]) -> Result<(), Erro> {
    // Etapa 7: separe o `--arquivo CAMINHO` do começo, interprete o
    // comando, escolha o caminho e chame `executar`.
    todo!()
}

fn main() -> ExitCode {
    // Etapa 7: chame `rodar` e traduza o resultado: sucesso 0; erro de uso 2
    // (com a linha de uso); outros erros 1. A mensagem é
    // "tarefas: <erro>: <causa>: <causa>" (percorra `source()`).
    todo!()
}
