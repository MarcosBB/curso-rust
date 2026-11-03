/// Um "iterador" cujos itens podem emprestar do próprio emprestador.
pub trait Emprestador {
    type Item<'a>
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<Self::Item<'_>>;
}

/// Janelas mutáveis e sobrepostas de tamanho `n`.
pub struct JanelasMut<'d, T> {
    dados: &'d mut [T],
    n: usize,
    pos: usize,
}

impl<'d, T> JanelasMut<'d, T> {
    pub fn new(dados: &'d mut [T], n: usize) -> Self {
        assert!(n > 0, "janela de tamanho zero");
        JanelasMut { dados, n, pos: 0 }
    }
}

impl<T> Emprestador for JanelasMut<'_, T> {
    type Item<'a>
        = &'a mut [T]
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&mut [T]> {
        let janela = self.dados.get_mut(self.pos..self.pos + self.n)?;
        self.pos += 1;
        Some(janela)
    }
}

/// Palavras normalizadas (minúsculas, só letras e dígitos), num buffer reaproveitado.
pub struct Palavras<'t> {
    resto: std::str::SplitWhitespace<'t>,
    buf: String,
}

impl<'t> Palavras<'t> {
    pub fn new(texto: &'t str) -> Self {
        Palavras {
            resto: texto.split_whitespace(),
            buf: String::new(),
        }
    }
}

impl Emprestador for Palavras<'_> {
    type Item<'a>
        = &'a str
    where
        Self: 'a;

    fn proximo(&mut self) -> Option<&str> {
        loop {
            let palavra = self.resto.next()?;
            self.buf.clear();
            self.buf.extend(
                palavra
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .flat_map(char::to_lowercase),
            );
            // Pula "palavras" que eram só pontuação, como "--".
            if !self.buf.is_empty() {
                return Some(&self.buf);
            }
        }
    }
}

/// Quantas palavras (normalizadas) começam com `prefixo`.
pub fn contar_com_prefixo(texto: &str, prefixo: &str) -> usize {
    let mut p = Palavras::new(texto);
    let mut n = 0;
    while let Some(palavra) = p.proximo() {
        if palavra.starts_with(prefixo) {
            n += 1;
        }
    }
    n
}

/// Em cada janela de tamanho `n`, soma o primeiro elemento ao último.
/// Com `n = 2`, é a soma acumulada.
pub fn propagar(v: &mut [i64], n: usize) {
    let mut j = JanelasMut::new(v, n);
    while let Some(janela) = j.proximo() {
        let primeiro = janela[0];
        if let Some(ultimo) = janela.last_mut() {
            *ultimo += primeiro;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn janelas_de_tamanho_n() {
        let mut v = [1, 2, 3, 4, 5];
        let mut j = JanelasMut::new(&mut v, 3);
        let mut vistas = Vec::new();
        while let Some(w) = j.proximo() {
            vistas.push(w.to_vec());
            w[0] *= 10;
        }
        assert_eq!(vistas, [vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]]);
        assert_eq!(v, [10, 20, 30, 4, 5]);
    }

    #[test]
    fn janela_maior_que_os_dados() {
        let mut v = [1, 2];
        assert!(JanelasMut::new(&mut v, 3).proximo().is_none());
    }

    #[test]
    fn propagar_soma_acumulada() {
        let mut v = [1, 2, 3, 4];
        propagar(&mut v, 2);
        assert_eq!(v, [1, 3, 6, 10]);
        let mut w = [1, 0, 0, 0, 0];
        propagar(&mut w, 3);
        assert_eq!(w, [1, 0, 1, 0, 1]);
    }

    #[test]
    fn palavras_normalizadas() {
        let mut p = Palavras::new("O Rato, -- roeu!");
        assert_eq!(p.proximo(), Some("o"));
        assert_eq!(p.proximo(), Some("rato"));
        assert_eq!(p.proximo(), Some("roeu"));
        assert_eq!(p.proximo(), None);
    }

    #[test]
    fn conta_por_prefixo() {
        let texto = "O rato roeu a roupa do Rei de Roma.";
        assert_eq!(contar_com_prefixo(texto, "r"), 5);
        assert_eq!(contar_com_prefixo("-- ... !", "a"), 0);
    }
}
