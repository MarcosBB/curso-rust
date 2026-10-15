//! Solução do exercício 6: o `Documento` do Python, traduzido para Rust.

pub struct Documento {
    pub palavras: Vec<String>,
}

/// Toma posse do vetor: o documento passa a ser o dono das palavras.
pub fn novo(palavras: Vec<String>) -> Documento {
    Documento { palavras }
}

/// Consome o documento, acrescenta a palavra e devolve a posse.
pub fn com_palavra(mut doc: Documento, palavra: &str) -> Documento {
    doc.palavras.push(palavra.to_string());
    doc
}

/// Só lê o original (`&Documento`) e cria um documento independente.
pub fn copiar(doc: &Documento) -> Documento {
    Documento {
        palavras: doc.palavras.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn novo_guarda_as_palavras() {
        let d = novo(vec![String::from("olá")]);
        assert_eq!(d.palavras, ["olá"]);
    }

    #[test]
    fn com_palavra_acrescenta_no_fim() {
        let d = novo(vec![String::from("olá")]);
        let d = com_palavra(d, "mundo");
        assert_eq!(d.palavras, ["olá", "mundo"]);
    }

    #[test]
    fn copia_e_independente_do_original() {
        let d = novo(vec![String::from("olá")]);
        let d2 = copiar(&d);
        let d2 = com_palavra(d2, "mundo");
        assert_eq!(d.palavras, ["olá"]);
        assert_eq!(d2.palavras, ["olá", "mundo"]);
    }
}
