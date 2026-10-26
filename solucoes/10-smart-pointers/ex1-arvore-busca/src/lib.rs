//! Exercício 1: árvore binária de busca com `Option<Box<No>>`.

#[derive(Debug, Default)]
pub struct Arvore {
    raiz: Option<Box<No>>,
}

#[derive(Debug)]
struct No {
    valor: i32,
    esq: Option<Box<No>>,
    dir: Option<Box<No>>,
}

impl Arvore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insere `valor`. Devolve `false` se ele já estava na árvore.
    pub fn inserir(&mut self, valor: i32) -> bool {
        // `cursor` é um &mut para o lugar onde o valor deve entrar.
        let mut cursor = &mut self.raiz;
        while let Some(no) = cursor {
            if valor == no.valor {
                return false;
            }
            cursor = if valor < no.valor {
                &mut no.esq
            } else {
                &mut no.dir
            };
        }
        *cursor = Some(Box::new(No {
            valor,
            esq: None,
            dir: None,
        }));
        true
    }

    pub fn contem(&self, valor: i32) -> bool {
        let mut cursor = self.raiz.as_deref();
        while let Some(no) = cursor {
            if valor == no.valor {
                return true;
            }
            cursor = if valor < no.valor {
                no.esq.as_deref()
            } else {
                no.dir.as_deref()
            };
        }
        false
    }

    /// Os valores em ordem crescente.
    pub fn em_ordem(&self) -> Vec<i32> {
        fn visitar(no: Option<&No>, saida: &mut Vec<i32>) {
            if let Some(no) = no {
                visitar(no.esq.as_deref(), saida);
                saida.push(no.valor);
                visitar(no.dir.as_deref(), saida);
            }
        }
        let mut saida = Vec::new();
        visitar(self.raiz.as_deref(), &mut saida);
        saida
    }

    /// Número de nós no caminho mais longo da raiz até uma folha (0 se vazia).
    pub fn altura(&self) -> usize {
        fn altura(no: Option<&No>) -> usize {
            no.map_or(0, |n| {
                1 + altura(n.esq.as_deref()).max(altura(n.dir.as_deref()))
            })
        }
        altura(self.raiz.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vazia() {
        let a = Arvore::new();
        assert!(!a.contem(1));
        assert!(a.em_ordem().is_empty());
        assert_eq!(a.altura(), 0);
    }

    #[test]
    fn insere_e_busca() {
        let mut a = Arvore::new();
        for v in [50, 30, 70, 20, 40, 60, 80] {
            assert!(a.inserir(v));
        }
        assert!(!a.inserir(40), "repetido não entra");
        assert!(a.contem(60));
        assert!(!a.contem(65));
        assert_eq!(a.em_ordem(), [20, 30, 40, 50, 60, 70, 80]);
        assert_eq!(a.altura(), 3);
    }

    #[test]
    fn degenerada() {
        let mut a = Arvore::new();
        for v in 1..=5 {
            a.inserir(v);
        }
        assert_eq!(a.altura(), 5);
    }
}
