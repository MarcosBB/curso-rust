//! Módulo 02: o que uma `String` guarda na stack e no heap,
//! e o que acontece com esses dados num move e num clone.

fn main() {
    layout();
    move_nao_copia_o_heap();
    clone_copia_o_heap();
    mover_para_funcao_e_de_volta();
}

/// Lição 01: tamanho na stack x conteúdo no heap.
fn layout() {
    println!("== layout ==");
    println!("size_of::<i32>()     = {}", size_of::<i32>());
    println!("size_of::<String>()  = {}", size_of::<String>());
    println!("size_of::<Vec<u8>>() = {}", size_of::<Vec<u8>>());
    println!("size_of::<&str>()    = {}", size_of::<&str>());

    let mut s = String::with_capacity(5);
    s.push_str("olá");
    println!("s = {s:?}");
    println!("len = {}, capacity = {}", s.len(), s.capacity());
    println!("bytes no heap = {:x?}", s.as_bytes());

    // `size_of` mede só a parte da stack: não importa o tamanho do texto.
    let grande = "x".repeat(1_000);
    println!(
        "texto de {} bytes, size_of_val = {}",
        grande.len(),
        size_of_val(&grande)
    );
}

/// Lição 02: o move copia ptr/len/cap; o buffer no heap continua o mesmo.
fn move_nao_copia_o_heap() {
    println!("\n== move ==");
    let s1 = String::from("oi");
    let antes = s1.as_ptr();
    let s2 = s1; // a partir daqui, `s1` não pode mais ser usada
    let depois = s2.as_ptr();
    println!("buffer antes do move:  {antes:p}");
    println!("buffer depois do move: {depois:p}");
    println!("mesmo buffer? {}", antes == depois);
}

/// Lição 03: `clone()` aloca um buffer novo e copia os bytes.
fn clone_copia_o_heap() {
    println!("\n== clone ==");
    let s1 = String::from("oi");
    let s2 = s1.clone();
    println!("buffer de s1: {:p}", s1.as_ptr());
    println!("buffer de s2: {:p}", s2.as_ptr());
    println!("mesmo buffer? {}", s1.as_ptr() == s2.as_ptr());
    println!("s1 = {s1}, s2 = {s2} (as duas continuam válidas)");
}

/// Lição 02: passar por valor move para a função; retornar devolve a posse.
fn mover_para_funcao_e_de_volta() {
    println!("\n== função ==");
    let s = cria();
    let (s, n) = tamanho(s);
    println!("{s} tem {n} bytes");
    consome(s);
    // `s` não pode mais ser usada aqui: a posse foi para `consome`.
}

fn cria() -> String {
    String::from("Ferris")
}

fn tamanho(s: String) -> (String, usize) {
    let n = s.len();
    (s, n) // devolve a posse junto com o resultado
}

fn consome(s: String) {
    println!("consome recebeu {s}; o buffer é liberado no fim desta função");
}
