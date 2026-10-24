//! Um relatório de notas escrito só com iteradores (lições 06 e 07).

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Aluno {
    pub nome: String,
    pub nota: f64,
}

/// Lê linhas `nome;nota`. Ignora linhas vazias e comentários (`#`).
/// Para no primeiro erro, com o número da linha (`collect` em `Result`).
pub fn ler(texto: &str) -> Result<Vec<Aluno>, String> {
    texto
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty() && !l.starts_with('#'))
        .map(|(n, l)| {
            let (nome, nota) = l
                .split_once(';')
                .ok_or_else(|| format!("linha {n}: falta o `;`"))?;
            let nota: f64 = nota
                .trim()
                .parse()
                .map_err(|_| format!("linha {n}: nota inválida `{}`", nota.trim()))?;
            Ok(Aluno {
                nome: nome.trim().to_string(),
                nota,
            })
        })
        .collect()
}

/// Média das notas, ou `None` se não há alunos.
pub fn media(alunos: &[Aluno]) -> Option<f64> {
    if alunos.is_empty() {
        return None;
    }
    let soma: f64 = alunos.iter().map(|a| a.nota).sum();
    Some(soma / alunos.len() as f64)
}

/// O aluno com a maior nota. `f64` não é `Ord`: `max_by` com `total_cmp`.
pub fn melhor(alunos: &[Aluno]) -> Option<&Aluno> {
    alunos.iter().max_by(|a, b| a.nota.total_cmp(&b.nota))
}

/// Nomes dos aprovados e dos reprovados, nessa ordem.
pub fn aprovados_e_reprovados<'a>(alunos: &'a [Aluno], corte: f64) -> (Vec<&'a str>, Vec<&'a str>) {
    let (aprovados, reprovados): (Vec<&Aluno>, Vec<&Aluno>) =
        alunos.iter().partition(|a| a.nota >= corte);
    // Uma closure que recebe e devolve referências não segue a elisão das
    // `fn` (lição 04). Sem o `'a` explícito nos dois lados, este é o erro
    // "lifetime may not live long enough".
    let nomes =
        |v: Vec<&'a Aluno>| -> Vec<&'a str> { v.into_iter().map(|a| a.nome.as_str()).collect() };
    (nomes(aprovados), nomes(reprovados))
}

pub fn conceito(nota: f64) -> char {
    match nota {
        n if n >= 9.0 => 'A',
        n if n >= 7.0 => 'B',
        n if n >= 5.0 => 'C',
        _ => 'D',
    }
}

/// Alunos agrupados por conceito, em ordem (`BTreeMap` + `or_default`).
pub fn por_conceito(alunos: &[Aluno]) -> BTreeMap<char, Vec<&str>> {
    let mut grupos: BTreeMap<char, Vec<&str>> = BTreeMap::new();
    for a in alunos {
        grupos.entry(conceito(a.nota)).or_default().push(&a.nome);
    }
    grupos
}

#[cfg(test)]
mod tests {
    use super::*;

    const TURMA: &str = "\
# nome;nota
Ana; 9.5
Bia;6
Caio; 7.25

Davi;4.5
";

    #[test]
    fn le_ignorando_comentarios_e_vazias() {
        let alunos = ler(TURMA).unwrap();
        assert_eq!(alunos.len(), 4);
        assert_eq!(
            alunos[2],
            Aluno {
                nome: "Caio".to_string(),
                nota: 7.25
            }
        );
    }

    #[test]
    fn erro_com_numero_da_linha() {
        assert_eq!(
            ler("Ana;9\nBia;x"),
            Err("linha 2: nota inválida `x`".to_string())
        );
        assert_eq!(ler("Ana 9"), Err("linha 1: falta o `;`".to_string()));
    }

    #[test]
    fn estatisticas() {
        let alunos = ler(TURMA).unwrap();
        assert_eq!(media(&alunos), Some(6.8125));
        assert_eq!(media(&[]), None);
        assert_eq!(melhor(&alunos).map(|a| a.nome.as_str()), Some("Ana"));
    }

    #[test]
    fn separa_aprovados() {
        let alunos = ler(TURMA).unwrap();
        let (sim, nao) = aprovados_e_reprovados(&alunos, 6.0);
        assert_eq!(sim, ["Ana", "Bia", "Caio"]);
        assert_eq!(nao, ["Davi"]);
    }

    #[test]
    fn agrupa_por_conceito() {
        let alunos = ler(TURMA).unwrap();
        let grupos: Vec<_> = por_conceito(&alunos).into_iter().collect();
        assert_eq!(
            grupos,
            [
                ('A', vec!["Ana"]),
                ('B', vec!["Caio"]),
                ('C', vec!["Bia"]),
                ('D', vec!["Davi"])
            ]
        );
    }
}
