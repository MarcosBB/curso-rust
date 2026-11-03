//! Ponteiros crus: criar, desreferenciar, aritmética, `Box::into_raw`,
//! `static mut` com `&raw` e campos de struct `packed`.
//! Lições 01 e 02 do Módulo 15. Rode também com `cargo +nightly miri run`.

use std::ptr;

fn criar_e_ler() {
    let mut x = 10;
    // Criar ponteiros crus é seguro: nenhuma memória é acessada aqui.
    let p_escrita = &raw mut x;
    // O ponteiro de leitura é derivado do de escrita (veja a lição 04: criar
    // os dois direto de `x` e escrever por um deles é rejeitado pelo Miri).
    let p_leitura = p_escrita as *const i32;
    println!("endereço: {p_leitura:p}");

    // SAFETY: os dois ponteiros vêm de `x`, que está viva até o fim da
    // função, e nenhuma referência para `x` existe enquanto eles são usados.
    unsafe {
        *p_escrita += 1;
        println!("valor: {}", *p_leitura);
    }
}

fn nulo_e_option() {
    let nulo: *const i32 = ptr::null();
    println!("é nulo? {}", nulo.is_null());

    let x = 7;
    let p = &raw const x;
    // `as_ref` devolve `None` para ponteiro nulo e `Some(&T)` caso contrário.
    // SAFETY: se não for nulo, `p` aponta para `x`, viva e inicializada.
    let r = unsafe { p.as_ref() };
    let n = unsafe { nulo.as_ref() };
    println!("as_ref: {r:?} e {n:?}");
}

fn aritmetica() {
    let numeros = [10, 20, 30, 40];
    let inicio = numeros.as_ptr();
    let mut soma = 0;
    for i in 0..numeros.len() {
        // SAFETY: `i < numeros.len()`, então `inicio.add(i)` fica dentro do
        // array, que está vivo e não é alterado durante o laço.
        soma += unsafe { *inicio.add(i) };
    }
    println!("soma: {soma}");
}

fn ler_escrever_trocar() {
    let mut a = String::from("um");
    let mut b = String::from("dois");
    // SAFETY: `&mut a` e `&mut b` são válidos, alinhados e não se sobrepõem.
    unsafe { ptr::swap(&mut a, &mut b) };
    println!("trocados: {a} {b}");
}

fn box_cru() {
    let caixa = Box::new(String::from("no heap"));
    // `into_raw` entrega a posse para um ponteiro cru: ninguém mais vai liberar.
    let p: *mut String = Box::into_raw(caixa);
    // SAFETY: `p` veio de `Box::into_raw` e ainda não foi liberado.
    unsafe { (*p).push_str(", via ponteiro") };
    // SAFETY: `p` veio de `Box::into_raw` e é devolvido uma única vez.
    let de_volta = unsafe { Box::from_raw(p) };
    println!("{de_volta}");
} // `de_volta` é descartada aqui e libera a memória.

static mut CHAMADAS: u32 = 0;

fn contar_chamada() {
    // SAFETY: este programa só tem uma thread, então não há acesso simultâneo.
    // Ler e escrever por valor não cria referência para o `static mut`.
    unsafe { CHAMADAS += 1 };
}

#[repr(C, packed)]
struct Cabecalho {
    tipo: u8,
    tamanho: u32,
}

fn campo_packed() {
    let c = Cabecalho {
        tipo: 1,
        tamanho: 512,
    };
    // `&c.tamanho` seria uma referência desalinhada (erro E0793).
    let p = &raw const c.tamanho;
    // SAFETY: `p` aponta para um campo inicializado de `c`; `read_unaligned`
    // não exige alinhamento.
    let tamanho = unsafe { p.read_unaligned() };
    // Copiar o campo (`{ c.tipo }`) também funciona, porque não cria referência.
    println!(
        "tipo {} tamanho {tamanho} (struct de {} bytes)",
        { c.tipo },
        size_of::<Cabecalho>()
    );
}

fn main() {
    criar_e_ler();
    nulo_e_option();
    aritmetica();
    ler_escrever_trocar();
    box_cru();
    contar_chamada();
    contar_chamada();
    // SAFETY: uma thread só. O bloco `unsafe` devolve uma cópia do valor, e
    // o `println!` empresta a cópia, não o `static mut`.
    println!("chamadas: {}", unsafe { CHAMADAS });
    // Quando é preciso um endereço (para passar a C, por exemplo), use `&raw`.
    let p: *const u32 = &raw const CHAMADAS;
    println!("CHAMADAS mora em {p:p}");
    campo_packed();
}
