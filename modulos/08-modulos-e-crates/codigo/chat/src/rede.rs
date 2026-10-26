//! Transporte: TCP e UDP.

pub mod tcp;
pub mod udp;

/// Porta usada quando o endereço não informa nenhuma.
pub const PORTA_PADRAO: u16 = 7000;

/// Privada de `rede`, mas visível nos filhos (`tcp` e `udp`).
fn host_valido(host: &str) -> bool {
    !host.is_empty()
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
}
