use std::cmp::Reverse;

use serde::{Deserialize, Serialize};

use crate::Erro;
use crate::tarefa::{Prioridade, Tarefa};

/// Todas as tarefas, mais o próximo id a usar. É o que vai para o arquivo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListaDeTarefas {
    /// Versão do formato do arquivo (veja "Para ir além" no enunciado).
    versao: u32,
    /// Ids nunca são reaproveitados, nem depois de `remover`.
    proximo_id: u32,
    tarefas: Vec<Tarefa>,
}

impl Default for ListaDeTarefas {
    fn default() -> Self {
        ListaDeTarefas {
            versao: 1,
            proximo_id: 1,
            tarefas: Vec::new(),
        }
    }
}

impl ListaDeTarefas {
    /// Cria uma tarefa pendente e devolve o id dela.
    ///
    /// `agora` vem de fora (segundos UNIX) para que os testes controlem o relógio.
    pub fn adicionar(&mut self, titulo: String, prioridade: Prioridade, agora: u64) -> u32 {
        // Etapa 3.
        todo!()
    }

    /// Marca a tarefa como feita.
    ///
    /// # Errors
    ///
    /// [`Erro::NaoEncontrada`] se não existe tarefa com esse id.
    pub fn concluir(&mut self, id: u32) -> Result<(), Erro> {
        // Etapa 3.
        todo!()
    }

    /// Remove a tarefa e a devolve.
    ///
    /// # Errors
    ///
    /// [`Erro::NaoEncontrada`] se não existe tarefa com esse id.
    pub fn remover(&mut self, id: u32) -> Result<Tarefa, Erro> {
        // Etapa 3.
        todo!()
    }

    /// As tarefas a mostrar: pendentes primeiro (alta prioridade antes, depois
    /// por id) e, com `todas`, as feitas no fim, na mesma ordem.
    pub fn listar(&self, todas: bool) -> Vec<&Tarefa> {
        // Etapa 3.
        todo!()
    }

    /// `(total, feitas)`.
    pub fn contagem(&self) -> (usize, usize) {
        // Etapa 3.
        todo!()
    }

    /// Todas as tarefas, na ordem em que foram criadas.
    pub fn tarefas(&self) -> &[Tarefa] {
        &self.tarefas
    }
}

#[cfg(test)]
mod step3_tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn ids_nao_sao_reaproveitados() {
        let mut lista = ListaDeTarefas::default();
        assert_eq!(lista.adicionar("a".into(), Prioridade::Media, 0), 1);
        assert_eq!(lista.adicionar("b".into(), Prioridade::Media, 0), 2);
        lista.remover(2).unwrap();
        assert_eq!(lista.adicionar("c".into(), Prioridade::Media, 0), 3);
    }

    #[test]
    fn adicionar_guarda_o_relogio() {
        let mut lista = ListaDeTarefas::default();
        lista.adicionar("a".into(), Prioridade::Alta, 1_792_046_347);
        assert_eq!(lista.tarefas()[0].criada_em, 1_792_046_347);
        assert!(!lista.tarefas()[0].feita);
    }

    #[test]
    fn concluir_e_contagem() {
        let mut lista = ListaDeTarefas::default();
        lista.adicionar("a".into(), Prioridade::Media, 0);
        lista.adicionar("b".into(), Prioridade::Media, 0);
        lista.concluir(1).unwrap();
        assert_eq!(lista.contagem(), (2, 1));
    }

    #[test]
    fn concluir_inexistente() {
        let mut lista = ListaDeTarefas::default();
        assert_matches!(lista.concluir(7), Err(Erro::NaoEncontrada(7)));
    }

    #[test]
    fn remover_devolve_a_tarefa() {
        let mut lista = ListaDeTarefas::default();
        lista.adicionar("pagar conta".into(), Prioridade::Alta, 0);
        assert_eq!(lista.remover(1).unwrap().titulo, "pagar conta");
        assert_matches!(lista.remover(1), Err(Erro::NaoEncontrada(1)));
        assert_eq!(lista.contagem(), (0, 0));
    }

    #[test]
    fn ordem_da_listagem() {
        let mut lista = ListaDeTarefas::default();
        lista.adicionar("feita".into(), Prioridade::Alta, 0);
        lista.adicionar("baixa".into(), Prioridade::Baixa, 0);
        lista.adicionar("alta".into(), Prioridade::Alta, 0);
        lista.concluir(1).unwrap();

        let titulos = |todas| -> Vec<String> {
            lista
                .listar(todas)
                .iter()
                .map(|t| t.titulo.clone())
                .collect()
        };
        assert_eq!(titulos(false), ["alta", "baixa"]);
        assert_eq!(titulos(true), ["alta", "baixa", "feita"]);
    }
}
