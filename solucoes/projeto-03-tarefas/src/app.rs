use std::io::Write;

use crate::Erro;
use crate::armazenamento::Armazenamento;
use crate::comando::Comando;

/// Executa um comando: carrega a lista, aplica o comando, grava (se mudou) e
/// escreve o resultado em `saida`.
///
/// `armazenamento`, `saida` e `agora` vêm de fora: o `main` passa o arquivo,
/// o stdout e o relógio de verdade; os testes passam uma
/// [`Memoria`](crate::armazenamento::Memoria), um `Vec<u8>` e um número fixo.
///
/// # Errors
///
/// Os erros de [`Armazenamento`], [`Erro::NaoEncontrada`] e [`Erro::Saida`].
pub fn executar(
    cmd: Comando,
    armazenamento: &mut impl Armazenamento,
    saida: &mut impl Write,
    agora: u64,
) -> Result<(), Erro> {
    let mut lista = armazenamento.carregar()?;
    match cmd {
        Comando::Adicionar { titulo, prioridade } => {
            let id = lista.adicionar(titulo, prioridade, agora);
            // Grava antes de anunciar: se a gravação falhar, nada foi "criado".
            armazenamento.salvar(&lista)?;
            writeln!(saida, "tarefa {id} criada")?;
        }
        Comando::Listar { todas } => {
            for tarefa in lista.listar(todas) {
                writeln!(saida, "{tarefa}")?;
            }
        }
        Comando::Concluir(id) => {
            lista.concluir(id)?;
            armazenamento.salvar(&lista)?;
        }
        Comando::Remover(id) => {
            let tarefa = lista.remover(id)?;
            armazenamento.salvar(&lista)?;
            writeln!(saida, "removida: {}", tarefa.titulo)?;
        }
        Comando::Estatisticas => {
            let (total, feitas) = lista.contagem();
            writeln!(
                saida,
                "{total} tarefas, {feitas} feitas, {} pendentes",
                total - feitas
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod step6_tests {
    use super::*;
    use crate::armazenamento::Memoria;
    use crate::tarefa::Prioridade;
    use std::assert_matches;

    /// Roda um comando sobre a memória e devolve o que foi escrito.
    fn rodar(memoria: &mut Memoria, cmd: Comando) -> Result<String, Erro> {
        let mut saida = Vec::new();
        executar(cmd, memoria, &mut saida, 100)?;
        Ok(String::from_utf8(saida).unwrap())
    }

    fn adicionar(titulo: &str) -> Comando {
        Comando::Adicionar {
            titulo: titulo.into(),
            prioridade: Prioridade::Media,
        }
    }

    #[test]
    fn adicionar_grava_e_imprime() {
        let mut memoria = Memoria::default();
        assert_eq!(
            rodar(&mut memoria, adicionar("a")).unwrap(),
            "tarefa 1 criada\n"
        );
        assert_eq!(memoria.gravacoes, 1);
        assert_eq!(memoria.lista.tarefas()[0].criada_em, 100);
    }

    #[test]
    fn concluir_grava_uma_vez() {
        let mut memoria = Memoria::default();
        rodar(&mut memoria, adicionar("a")).unwrap();
        assert_eq!(rodar(&mut memoria, Comando::Concluir(1)).unwrap(), "");
        assert_eq!(memoria.gravacoes, 2);
        assert!(memoria.lista.tarefas()[0].feita);
    }

    #[test]
    fn listar_nao_grava() {
        let mut memoria = Memoria::default();
        rodar(&mut memoria, adicionar("a")).unwrap();
        let saida = rodar(&mut memoria, Comando::Listar { todas: false }).unwrap();
        assert_eq!(saida, "[ ]   1  media  a\n");
        assert_eq!(memoria.gravacoes, 1);
    }

    #[test]
    fn erro_nao_grava() {
        let mut memoria = Memoria::default();
        assert_matches!(
            rodar(&mut memoria, Comando::Remover(7)),
            Err(Erro::NaoEncontrada(7))
        );
        assert_eq!(memoria.gravacoes, 0);
    }

    #[test]
    fn remover_e_estatisticas() {
        let mut memoria = Memoria::default();
        rodar(&mut memoria, adicionar("a")).unwrap();
        rodar(&mut memoria, adicionar("b")).unwrap();
        rodar(&mut memoria, Comando::Concluir(2)).unwrap();
        assert_eq!(
            rodar(&mut memoria, Comando::Estatisticas).unwrap(),
            "2 tarefas, 1 feitas, 1 pendentes\n"
        );
        assert_eq!(
            rodar(&mut memoria, Comando::Remover(1)).unwrap(),
            "removida: a\n"
        );
    }

    // Escrito pelo aluno na etapa 6: `stats` também não grava.
    #[test]
    fn estatisticas_nao_grava() {
        let mut memoria = Memoria::default();
        rodar(&mut memoria, Comando::Estatisticas).unwrap();
        assert_eq!(memoria.gravacoes, 0);
    }
}
