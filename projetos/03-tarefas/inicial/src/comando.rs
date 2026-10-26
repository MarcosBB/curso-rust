use crate::Erro;
use crate::tarefa::Prioridade;

/// A linha de uso, mostrada junto de todo erro de uso.
pub const USO: &str = "uso: tarefas [--arquivo CAMINHO] <add TÍTULO... [-p PRIORIDADE] | list [--todas] | done ID | rm ID | stats>";

/// O que o usuário pediu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comando {
    Adicionar {
        titulo: String,
        prioridade: Prioridade,
    },
    Listar {
        todas: bool,
    },
    Concluir(u32),
    Remover(u32),
    Estatisticas,
}

fn id(texto: &str) -> Result<u32, Erro> {
    // Etapa 5: `parse` + `map_err` para `Erro::Uso("id inválido: ...")`.
    todo!()
}

impl Comando {
    /// Interpreta os argumentos (sem o nome do programa e sem `--arquivo`).
    ///
    /// # Errors
    ///
    /// [`Erro::Uso`] quando os argumentos não formam um comando válido.
    pub fn de_args(args: &[&str]) -> Result<Comando, Erro> {
        // Etapa 5: `match args { ["add", resto @ ..] => ..., ["list"] => ..., ... }`
        todo!()
    }
}

#[cfg(test)]
mod step5_tests {
    use super::*;
    use rstest::rstest;
    use std::assert_matches;

    fn adicionar(titulo: &str, prioridade: Prioridade) -> Comando {
        Comando::Adicionar {
            titulo: titulo.into(),
            prioridade,
        }
    }

    #[rstest]
    #[case::add(&["add", "comprar", "pão"], adicionar("comprar pão", Prioridade::Media))]
    #[case::add_com_prioridade(&["add", "pagar", "-p", "alta"], adicionar("pagar", Prioridade::Alta))]
    #[case::list(&["list"], Comando::Listar { todas: false })]
    #[case::list_todas(&["list", "--todas"], Comando::Listar { todas: true })]
    #[case::done(&["done", "3"], Comando::Concluir(3))]
    #[case::rm(&["rm", "12"], Comando::Remover(12))]
    #[case::stats(&["stats"], Comando::Estatisticas)]
    fn validos(#[case] args: &[&str], #[case] esperado: Comando) {
        assert_eq!(Comando::de_args(args).unwrap(), esperado);
    }

    #[rstest]
    #[case::vazio(&[])]
    #[case::add_sem_titulo(&["add"])]
    #[case::id_nao_numerico(&["done", "abc"])]
    #[case::id_negativo(&["rm", "-1"])]
    #[case::desconhecido(&["voar"])]
    #[case::list_com_lixo(&["list", "--tudo"])]
    #[case::prioridade_invalida(&["add", "x", "-p", "urgente"])]
    fn invalidos(#[case] args: &[&str]) {
        assert_matches!(Comando::de_args(args), Err(Erro::Uso(_)));
    }

    #[test]
    fn mensagens() {
        let msg = |args: &[&str]| Comando::de_args(args).unwrap_err().to_string();
        assert_eq!(msg(&["add"]), "faltou o título");
        assert_eq!(
            msg(&["add", "x", "-p", "urgente"]),
            "prioridade inválida: urgente (use alta, media ou baixa)"
        );
        assert_eq!(msg(&["done", "abc"]), "id inválido: abc");
    }
}
