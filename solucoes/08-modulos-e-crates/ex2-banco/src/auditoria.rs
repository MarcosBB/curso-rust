//! Relatórios sobre as contas.

use crate::conta::Conta;

/// O extrato: o titular, cada lançamento e o saldo.
pub fn extrato(conta: &Conta) -> String {
    let mut s = format!("{}\n", conta.titular());
    for v in &conta.historico {
        s.push_str(&format!("{v:+}\n"));
    }
    s.push_str(&format!("saldo: {}\n", conta.saldo()));
    s
}
