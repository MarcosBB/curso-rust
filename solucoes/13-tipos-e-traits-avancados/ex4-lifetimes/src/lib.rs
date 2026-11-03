use std::fmt::Debug;

/// a) Os índices do miolo: sem a primeira e a última posição.
pub fn indices_do_meio(v: &[u8]) -> impl Iterator<Item = usize> + use<> {
    1..v.len().saturating_sub(1)
}

/// b) Guarda qualquer valor `Debug` numa lista de caixas.
pub fn guardar<'a, T: Debug + 'a>(caixas: &mut Vec<Box<dyn Debug + 'a>>, x: T) {
    caixas.push(Box::new(x));
}

/// c) O maior e o menor de qualquer coleção que itera `&i32`.
pub fn extremos<C>(c: C) -> Option<(i32, i32)>
where
    for<'a> &'a C: IntoIterator<Item = &'a i32>,
{
    let maior = (&c).into_iter().max()?;
    let menor = (&c).into_iter().min()?;
    Some((*maior, *menor))
}

/// d) Acrescenta um evento ao log.
pub fn registrar<'a>(log: &mut Vec<&'a str>, evento: &'a str) {
    log.push(evento);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn indices_nao_prendem_o_vetor() {
        let mut v = vec![1, 2, 3, 4, 5];
        let it = indices_do_meio(&v);
        v.clear();
        assert_eq!(it.collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn guarda_valores_e_emprestimos() {
        let texto = String::from("emprestado");
        let mut caixas = Vec::new();
        guardar(&mut caixas, 42);
        guardar(&mut caixas, &texto);
        assert_eq!(format!("{caixas:?}"), r#"[42, "emprestado"]"#);
    }

    #[test]
    fn extremos_de_varias_colecoes() {
        assert_eq!(extremos(vec![3, 1, 2]), Some((3, 1)));
        assert_eq!(extremos([7]), Some((7, 7)));
        assert_eq!(extremos(BTreeSet::from([5, -2])), Some((5, -2)));
        assert_eq!(extremos(Vec::new()), None);
    }

    #[test]
    fn registra_eventos_locais() {
        let mut log = vec!["inicio"];
        let evento = format!("login de {}", "ana");
        registrar(&mut log, &evento);
        assert_eq!(log, ["inicio", "login de ana"]);
    }
}
