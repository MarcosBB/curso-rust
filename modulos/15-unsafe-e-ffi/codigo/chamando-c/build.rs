// Compila csrc/estatisticas.c numa biblioteca estática e pede ao Cargo para
// ligá-la ao crate. O `cc` encontra o compilador C do sistema (clang, gcc ou
// MSVC) e repassa as flags certas para o alvo.
fn main() {
    cc::Build::new()
        .file("csrc/estatisticas.c")
        .warnings(true)
        .compile("estatisticas");
    // Recompila só se o arquivo C mudar.
    println!("cargo::rerun-if-changed=csrc/estatisticas.c");
}
