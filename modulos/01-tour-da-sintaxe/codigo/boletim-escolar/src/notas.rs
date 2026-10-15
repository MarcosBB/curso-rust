/// Média das notas, ou `None` se a lista estiver vazia.
pub fn media(notas: &[f64]) -> Option<f64> {
    if notas.is_empty() {
        return None;
    }
    Some(soma(notas) / notas.len() as f64)
}

/// Conceito de A a D a partir da média (de 0 a 10).
pub fn conceito(media: f64) -> char {
    // `as u32` trunca: 8.95 * 10.0 = 89.5 vira 89.
    match (media * 10.0) as u32 {
        90.. => 'A',
        70..=89 => 'B',
        50..=69 => 'C',
        _ => 'D',
    }
}

// Privada: só este módulo (e os testes dele) enxergam.
fn soma(notas: &[f64]) -> f64 {
    let mut total = 0.0;
    for nota in notas {
        total += nota;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_simples() {
        assert_eq!(media(&[6.0, 8.0]), Some(7.0));
    }

    #[test]
    fn media_de_lista_vazia() {
        assert_eq!(media(&[]), None);
    }

    #[test]
    fn conceitos_nas_fronteiras() {
        assert_eq!(conceito(10.0), 'A');
        assert_eq!(conceito(9.0), 'A');
        assert_eq!(conceito(8.95), 'B');
        assert_eq!(conceito(5.0), 'C');
        assert_eq!(conceito(4.9), 'D', "4.9 ainda é reprovação");
    }

    #[test]
    fn soma_privada_e_testavel() {
        assert_eq!(soma(&[1.5, 2.5]), 4.0);
    }
}
