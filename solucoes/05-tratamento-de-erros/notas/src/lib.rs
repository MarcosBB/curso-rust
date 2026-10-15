//! Biblioteca: lê um arquivo de notas (`nome;nota`, uma por linha) e calcula a média.

use std::fs;
use std::io;
use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotasError {
    #[error("não consegui ler {path}")]
    Io {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("linha {line}: esperava `nome;nota`")]
    MissingSeparator { line: usize },

    // Sem `{source}` no texto: a causa sai pelo source().
    #[error("linha {line}: nota inválida")]
    BadGrade {
        line: usize,
        #[source]
        source: ParseIntError,
    },

    #[error("linha {line}: nota {grade} fora do intervalo 0 a 100")]
    OutOfRange { line: usize, grade: u32 },

    #[error("nenhum aluno no arquivo")]
    Empty,
}

pub fn load(path: &str) -> Result<Vec<(String, u32)>, NotasError> {
    let text = fs::read_to_string(path).map_err(|source| NotasError::Io {
        path: path.to_string(),
        source,
    })?;
    parse(&text)
}

pub fn parse(text: &str) -> Result<Vec<(String, u32)>, NotasError> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        if raw.trim().is_empty() {
            continue;
        }
        let (name, grade) = raw
            .split_once(';')
            .ok_or(NotasError::MissingSeparator { line })?;
        let grade: u32 = grade
            .trim()
            .parse()
            .map_err(|source| NotasError::BadGrade { line, source })?;
        if grade > 100 {
            return Err(NotasError::OutOfRange { line, grade });
        }
        out.push((name.trim().to_string(), grade));
    }
    Ok(out)
}

pub fn average(notas: &[(String, u32)]) -> Result<f64, NotasError> {
    if notas.is_empty() {
        return Err(NotasError::Empty);
    }
    let sum: u32 = notas.iter().map(|(_, g)| g).sum();
    Ok(f64::from(sum) / notas.len() as f64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn media_de_uma_turma() {
        let notas = parse("ana;90\n\nbruno; 70\n").unwrap();
        assert_eq!(
            notas,
            vec![("ana".to_string(), 90), ("bruno".to_string(), 70)]
        );
        assert_eq!(average(&notas).unwrap(), 80.0);
    }

    #[test]
    fn variantes() {
        assert!(matches!(
            parse("ana 90"),
            Err(NotasError::MissingSeparator { line: 1 })
        ));
        assert!(matches!(
            parse("ana;90\nbia;101"),
            Err(NotasError::OutOfRange {
                line: 2,
                grade: 101
            })
        ));
        assert!(matches!(average(&[]), Err(NotasError::Empty)));
    }

    #[test]
    fn nota_invalida_tem_causa() {
        let e = parse("ana;90\nbruno;setenta").unwrap_err();
        assert!(matches!(e, NotasError::BadGrade { line: 2, .. }));
        assert_eq!(e.to_string(), "linha 2: nota inválida");
        assert!(e.source().is_some());
    }

    #[test]
    fn arquivo_inexistente_tem_o_caminho_e_a_causa() {
        let e = load("nao-existe.txt").unwrap_err();
        assert_eq!(e.to_string(), "não consegui ler nao-existe.txt");
        let NotasError::Io { source, .. } = &e else {
            panic!("esperava NotasError::Io, veio {e:?}");
        };
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn cadeia_sem_mensagem_duplicada() {
        let e = anyhow::Error::from(parse("bruno;setenta").unwrap_err())
            .context("falha ao calcular a média");
        assert_eq!(
            format!("{e:#}"),
            "falha ao calcular a média: linha 1: nota inválida: invalid digit found in string"
        );
    }
}
