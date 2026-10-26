//! Separar os campos de uma linha de CSV sem copiar o que não precisa.
//! Lição 06 (retoma o benchmark do Projeto 2).
//!
//! Regras (um subconjunto da RFC 4180, sem quebra de linha dentro de campo):
//! campos separados por `,`; um campo entre aspas pode ter `,`, e `""`
//! dentro dele vale uma aspa. Para focar no `Cow`, não há tratamento de erro:
//! uma aspa sem fechamento vai até o fim da linha.

use std::borrow::Cow;

/// Separa os campos emprestando da linha sempre que possível.
///
/// Só os campos com `""` precisam de uma `String` nova.
///
/// ```
/// use std::borrow::Cow;
/// use cow_csv::campos;
///
/// let c = campos(r#"Ana,"Rio, RJ","diz ""oi""""#);
/// assert_eq!(c, ["Ana", "Rio, RJ", r#"diz "oi""#]);
/// assert!(matches!(c[1], Cow::Borrowed(_)));
/// assert!(matches!(c[2], Cow::Owned(_)));
/// ```
pub fn campos(linha: &str) -> Vec<Cow<'_, str>> {
    let mut saida = Vec::new();
    let mut resto = linha;
    loop {
        let (campo, depois) = match resto.strip_prefix('"') {
            Some(dentro) => campo_com_aspas(dentro),
            None => match resto.split_once(',') {
                Some((campo, depois)) => (Cow::Borrowed(campo), Some(depois)),
                None => (Cow::Borrowed(resto), None),
            },
        };
        saida.push(campo);
        match depois {
            Some(d) => resto = d,
            None => return saida,
        }
    }
}

/// Recebe o texto logo depois da aspa de abertura. Devolve o campo e o que
/// vem depois da vírgula (ou `None` se a linha acabou).
fn campo_com_aspas(dentro: &str) -> (Cow<'_, str>, Option<&str>) {
    let bytes = dentro.as_bytes();
    let mut i = 0;
    let mut tem_escape = false;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            if bytes.get(i + 1) == Some(&b'"') {
                tem_escape = true;
                i += 2;
                continue;
            }
            break;
        }
        i += 1;
    }
    let conteudo = &dentro[..i];
    let campo = if tem_escape {
        Cow::Owned(conteudo.replace("\"\"", "\""))
    } else {
        Cow::Borrowed(conteudo)
    };
    // Pula a aspa de fechamento e procura a vírgula seguinte.
    let depois = dentro
        .get(i + 1..)
        .and_then(|r| r.split_once(','))
        .map(|(_, d)| d);
    (campo, depois)
}

/// A versão "como no Projeto 2": uma `String` por campo.
pub fn campos_string(linha: &str) -> Vec<String> {
    campos(linha).into_iter().map(Cow::into_owned).collect()
}

/// Bytes que deveriam ser UTF-8. Se forem, nada é copiado; se não forem,
/// os bytes inválidos viram `�` numa `String` nova.
pub fn texto_de_bytes(bytes: &[u8]) -> Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simples_so_empresta() {
        let c = campos("1,nome,x");
        assert_eq!(c, ["1", "nome", "x"]);
        assert!(c.iter().all(|f| matches!(f, Cow::Borrowed(_))));
    }

    #[test]
    fn aspas_e_escape() {
        let c = campos(r#""a,b","",c,"x""y""#);
        assert_eq!(c, ["a,b", "", "c", r#"x"y"#]);
        assert!(matches!(c[0], Cow::Borrowed(_)));
        assert!(matches!(c[3], Cow::Owned(_)));
    }

    #[test]
    fn campos_vazios() {
        assert_eq!(campos(",,"), ["", "", ""]);
        assert_eq!(campos(""), [""]);
    }

    #[test]
    fn as_duas_versoes_concordam() {
        let linha = r#"10,"nome ""10""","texto, com vírgula""#;
        assert_eq!(campos_string(linha), campos(linha));
    }

    #[test]
    fn lossy() {
        assert!(matches!(texto_de_bytes(b"ola"), Cow::Borrowed("ola")));
        let t = texto_de_bytes(b"ol\xffa");
        assert_eq!(t, "ol\u{FFFD}a");
        assert!(matches!(t, Cow::Owned(_)));
    }
}
