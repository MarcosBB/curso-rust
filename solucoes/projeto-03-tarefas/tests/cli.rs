//! Testes do executável. Cada teste cria a própria pasta temporária
//! (`tempfile::tempdir()`, apagada no fim) e passa o arquivo por `--arquivo`
//! ou por `Command::env`. Nenhum teste toca no `~/.tarefas.json` de verdade.

// O `allow-unwrap-in-tests` do clippy.toml só cobre funções `#[test]` e
// código em `#[cfg(test)]`. As funções auxiliares abaixo não são nem uma
// coisa nem outra, então liberamos o `unwrap` no arquivo todo.
#![allow(clippy::unwrap_used, reason = "arquivo de testes")]

use std::path::Path;
use std::process::{Command, Output};

fn tarefas(arquivo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tarefas"))
        .arg("--arquivo")
        .arg(arquivo)
        .args(args)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> &str {
    std::str::from_utf8(&out.stdout).unwrap()
}

fn stderr(out: &Output) -> &str {
    std::str::from_utf8(&out.stderr).unwrap()
}

#[test]
fn fluxo_completo() {
    let dir = tempfile::tempdir().unwrap();
    let arq = dir.path().join("t.json");
    assert_eq!(
        stdout(&tarefas(&arq, &["add", "comprar", "pão"])),
        "tarefa 1 criada\n"
    );
    tarefas(&arq, &["add", "pagar", "conta", "-p", "alta"]);
    let out = tarefas(&arq, &["done", "1"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "");
    assert_eq!(
        stdout(&tarefas(&arq, &["list"])),
        "[ ]   2  alta   pagar conta\n"
    );
    assert_eq!(
        stdout(&tarefas(&arq, &["list", "--todas"])),
        "[ ]   2  alta   pagar conta\n[x]   1  media  comprar pão\n"
    );
    assert_eq!(
        stdout(&tarefas(&arq, &["stats"])),
        "2 tarefas, 1 feitas, 1 pendentes\n"
    );
    assert_eq!(
        stdout(&tarefas(&arq, &["rm", "1"])),
        "removida: comprar pão\n"
    );
    assert!(!dir.path().join("t.json.tmp").exists());
}

#[test]
fn id_inexistente_sai_com_1() {
    let dir = tempfile::tempdir().unwrap();
    let out = tarefas(&dir.path().join("t.json"), &["done", "7"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stderr(&out), "tarefas: não existe tarefa com id 7\n");
}

#[test]
fn json_corrompido_mostra_a_causa() {
    let dir = tempfile::tempdir().unwrap();
    let arq = dir.path().join("t.json");
    std::fs::write(&arq, "isto não é JSON").unwrap();
    let out = tarefas(&arq, &["list"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out)
            .ends_with("não é um arquivo de tarefas válido: expected value at line 1 column 1\n"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn uso_errado_sai_com_2() {
    let dir = tempfile::tempdir().unwrap();
    let out = tarefas(&dir.path().join("t.json"), &["add"]);
    assert_eq!(out.status.code(), Some(2));
    assert_eq!(
        stderr(&out),
        "tarefas: faltou o título\nuso: tarefas [--arquivo CAMINHO] <add TÍTULO... [-p PRIORIDADE] | list [--todas] | done ID | rm ID | stats>\n"
    );
    // Erro de uso não cria o arquivo.
    assert!(!dir.path().join("t.json").exists());
}

#[test]
fn variavel_de_ambiente_escolhe_o_arquivo() {
    let dir = tempfile::tempdir().unwrap();
    let arq = dir.path().join("env.json");
    let out = Command::new(env!("CARGO_BIN_EXE_tarefas"))
        .args(["add", "x"])
        .env("TAREFAS_ARQUIVO", &arq)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(arq.exists());
}

#[test]
fn sem_arquivo_usa_a_pasta_pessoal() {
    let dir = tempfile::tempdir().unwrap();
    // `HOME` aponta para a pasta temporária, só neste processo filho.
    let out = Command::new(env!("CARGO_BIN_EXE_tarefas"))
        .args(["add", "x"])
        .env("HOME", dir.path())
        .env_remove("TAREFAS_ARQUIVO")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(dir.path().join(".tarefas.json").exists());
}
