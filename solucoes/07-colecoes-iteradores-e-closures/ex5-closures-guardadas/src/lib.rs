/// Uma sequência de transformações aplicadas em ordem.
/// Cada etapa é uma closure diferente (tipos diferentes): `Box<dyn Fn>`.
#[derive(Default)]
pub struct Pipeline {
    etapas: Vec<Box<dyn Fn(i32) -> i32>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self::default()
    }

    /// Acrescenta uma etapa no fim. O `'static` exige que a closure não
    /// guarde empréstimos de variáveis locais (use `move`).
    pub fn etapa(mut self, f: impl Fn(i32) -> i32 + 'static) -> Self {
        self.etapas.push(Box::new(f));
        self
    }

    pub fn aplicar(&self, x: i32) -> i32 {
        self.etapas.iter().fold(x, |acc, f| f(acc))
    }

    pub fn len(&self) -> usize {
        self.etapas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.etapas.is_empty()
    }
}

/// Um filtro de linhas com um único predicado, conhecido na criação:
/// genérico, sem `Box` nem indireção.
pub struct Filtro<P> {
    pred: P,
}

impl<P: Fn(&str) -> bool> Filtro<P> {
    pub fn new(pred: P) -> Self {
        Filtro { pred }
    }

    pub fn filtrar<'a>(&self, linhas: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
        linhas.into_iter().filter(|l| (self.pred)(l)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_vazio_e_identidade() {
        let p = Pipeline::new();
        assert!(p.is_empty());
        assert_eq!(p.aplicar(7), 7);
    }

    #[test]
    fn pipeline_aplica_em_ordem() {
        let p = Pipeline::new().etapa(|x| x + 1).etapa(|x| x * 10);
        assert_eq!(p.aplicar(2), 30); // (2 + 1) * 10, e não 2 * 10 + 1
        assert_eq!(p.len(), 2);
    }

    #[test]
    fn pipeline_com_captura() {
        let deslocamento = 100;
        let limite = 150;
        let p = Pipeline::new()
            .etapa(move |x| x + deslocamento)
            .etapa(move |x| x.min(limite));
        assert_eq!(p.aplicar(20), 120);
        assert_eq!(p.aplicar(90), 150);
    }

    #[test]
    fn filtro_de_log() {
        let nivel = String::from("ERRO");
        let f = Filtro::new(|l: &str| l.starts_with(&nivel));
        let log = "ERRO disco cheio\nINFO ok\nERRO rede caiu";
        assert_eq!(
            f.filtrar(log.lines()),
            ["ERRO disco cheio", "ERRO rede caiu"]
        );
        // `nivel` foi só emprestado: continua aqui.
        assert_eq!(nivel, "ERRO");
    }

    #[test]
    fn filtro_aceita_vec_e_array() {
        let curtas = Filtro::new(|l: &str| l.len() <= 3);
        assert_eq!(curtas.filtrar(vec!["ab", "abcd"]), ["ab"]);
        assert_eq!(curtas.filtrar(["xyz", "", "long"]), ["xyz", ""]);
    }
}
