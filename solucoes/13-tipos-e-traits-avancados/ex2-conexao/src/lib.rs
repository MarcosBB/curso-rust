//! Conexão com typestate: cada estado é um tipo.
//!
//! Enviar sem estar conectado não compila:
//!
//! ```compile_fail,E0599
//! use ex2_conexao::Conexao;
//! let mut c = Conexao::new("10.0.0.1:80");
//! c.enviar("oi");
//! ```
//!
//! Nem confirmar uma conexão que já foi confirmada (a transição consome o valor):
//!
//! ```compile_fail,E0382
//! use ex2_conexao::Conexao;
//! let tentando = Conexao::new("10.0.0.1:80").conectar();
//! let a = tentando.confirmar();
//! let b = tentando.confirmar();
//! ```
//!
//! Nem desconectar quem ainda está tentando conectar:
//!
//! ```compile_fail,E0599
//! use ex2_conexao::Conexao;
//! let c = Conexao::new("10.0.0.1:80").conectar().desconectar();
//! ```

use std::marker::PhantomData;

#[derive(Debug)]
pub struct Desconectada;
#[derive(Debug)]
pub struct Conectando;
#[derive(Debug)]
pub struct Conectada;

pub trait Estado {
    const NOME: &'static str;
}

impl Estado for Desconectada {
    const NOME: &'static str = "desconectada";
}
impl Estado for Conectando {
    const NOME: &'static str = "conectando";
}
impl Estado for Conectada {
    const NOME: &'static str = "conectada";
}

#[derive(Debug)]
pub struct Conexao<E> {
    endereco: String,
    tentativas: u32,
    enviadas: Vec<String>,
    _estado: PhantomData<E>,
}

impl<E> Conexao<E> {
    /// Muda só o estado, mantendo os dados.
    fn para<F>(self) -> Conexao<F> {
        Conexao {
            endereco: self.endereco,
            tentativas: self.tentativas,
            enviadas: self.enviadas,
            _estado: PhantomData,
        }
    }
}

impl<E: Estado> Conexao<E> {
    pub fn estado(&self) -> &'static str {
        E::NOME
    }

    pub fn endereco(&self) -> &str {
        &self.endereco
    }

    pub fn tentativas(&self) -> u32 {
        self.tentativas
    }
}

impl Conexao<Desconectada> {
    pub fn new(endereco: impl Into<String>) -> Self {
        Conexao {
            endereco: endereco.into(),
            tentativas: 0,
            enviadas: Vec::new(),
            _estado: PhantomData,
        }
    }

    pub fn conectar(mut self) -> Conexao<Conectando> {
        self.tentativas += 1;
        self.para()
    }
}

impl Conexao<Conectando> {
    pub fn confirmar(self) -> Conexao<Conectada> {
        self.para()
    }

    pub fn falhar(self) -> Conexao<Desconectada> {
        self.para()
    }
}

impl Conexao<Conectada> {
    pub fn enviar(&mut self, msg: &str) {
        self.enviadas.push(msg.to_string());
    }

    pub fn enviadas(&self) -> &[String] {
        &self.enviadas
    }

    pub fn desconectar(self) -> Conexao<Desconectada> {
        self.para()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluxo_feliz() {
        let c = Conexao::new("10.0.0.1:80");
        assert_eq!(c.estado(), "desconectada");
        let c = c.conectar();
        assert_eq!(c.estado(), "conectando");
        let mut c = c.confirmar();
        assert_eq!(c.estado(), "conectada");
        c.enviar("oi");
        c.enviar("tchau");
        assert_eq!(c.enviadas(), ["oi", "tchau"]);
        let c = c.desconectar();
        assert_eq!(c.estado(), "desconectada");
        assert_eq!(c.endereco(), "10.0.0.1:80");
    }

    #[test]
    fn falha_volta_a_desconectada_e_conta_tentativas() {
        let c = Conexao::new(String::from("host:1")).conectar().falhar();
        assert_eq!(c.estado(), "desconectada");
        let c = c.conectar().falhar().conectar();
        assert_eq!(c.tentativas(), 3);
        let c = c.confirmar();
        assert_eq!(c.estado(), "conectada");
    }

    #[test]
    fn mensagens_sobrevivem_a_reconexao() {
        let mut c = Conexao::new("h").conectar().confirmar();
        c.enviar("a");
        let mut c = c.desconectar().conectar().confirmar();
        c.enviar("b");
        assert_eq!(c.enviadas(), ["a", "b"]);
    }
}
