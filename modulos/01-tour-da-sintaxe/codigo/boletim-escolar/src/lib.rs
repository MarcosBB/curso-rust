//! Boletim escolar: exemplo do kit mínimo do Módulo 01
//! (`Vec`, `HashMap`, closures, módulos e testes).

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
}
