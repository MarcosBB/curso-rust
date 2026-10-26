//! A conta e as operações sobre ela.

use crate::taxas;

/// Uma conta bancária. O saldo só muda por [`Conta::depositar`] e [`Conta::sacar`].
#[derive(Debug)]
pub struct Conta {
    titular: String,
    saldo: i64,
    pub(crate) historico: Vec<i64>,
}

impl Conta {
    /// Abre uma conta com saldo zero.
    pub fn abrir(titular: &str) -> Conta {
        Conta {
            titular: titular.to_string(),
            saldo: 0,
            historico: Vec::new(),
        }
    }

    /// O nome do titular.
    pub fn titular(&self) -> &str {
        &self.titular
    }

    /// O saldo atual, em centavos.
    pub fn saldo(&self) -> i64 {
        self.saldo
    }

    /// Deposita `valor` centavos.
    pub fn depositar(&mut self, valor: i64) {
        self.registrar(valor);
    }

    /// Saca `valor` centavos, mais a tarifa. Falha se o saldo não cobrir os dois.
    pub fn sacar(&mut self, valor: i64) -> Result<(), String> {
        let total = valor + taxas::tarifa_saque(valor);
        if total > self.saldo {
            return Err(format!("saldo insuficiente: precisa de {total}"));
        }
        self.registrar(-total);
        Ok(())
    }

    fn registrar(&mut self, valor: i64) {
        self.saldo += valor;
        self.historico.push(valor);
    }
}
