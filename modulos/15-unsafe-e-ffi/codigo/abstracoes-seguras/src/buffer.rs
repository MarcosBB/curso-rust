//! Um leitor de bytes cuja invariante protege um acesso sem verificação.

/// Lê bytes em sequência.
///
/// Invariante: `pos <= dados.len()`. Os campos são privados, então só o
/// código deste módulo pode quebrar a invariante.
pub struct Leitor {
    dados: Vec<u8>,
    pos: usize,
}

impl Leitor {
    pub fn new(dados: Vec<u8>) -> Self {
        Leitor { dados, pos: 0 }
    }

    /// Próximo byte, ou `None` no fim.
    pub fn proximo(&mut self) -> Option<u8> {
        if self.pos == self.dados.len() {
            return None;
        }
        // SAFETY: pela invariante, `pos <= len`, e acabamos de ver que
        // `pos != len`; então `pos < len` e o índice é válido.
        let b = unsafe { *self.dados.get_unchecked(self.pos) };
        self.pos += 1;
        Some(b)
    }

    /// Avança `n` bytes, sem passar do fim.
    pub fn pular(&mut self, n: usize) {
        // Esta função não tem `unsafe`, mas precisa manter a invariante:
        // um `self.pos += n` simples faria `proximo` ler fora do vetor.
        self.pos = (self.pos + n).min(self.dados.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_e_pula() {
        let mut l = Leitor::new(vec![1, 2, 3, 4]);
        assert_eq!(l.proximo(), Some(1));
        l.pular(2);
        assert_eq!(l.proximo(), Some(4));
        assert_eq!(l.proximo(), None);
    }

    #[test]
    fn pular_alem_do_fim() {
        let mut l = Leitor::new(vec![1, 2]);
        l.pular(100);
        assert_eq!(l.proximo(), None);
    }
}
