//! Uma trait para transformar structs em linhas de CSV, com um `derive`.

/// O `derive` mora numa crate `proc-macro` separada. Reexportá-lo aqui deixa
/// quem usa escrever só `use linha_csv::LinhaCsv;` e ganhar a trait e o derive.
pub use minhas_macros::LinhaCsv;

/// Um tipo que vira uma linha de CSV.
pub trait LinhaCsv {
    /// Os títulos das colunas.
    fn cabecalho() -> Vec<&'static str>;
    /// Os valores desta linha, na ordem do cabeçalho.
    fn linha(&self) -> Vec<String>;
}

/// Monta um CSV completo: cabeçalho e uma linha por item.
pub fn para_csv<T: LinhaCsv>(itens: &[T]) -> String {
    let mut saida = T::cabecalho().join(",");
    saida.push('\n');
    for item in itens {
        let campos: Vec<String> = item.linha().iter().map(|c| escapar(c)).collect();
        saida.push_str(&campos.join(","));
        saida.push('\n');
    }
    saida
}

/// Põe aspas no campo quando ele tem vírgula, aspas ou quebra de linha.
fn escapar(campo: &str) -> String {
    if campo.contains([',', '"', '\n']) {
        format!("\"{}\"", campo.replace('"', "\"\""))
    } else {
        campo.to_string()
    }
}
