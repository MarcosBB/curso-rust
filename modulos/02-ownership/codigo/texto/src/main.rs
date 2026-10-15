//! Módulo 02, lições 05 e 06: `String` x `&str` e as três visões de um texto UTF-8.

fn main() {
    string_e_str();
    println!();
    tres_visoes();
    println!();
    fatias_seguras();
}

// ---------- Lição 05 ----------

/// Só lê o texto: recebe `&str`, que aceita literal, `&String` e fatias.
fn contar_vogais(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

/// Cria um texto novo: devolve `String`, porque quem chamou vai ser o dono.
fn saudacao(nome: &str) -> String {
    format!("Olá, {nome}!")
}

fn string_e_str() {
    println!("== String x &str ==");
    let literal: &str = "banana"; // fica dentro do executável
    let nome = String::from("Ferris"); // fica no heap; `nome` é a dona

    println!("{}", contar_vogais(literal));
    println!("{}", contar_vogais(&nome)); // &String vira &str sozinho
    println!("{}", contar_vogais(&nome[0..3])); // uma fatia de "Ferris"

    let s = saudacao(&nome);
    println!("{s} ({nome} continua válido)");

    // Concatenar: format! não move nada; `+` consome o lado esquerdo.
    let a = String::from("Olá, ");
    let b = String::from("mundo");
    let c = format!("{a}{b}!");
    println!("{c} | a = {a:?} | b = {b:?}");
    let d = a + &b; // `a` foi movida para dentro de `d`
    println!("{d}");

    let mut t = String::from("ab");
    t.push_str("cd");
    t.push('!');
    println!("{t}");
}

// ---------- Lição 06 ----------

fn tres_visoes() {
    println!("== bytes, chars e grafemas ==");
    for s in ["olá", "é", "e\u{301}", "👍🏽"] {
        println!(
            "{s:?}: {} bytes, {} chars, bytes = {:?}, chars = {:?}",
            s.len(),
            s.chars().count(),
            s.as_bytes(),
            s.chars().collect::<Vec<char>>()
        );
    }
    println!("\"é\" == \"e\\u{{301}}\"? {}", "é" == "e\u{301}");
}

fn fatias_seguras() {
    println!("== índices e fatias ==");
    let s = "olá";
    let indices: Vec<(usize, char)> = s.char_indices().collect();
    println!("char_indices = {indices:?}");
    println!("chars().nth(2) = {:?}", s.chars().nth(2));
    println!("as_bytes()[2] = {}", s.as_bytes()[2]);
    println!("get(0..3) = {:?}", s.get(0..3));
    println!("get(0..4) = {:?}", s.get(0..4));
    println!("is_char_boundary(3) = {}", s.is_char_boundary(3));
    println!("find('á') = {:?} (índice em bytes)", s.find('á'));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contar_vogais_aceita_literal_string_e_fatia() {
        let nome = String::from("Ferris");
        assert_eq!(contar_vogais("banana"), 3);
        assert_eq!(contar_vogais(&nome), 2);
        assert_eq!(contar_vogais(&nome[0..3]), 1);
    }

    #[test]
    fn saudacao_devolve_string_nova() {
        assert_eq!(saudacao("Ana"), "Olá, Ana!");
    }

    #[test]
    fn len_conta_bytes() {
        assert_eq!("olá".len(), 4);
        assert_eq!("olá".chars().count(), 3);
        assert_eq!("👍🏽".chars().count(), 2);
    }
}
