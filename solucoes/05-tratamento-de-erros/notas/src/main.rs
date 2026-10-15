//! Aplicação: `cargo run -- exemplos/turma.txt`

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        bail!("uso: notas <arquivo>");
    };
    let notas = notas::load(&path).context("falha ao calcular a média")?;
    let media = notas::average(&notas).context("falha ao calcular a média")?;
    println!("{} alunos, média {media:.1}", notas.len());
    Ok(())
}
