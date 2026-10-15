#[derive(Debug, PartialEq)]
pub struct Item {
    pub nome: String,
    pub preco: u32,
}

pub struct Inventario {
    pub itens: Vec<Item>,
    pub ouro: u32,
}

impl Inventario {
    pub fn mais_caro(&self) -> Option<&Item> {
        self.itens.iter().max_by_key(|i| i.preco)
    }

    pub fn media(&self) -> u32 {
        if self.itens.is_empty() {
            return 0;
        }
        let total: u32 = self.itens.iter().map(|i| i.preco).sum();
        total / self.itens.len() as u32
    }

    /// Compra uma réplica do item mais caro: custa 1/10 do preço, tem esse
    /// mesmo preço e o nome "<nome> (réplica)". Devolve `false` se não há
    /// itens ou se falta ouro.
    pub fn compra_replica(&mut self) -> bool {
        // 1. Monta a réplica enquanto `caro` está emprestado. O `format!`
        //    cria uma String nova a partir de `&caro.nome` (não é clone).
        let replica = match self.mais_caro() {
            Some(caro) => Item {
                nome: format!("{} (réplica)", caro.nome),
                preco: caro.preco / 10,
            },
            None => return false,
        };
        // 2. Daqui em diante `caro` não é mais usado: o empréstimo acabou,
        //    e `self` pode ser alterado à vontade.
        if self.ouro < replica.preco {
            return false;
        }
        self.ouro -= replica.preco;
        self.itens.push(replica);
        true
    }

    /// Soma o preço do item `de` no item `para` e zera o `de`. Devolve
    /// `false` (sem mudar nada) se os índices forem iguais ou inválidos.
    pub fn funde(&mut self, de: usize, para: usize) -> bool {
        // Dois `&mut` em posições diferentes do mesmo slice: `get_disjoint_mut`
        // confere em runtime que os índices são válidos e distintos.
        match self.itens.get_disjoint_mut([de, para]) {
            Ok([origem, destino]) => {
                destino.preco += origem.preco;
                origem.preco = 0;
                true
            }
            Err(_) => false,
        }
    }

    /// Todo item acima da média passa a custar a média (calculada antes
    /// de qualquer mudança).
    pub fn limita_a_media(&mut self) {
        // Copia o valor (um u32) antes do laço. Dentro do laço, `self.itens`
        // está emprestado como `&mut`, e `self.media()` pediria `&self` inteiro.
        let media = self.media();
        for item in &mut self.itens {
            if item.preco > media {
                item.preco = media;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(nome: &str, preco: u32) -> Item {
        Item {
            nome: nome.to_string(),
            preco,
        }
    }

    fn inventario() -> Inventario {
        Inventario {
            itens: vec![item("espada", 300), item("escudo", 120), item("poção", 30)],
            ouro: 50,
        }
    }

    #[test]
    fn compra_replica_do_mais_caro() {
        let mut inv = inventario();
        assert!(inv.compra_replica());
        assert_eq!(inv.ouro, 20);
        assert_eq!(inv.itens[3], item("espada (réplica)", 30));
    }

    #[test]
    fn compra_replica_sem_ouro() {
        let mut inv = inventario();
        inv.ouro = 10;
        assert!(!inv.compra_replica());
        assert_eq!(inv.itens.len(), 3);
        assert_eq!(inv.ouro, 10);
    }

    #[test]
    fn funde_dois_itens() {
        let mut inv = inventario();
        assert!(inv.funde(2, 0));
        assert_eq!(inv.itens[0].preco, 330);
        assert_eq!(inv.itens[2].preco, 0);
    }

    #[test]
    fn funde_recusa_indices_iguais_ou_invalidos() {
        let mut inv = inventario();
        assert!(!inv.funde(1, 1));
        assert!(!inv.funde(0, 9));
        assert_eq!(inv.itens[1].preco, 120);
    }

    #[test]
    fn limita_pela_media_original() {
        let mut inv = inventario();
        inv.limita_a_media(); // média = 150
        let precos: Vec<u32> = inv.itens.iter().map(|i| i.preco).collect();
        assert_eq!(precos, [150, 120, 30]);
    }
}
