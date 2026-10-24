use std::io::{self, Write};

use relatorio::relatorio;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    relatorio(&["alfa", "beta", "gama"], &mut stdout)?;
    stdout.flush()
}
