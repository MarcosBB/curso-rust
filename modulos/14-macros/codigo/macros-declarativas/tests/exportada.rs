//! Um teste de integração é outra crate: aqui só vale o que foi exportado.

use macros_declarativas::registrar;

#[test]
fn registrar_funciona_fora_da_crate() {
    let total = 3;
    assert_eq!(registrar!(info, "{total} arquivos"), "[info] 3 arquivos");
}
