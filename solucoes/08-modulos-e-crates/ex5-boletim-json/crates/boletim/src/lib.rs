//! Boletim escolar: a biblioteca do workspace do exercício 4 (Módulo 08),
//! a partir do exemplo da lição 09 do Módulo 01.

pub mod notas;

use std::collections::HashMap;

/// Agrupa os lançamentos `(aluno, nota)` por aluno.
pub fn agrupar(lancamentos: Vec<(&str, f64)>) -> HashMap<String, Vec<f64>> {
    let mut turma: HashMap<String, Vec<f64>> = HashMap::new();
    for (aluno, nota) in lancamentos {
        turma.entry(aluno.to_string()).or_default().push(nota);
    }
    turma
}

/// Devolve os nomes dos alunos em ordem alfabética.
pub fn nomes_ordenados(turma: &HashMap<String, Vec<f64>>) -> Vec<String> {
    let mut nomes: Vec<String> = turma.keys().cloned().collect();
    nomes.sort();
    nomes
}

/// A turma em JSON, com os alunos em ordem alfabética: `{"Ana":[9.5,8],"Bruno":[6]}`.
#[cfg(feature = "json")]
pub fn para_json(turma: &HashMap<String, Vec<f64>>) -> String {
    let campos: Vec<String> = nomes_ordenados(turma)
        .iter()
        .map(|nome| {
            let notas: Vec<String> = turma[nome].iter().map(|n| n.to_string()).collect();
            format!("\"{nome}\":[{}]", notas.join(","))
        })
        .collect();
    format!("{{{}}}", campos.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrupa_notas_do_mesmo_aluno() {
        let turma = agrupar(vec![("Ana", 9.0), ("Bruno", 6.0), ("Ana", 7.0)]);
        assert_eq!(turma.len(), 2);
        assert_eq!(turma["Ana"], vec![9.0, 7.0]);
        assert_eq!(turma.get("Zé"), None);
    }

    #[test]
    fn nomes_saem_em_ordem() {
        let turma = agrupar(vec![("Carla", 5.0), ("Ana", 9.0), ("Bruno", 6.0)]);
        assert_eq!(nomes_ordenados(&turma), vec!["Ana", "Bruno", "Carla"]);
    }

    #[cfg(feature = "json")]
    #[test]
    fn exporta_json_em_ordem() {
        let turma = agrupar(vec![("Bruno", 6.0), ("Ana", 9.5), ("Ana", 8.0)]);
        assert_eq!(para_json(&turma), r#"{"Ana":[9.5,8],"Bruno":[6]}"#);
    }

    #[cfg(feature = "json")]
    #[test]
    fn turma_vazia_vira_objeto_vazio() {
        assert_eq!(para_json(&HashMap::new()), "{}");
    }
}
