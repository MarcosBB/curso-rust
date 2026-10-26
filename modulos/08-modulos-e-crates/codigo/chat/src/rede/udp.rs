//! Datagramas UDP (simulados).

/// "Envia" um datagrama e devolve quantos bytes sairiam.
pub fn enviar(host: &str, dados: &[u8]) -> Result<usize, String> {
    if !super::host_valido(host) {
        return Err(format!("host inválido: {host:?}"));
    }
    Ok(dados.len())
}
