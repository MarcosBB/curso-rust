//! Codifica e decodifica **uma linha** de CSV, com campos entre aspas.
//! Exemplo das lições 07 a 09 do Módulo 09 (CI, proptest e criterion).
//! O Projeto 2 faz a versão completa, com várias linhas e registros.

/// Por que uma linha não pôde ser lida.
#[derive(Debug, PartialEq, Eq)]
pub enum ErroLinha {
    /// Um campo abriu aspas e a linha acabou antes de fechá-las.
    AspasAbertas,
}

/// Junta os campos numa linha, separados por vírgula.
///
/// Um campo que contém vírgula, aspas ou quebra de linha vai entre aspas,
/// e as aspas de dentro são dobradas.
///
/// # Examples
///
/// ```
/// let linha = linha_csv::codificar(&["nome", "São Paulo, SP"]);
/// assert_eq!(linha, r#"nome,"São Paulo, SP""#);
/// ```
pub fn codificar<S: AsRef<str>>(campos: &[S]) -> String {
    let mut linha = String::new();
    for (i, campo) in campos.iter().enumerate() {
        if i > 0 {
            linha.push(',');
        }
        let campo = campo.as_ref();
        if precisa_de_aspas(campo) {
            linha.push('"');
            linha.push_str(&campo.replace('"', "\"\""));
            linha.push('"');
        } else {
            linha.push_str(campo);
        }
    }
    linha
}

fn precisa_de_aspas(campo: &str) -> bool {
    campo.contains([',', '"', '\n', '\r'])
}

/// Separa uma linha em campos. Uma linha vazia tem um campo vazio.
///
/// # Errors
///
/// [`ErroLinha::AspasAbertas`] se um campo abrir aspas e não fechar.
///
/// # Examples
///
/// ```
/// let campos = linha_csv::decodificar(r#"a,"b,c",d"#)?;
/// assert_eq!(campos, ["a", "b,c", "d"]);
/// # Ok::<(), linha_csv::ErroLinha>(())
/// ```
pub fn decodificar(linha: &str) -> Result<Vec<String>, ErroLinha> {
    let mut campos = Vec::new();
    let mut atual = String::new();
    let mut entre_aspas = false;
    let mut chars = linha.chars().peekable();

    while let Some(c) = chars.next() {
        if entre_aspas {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    chars.next(); // aspas dobradas: uma aspa literal
                    atual.push('"');
                }
                '"' => entre_aspas = false,
                _ => atual.push(c),
            }
        } else {
            match c {
                ',' => campos.push(std::mem::take(&mut atual)),
                '"' if atual.is_empty() => entre_aspas = true,
                _ => atual.push(c),
            }
        }
    }

    if entre_aspas {
        return Err(ErroLinha::AspasAbertas);
    }
    campos.push(atual);
    Ok(campos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn campo_com_virgula() {
        assert_eq!(codificar(&["a", "b,c"]), r#"a,"b,c""#);
        assert_eq!(decodificar(r#"a,"b,c""#).unwrap(), ["a", "b,c"]);
    }

    #[test]
    fn aspas_sem_fechar() {
        assert_eq!(decodificar(r#"a,"b"#), Err(ErroLinha::AspasAbertas));
    }

    /// O caso mínimo que o proptest encontrou na versão com bug (lição 08),
    /// guardado como teste com nome.
    #[test]
    fn campo_que_e_so_uma_aspa() {
        let campos = ["\""];
        assert_eq!(codificar(&campos), r#""""""#);
        assert_eq!(decodificar(&codificar(&campos)).unwrap(), campos);
    }

    proptest! {
        // O padrão é 256 casos por propriedade. Esta linha muda o número para
        // o bloco inteiro: útil quando cada caso é caro.
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Ida e volta: decodificar o que foi codificado devolve os mesmos campos.
        #[test]
        fn ida_e_volta(campos in prop::collection::vec("\\PC*", 1..5)) {
            let linha = codificar(&campos);
            prop_assert_eq!(decodificar(&linha), Ok(campos));
        }

        /// Invariante: o número de campos não muda.
        #[test]
        fn mesmo_numero_de_campos(campos in prop::collection::vec("[a-z,\"]{0,4}", 1..6)) {
            let n = decodificar(&codificar(&campos)).map(|c| c.len());
            prop_assert_eq!(n, Ok(campos.len()));
        }

        /// Um campo sem vírgula nem aspas sai do jeito que entrou.
        #[test]
        fn campo_simples_nao_ganha_aspas(campo in "[a-zA-Z0-9 ]*") {
            prop_assert_eq!(codificar(&[campo.as_str()]), campo);
        }
    }
}
