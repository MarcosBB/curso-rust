//! O estado que todas as conexões dividem: quem está na sala e o canal
//! `broadcast` por onde passam os eventos.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::broadcast;

use crate::protocolo::{self, ErroApelido};

/// Uma linha que vai para todos da sala, menos para o `autor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evento {
    /// Id da conexão que gerou o evento. Ela não recebe o próprio evento.
    pub autor: u64,
    /// A linha pronta, sem `\n`.
    pub texto: String,
}

/// Quem acabou de entrar: o id, o apelido provisório e o seu receptor.
#[derive(Debug)]
pub struct Membro {
    pub id: u64,
    pub apelido: String,
    pub rx: broadcast::Receiver<Evento>,
}

/// A sala de chat. Vive num `Arc` e é dividida por todas as tasks.
///
/// Os métodos são síncronos de propósito: o `Mutex` (o da std) é travado e
/// solto dentro de cada um, e nunca fica preso através de um `.await`.
#[derive(Debug)]
pub struct Sala {
    tx: broadcast::Sender<Evento>,
    apelidos: Mutex<HashSet<String>>,
    proximo_id: AtomicU64,
}

impl Sala {
    /// Uma sala vazia. `capacidade` é quantos eventos o canal guarda para
    /// quem ainda não leu; quem ficar mais atrasado que isso perde os mais
    /// antigos (`RecvError::Lagged`).
    pub fn new(capacidade: usize) -> Sala {
        // Etapa 2: `broadcast::channel(capacidade)` devolve `(Sender, Receiver)`.
        // Guarde o `Sender`; o `Receiver` pode ser descartado (cada membro ganha
        // o seu com `subscribe`).
        todo!()
    }

    /// Registra uma conexão nova: um id único, um apelido livre do tipo
    /// `convidado-N` e um receptor inscrito no canal a partir de agora.
    pub fn entrar(&self) -> Membro {
        // Etapa 2: `fetch_add` no contador, trave `apelidos`, procure o primeiro
        // `convidado-N` livre a partir de N = id, insira e `self.tx.subscribe()`.
        todo!()
    }

    /// Troca o apelido `atual` por `novo`, se `novo` for válido e estiver
    /// livre. Trocar pelo mesmo nome dá certo e não muda nada.
    pub fn renomear(&self, atual: &str, novo: &str) -> Result<(), ErroApelido> {
        // Etapa 2: `protocolo::validar_apelido(novo)?` primeiro; depois, com o
        // `Mutex` travado, `EmUso` se já existe, senão troque um pelo outro.
        todo!()
    }

    /// Libera o apelido de quem saiu.
    pub fn sair(&self, apelido: &str) {
        // Etapa 2.
        todo!()
    }

    /// Os apelidos de quem está na sala, em ordem alfabética.
    pub fn quem(&self) -> Vec<String> {
        // Etapa 2: copie para um `Vec`, ordene, e só então devolva
        // (o guarda morre no fim da função, nunca vai para fora).
        todo!()
    }

    /// Manda `texto` para todos os membros, menos o `autor`.
    pub fn anunciar(&self, autor: u64, texto: String) {
        // Etapa 2: `self.tx.send(..)`. Ele devolve `Err` quando não há nenhum
        // receptor; decida o que fazer com isso (e escreva um comentário).
        todo!()
    }
}

#[cfg(test)]
mod step2_tests {
    use super::*;
    use tokio::sync::broadcast::error::TryRecvError;

    #[test]
    fn quem_entra_ganha_id_e_apelido_unicos() {
        let sala = Sala::new(16);
        let a = sala.entrar();
        let b = sala.entrar();
        assert_ne!(a.id, b.id);
        assert_eq!(a.apelido, "convidado-1");
        assert_eq!(b.apelido, "convidado-2");
        assert_eq!(sala.quem(), ["convidado-1", "convidado-2"]);
    }

    #[test]
    fn apelido_provisorio_pula_os_que_estao_em_uso() {
        let sala = Sala::new(16);
        let a = sala.entrar();
        sala.renomear(&a.apelido, "convidado-2").unwrap();
        let b = sala.entrar();
        assert_eq!(b.apelido, "convidado-3");
    }

    #[test]
    fn renomear_valida_e_recusa_repetidos() {
        let sala = Sala::new(16);
        let a = sala.entrar();
        let b = sala.entrar();
        assert_eq!(sala.renomear(&a.apelido, "ana"), Ok(()));
        assert_eq!(
            sala.renomear(&b.apelido, "ana"),
            Err(ErroApelido::EmUso("ana".into()))
        );
        assert_eq!(sala.renomear("ana", "ana"), Ok(()));
        assert_eq!(sala.renomear("ana", ""), Err(ErroApelido::Vazio));
        assert_eq!(sala.quem(), ["ana", "convidado-2"]);
    }

    #[test]
    fn sair_libera_o_apelido() {
        let sala = Sala::new(16);
        let a = sala.entrar();
        sala.renomear(&a.apelido, "ana").unwrap();
        sala.sair("ana");
        assert!(sala.quem().is_empty());
        let b = sala.entrar();
        assert_eq!(sala.renomear(&b.apelido, "ana"), Ok(()));
    }

    #[test]
    fn anunciar_chega_a_todos_os_inscritos() {
        let sala = Sala::new(16);
        let mut a = sala.entrar();
        let mut b = sala.entrar();
        sala.anunciar(a.id, "oi".into());
        let esperado = Evento {
            autor: a.id,
            texto: "oi".into(),
        };
        // O canal entrega para todos; quem filtra o próprio evento é a conexão.
        assert_eq!(a.rx.try_recv(), Ok(esperado.clone()));
        assert_eq!(b.rx.try_recv(), Ok(esperado));
        assert_eq!(b.rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn quem_entra_depois_nao_ve_o_passado() {
        let sala = Sala::new(16);
        let _a = sala.entrar();
        sala.anunciar(0, "antes".into());
        let mut b = sala.entrar();
        assert_eq!(b.rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn anunciar_numa_sala_vazia_nao_da_panic() {
        let sala = Sala::new(16);
        sala.anunciar(0, "ninguém ouve".into());
    }
}
