//! Exemplo da lição 10 do Módulo 09: `rstest` (casos e *fixtures*),
//! `insta` (snapshots) e um *mock* feito com uma trait.

/// O conceito de uma nota de 0 a 10 (o mesmo do boletim do Módulo 01).
pub fn conceito(nota: f64) -> char {
    match nota {
        n if n >= 9.0 => 'A',
        n if n >= 7.0 => 'B',
        n if n >= 5.0 => 'C',
        _ => 'D',
    }
}

/// Monta o boletim: uma linha por aluno, com a nota e o conceito.
pub fn boletim(alunos: &[(&str, f64)]) -> String {
    let mut saida = String::new();
    for (nome, nota) in alunos {
        saida.push_str(&format!("{nome:<8} {nota:>4.1}  {}\n", conceito(*nota)));
    }
    saida
}

/// De onde vem a hora atual. Em produção, do relógio do sistema; nos testes,
/// de um relógio falso que o teste controla.
pub trait Relogio {
    /// Segundos desde 1970-01-01 (*Unix time*).
    fn agora(&self) -> u64;
}

/// O relógio de verdade.
pub struct RelogioDoSistema;

impl Relogio for RelogioDoSistema {
    fn agora(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
    }
}

/// Um prazo venceu? Recebe o relógio como parâmetro para poder ser testada.
pub fn venceu(prazo: u64, relogio: &impl Relogio) -> bool {
    relogio.agora() > prazo
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::{fixture, rstest};

    // ---------- rstest: um teste por caso ----------

    #[rstest]
    #[case::dez(10.0, 'A')]
    #[case::fronteira_do_a(9.0, 'A')]
    #[case::quase_a(8.95, 'B')]
    #[case::fronteira_do_c(5.0, 'C')]
    #[case::reprovado(4.9, 'D')]
    fn conceitos(#[case] nota: f64, #[case] esperado: char) {
        assert_eq!(conceito(nota), esperado);
    }

    /// *Fixture*: o rstest injeta o valor no parâmetro de mesmo nome.
    #[fixture]
    fn turma() -> Vec<(&'static str, f64)> {
        vec![("ana", 9.5), ("bruno", 6.0), ("carla", 4.0)]
    }

    #[rstest]
    fn turma_tem_um_reprovado(turma: Vec<(&'static str, f64)>) {
        let reprovados = turma.iter().filter(|(_, n)| conceito(*n) == 'D').count();
        assert_eq!(reprovados, 1);
    }

    // ---------- insta: o valor esperado fica gravado no próprio teste ----------

    #[rstest]
    fn boletim_da_turma(turma: Vec<(&'static str, f64)>) {
        insta::assert_snapshot!(boletim(&turma), @"
        ana       9.5  A
        bruno     6.0  C
        carla     4.0  D
        ");
    }

    // ---------- mock sem biblioteca: uma trait e uma implementação de teste ----------

    struct RelogioParado(u64);

    impl Relogio for RelogioParado {
        fn agora(&self) -> u64 {
            self.0
        }
    }

    #[test]
    fn prazo_vencido() {
        assert!(venceu(100, &RelogioParado(101)));
        assert!(!venceu(100, &RelogioParado(100)));
    }

    #[test]
    fn relogio_do_sistema_anda() {
        assert!(RelogioDoSistema.agora() > 1_700_000_000);
    }
}
