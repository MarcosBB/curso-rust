//! `tarefas`: uma lista de tarefas na linha de comando, gravada em JSON.
//!
//! A biblioteca não toca no mundo: não lê o relógio, não escolhe o arquivo e
//! não imprime direto no stdout. Quem faz isso é o `main.rs`. Tudo aqui
//! recebe o que precisa por parâmetro e pode ser testado sem rodar o programa.

// Só no projeto inicial: enquanto os corpos forem `todo!()`, parâmetros e
// imports aparecem como "não usados". Apague esta linha na etapa 6.
#![allow(unused)]

pub mod app;
pub mod armazenamento;
pub mod comando;
mod erro;
pub mod lista;
pub mod tarefa;

pub use erro::Erro;
