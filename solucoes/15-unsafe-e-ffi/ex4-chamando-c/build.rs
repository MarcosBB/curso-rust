fn main() {
    cc::Build::new().file("csrc/texto.c").compile("texto");
    println!("cargo::rerun-if-changed=csrc/texto.c");
}
