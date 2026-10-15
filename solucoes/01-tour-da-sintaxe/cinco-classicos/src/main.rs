fn dobro(x: i32) -> i32 {
    x * 2 // 1. sem `;`: a expressão final é o valor de retorno
}

fn resumo(nome: &str, pontos: i32) -> String {
    format!("{nome} fez {pontos} pontos")
}

fn main() {
    let limite: i32 = "50".parse().unwrap(); // 2. `parse` precisa saber o tipo de destino
    let mut total = 0; // 3. `total` muda dentro do laço
    for parte in "3 10 7".split_whitespace() {
        let n: i32 = parte.parse().unwrap();
        total += dobro(n);
    }
    let vitorias = 2;
    if vitorias > 0 {
        // 4. `if` só aceita `bool`
        println!("{}", resumo("Ana", total)); // 5. não há argumento padrão
    }
    println!("total: {total} (limite: {limite})");
}
