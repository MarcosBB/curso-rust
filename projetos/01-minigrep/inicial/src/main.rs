// Só no projeto inicial: enquanto `main` e `run` forem `todo!()`, os imports
// e as constantes aparecem como "não usados". Este atributo cala esses
// avisos. Apague esta linha quando terminar a etapa 6.
#![allow(unused)]

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
    // Etapa 6:
    // 1. Junte os argumentos (sem o nome do programa) num `Vec<String>`.
    // 2. Veja se a variável de ambiente `IGNORE_CASE` existe.
    // 3. Chame `Config::from_args`. Em erro: mensagem + uso no stderr, código 2.
    // 4. Chame `run` e transforme o resultado em `ExitCode`.
    //    Um `io::Error` do tipo `BrokenPipe` não é erro: saia com 0.
    todo!()
}

/// Processa todas as entradas e devolve o código de saída (0, 1 ou 2).
///
/// Só a escrita na saída padrão é fatal (devolva o `io::Error` com `?`).
/// Um arquivo que não abre vira aviso no stderr, e o laço continua.
fn run(cfg: &Config) -> io::Result<u8> {
    // Use `io::stdout().lock()` e `writeln!`, e não `println!`.
    todo!()
}
