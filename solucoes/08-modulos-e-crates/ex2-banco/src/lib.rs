//! Exercício 2: visibilidade mínima.
//!
//! A API pública é só `conta::Conta` (com `abrir`, `titular`, `saldo`,
//! `depositar` e `sacar`) e `auditoria::extrato`. Os doctests abaixo
//! conferem que o resto ficou inacessível de fora da crate.
//!
//! O saldo não pode ser alterado sem passar pelos métodos:
//!
//! ```compile_fail
//! let mut c = banco::conta::Conta::abrir("Ana");
//! c.saldo = 1_000_000;
//! ```
//!
//! O histórico é interno (só a auditoria lê):
//!
//! ```compile_fail
//! let c = banco::conta::Conta::abrir("Ana");
//! println!("{:?}", c.historico);
//! ```
//!
//! O módulo de taxas não faz parte da API:
//!
//! ```compile_fail
//! let t = banco::taxas::tarifa_saque(100);
//! ```
//!
//! Nem o método que registra lançamentos:
//!
//! ```compile_fail
//! let mut c = banco::conta::Conta::abrir("Ana");
//! c.registrar(50);
//! ```

pub mod auditoria;
pub mod conta;
mod taxas;
