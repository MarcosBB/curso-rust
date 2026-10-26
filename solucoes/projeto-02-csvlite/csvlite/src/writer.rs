use std::io::{self, Write};

/// Escreve um registro em `out`, terminado por `\n`.
///
/// Um campo só ganha aspas quando precisa: quando contém o separador, `"`,
/// `\n` ou `\r`. Dentro das aspas, cada `"` é duplicada. O resultado é lido
/// de volta pelo [`Reader`](crate::Reader) exatamente como foi escrito.
///
/// Funciona com qualquer destino que implemente [`Write`] (um arquivo, a
/// saída padrão, um `Vec<u8>`) e com campos de qualquer tipo que se veja como
/// `&str` (`&str`, `String`...).
///
/// ```
/// let mut saida = Vec::new();
/// csvlite::write_record(&mut saida, &["Ana", "Rio; RJ", "diz \"oi\""], ';')?;
/// assert_eq!(saida, b"Ana;\"Rio; RJ\";\"diz \"\"oi\"\"\"\n");
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Errors
///
/// Devolve o erro de escrita de `out`.
pub fn write_record<W: Write, S: AsRef<str>>(
    out: &mut W,
    fields: &[S],
    delimiter: char,
) -> io::Result<()> {
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            write!(out, "{delimiter}")?;
        }
        let field = field.as_ref();
        if field.contains([delimiter, '"', '\n', '\r']) {
            write!(out, "\"{}\"", field.replace('"', "\"\""))?;
        } else {
            out.write_all(field.as_bytes())?;
        }
    }
    writeln!(out)
}

#[cfg(test)]
mod step5_tests {
    use super::*;

    fn write(fields: &[&str], delimiter: char) -> String {
        let mut out = Vec::new();
        write_record(&mut out, fields, delimiter).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn plain_fields_have_no_quotes() {
        assert_eq!(write(&["a", "b", ""], ','), "a,b,\n");
    }

    #[test]
    fn delimiter_needs_quotes() {
        assert_eq!(write(&["a,b", "c"], ','), "\"a,b\",c\n");
    }

    #[test]
    fn quotes_are_doubled() {
        assert_eq!(write(&["diz \"oi\""], ','), "\"diz \"\"oi\"\"\"\n");
    }

    #[test]
    fn newlines_need_quotes() {
        assert_eq!(write(&["a\nb", "c\rd"], ','), "\"a\nb\",\"c\rd\"\n");
    }

    #[test]
    fn semicolon_delimiter() {
        // Com `;`, a vírgula é texto comum, e o `;` é que pede aspas.
        assert_eq!(write(&["9,5", "Rio; RJ"], ';'), "9,5;\"Rio; RJ\"\n");
    }

    #[test]
    fn accepts_strings() {
        let mut out = Vec::new();
        let fields = vec![String::from("x"), String::from("y")];
        write_record(&mut out, &fields, ',').unwrap();
        assert_eq!(out, b"x,y\n");
    }
}
