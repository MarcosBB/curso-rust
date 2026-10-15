/// Ordena da pessoa mais nova para a mais velha.
pub fn ordenar_por_idade(pessoas: &mut [(&str, u32)]) {
    pessoas.sort_by_key(|p| p.1);
}

/// Quantas pessoas têm 18 anos ou mais.
pub fn maiores_de_idade(pessoas: &[(&str, u32)]) -> usize {
    pessoas.iter().filter(|p| p.1 >= 18).count()
}

/// Os nomes, em letras maiúsculas, na ordem em que aparecem.
pub fn nomes_em_maiusculas(pessoas: &[(&str, u32)]) -> Vec<String> {
    pessoas.iter().map(|p| p.0.to_uppercase()).collect()
}

/// Remove as pessoas com idade menor que `minima`.
pub fn manter_a_partir_de(pessoas: &mut Vec<(&str, u32)>, minima: u32) {
    pessoas.retain(|p| p.1 >= minima);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordena() {
        let mut v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18), ("Duda", 7)];
        ordenar_por_idade(&mut v);
        assert_eq!(v, vec![("Duda", 7), ("Bia", 12), ("Caio", 18), ("Ana", 31)]);
    }

    #[test]
    fn conta_maiores() {
        let v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18), ("Duda", 7)];
        assert_eq!(maiores_de_idade(&v), 2);
        assert_eq!(maiores_de_idade(&[]), 0);
    }

    #[test]
    fn maiusculas() {
        let v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18)];
        assert_eq!(nomes_em_maiusculas(&v), vec!["ANA", "BIA", "CAIO"]);
    }

    #[test]
    fn filtra_no_lugar() {
        let mut v = vec![("Ana", 31), ("Bia", 12), ("Caio", 18), ("Duda", 7)];
        manter_a_partir_de(&mut v, 12);
        assert_eq!(v, vec![("Ana", 31), ("Bia", 12), ("Caio", 18)]);
    }
}
