use chat::rede::udp;
use chat::{Conexao, cliente};

fn main() {
    let (conexao, msg): (Conexao, String) = match cliente::iniciar("chat.exemplo.com") {
        Ok(par) => par,
        Err(e) => {
            eprintln!("erro: {e}");
            return;
        }
    };
    println!("{msg}");
    println!("endereço: {}", conexao.endereco());

    match udp::enviar("chat.exemplo.com", b"oi") {
        Ok(n) => println!("udp: {n} bytes"),
        Err(e) => eprintln!("erro: {e}"),
    }
}
