# 04 — E/S assíncrona: um servidor de eco

**Objetivos.** Ao final, você vai saber:

- aceitar conexões com `TcpListener` e criar uma task por cliente;
- ler e escrever com `AsyncReadExt` e `AsyncWriteExt`, e reconhecer o fim da conexão;
- testar um servidor TCP sem fixar porta;
- onde ficam os equivalentes assíncronos de arquivos e outras E/S.

## O servidor completo

Um servidor de eco devolve ao cliente tudo o que recebe. É o "olá, mundo" da programação de redes, e já mostra o padrão de quase todo servidor com tokio: um laço que aceita conexões e uma task por conexão.

```rust
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Devolve ao cliente tudo o que ele mandar, até ele fechar a conexão.
async fn atender(mut socket: TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            return Ok(()); // o cliente fechou
        }
        socket.write_all(&buf[..n]).await?;
    }
}

/// Aceita conexões para sempre, uma task por cliente.
async fn servir(listener: TcpListener) -> io::Result<()> {
    loop {
        let (socket, cliente) = listener.accept().await?;
        println!("conexão de {cliente}");
        tokio::spawn(async move {
            if let Err(e) = atender(socket).await {
                eprintln!("erro com {cliente}: {e}");
            }
            println!("{cliente} saiu");
        });
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    println!("ouvindo em {}", listener.local_addr()?);
    servir(listener).await
}
```

Rodando o servidor e conectando dois clientes, um depois do outro, com `nc` (netcat):

```bash
$ printf 'oi\nsegunda linha\n' | nc 127.0.0.1 7878
oi
segunda linha
$ printf 'outro cliente\n' | nc 127.0.0.1 7878
outro cliente
```

E no terminal do servidor:

```text
ouvindo em 127.0.0.1:7878
conexão de 127.0.0.1:58211
127.0.0.1:58211 saiu
conexão de 127.0.0.1:58212
127.0.0.1:58212 saiu
```

Código completo, com testes, em [`codigo/eco-tcp`](codigo/eco-tcp/src/main.rs).

## Peça por peça

**`TcpListener::bind(...).await`** abre a porta. Até o `bind` é assíncrono, porque resolver um nome pode envolver rede.

**`listener.accept().await`** espera o próximo cliente e devolve o `TcpStream` dele e o endereço. Enquanto ninguém conecta, a task do laço fica parada nesse `.await`, sem ocupar thread.

**`tokio::spawn` por conexão** é o que permite atender vários clientes ao mesmo tempo. Se o laço chamasse `atender(socket).await` diretamente, o segundo cliente só seria aceito depois que o primeiro saísse. O `async move` leva o `socket` e o `cliente` para dentro da task (`'static`, lição 02).

**`read` devolve quantos bytes leu**, e **`0` significa que o outro lado fechou** a conexão. Esquecer esse teste é um clássico: o laço vira um laço infinito lendo zero bytes.

**`write_all`** escreve o buffer inteiro, chamando `write` quantas vezes forem necessárias (um `write` simples pode escrever só uma parte).

**Erros por conexão ficam na task.** Um cliente que some no meio da conversa gera um `Err` de E/S naquela task e só nela. O `?` dentro de `atender` sobe o erro até o `if let Err` da task, que registra e segue. O servidor continua aceitando os outros.

### `AsyncReadExt` e `AsyncWriteExt`

`read` e `write_all` não são métodos de `TcpStream`: vêm das *extension traits* `tokio::io::AsyncReadExt` e `AsyncWriteExt` (por isso o `use`). A ideia é a mesma de `std::io::Read`/`Write`, com `.await`:

| std (bloqueia) | tokio (assíncrono) |
|---|---|
| `std::io::Read::read` | `AsyncReadExt::read(...).await` |
| `read_exact`, `read_to_string` | idem, com `.await` |
| `std::io::Write::write_all` | `AsyncWriteExt::write_all(...).await` |
| `std::io::BufReader` + `lines()` | `tokio::io::BufReader` + `AsyncBufReadExt::lines()` |
| `std::io::copy` | `tokio::io::copy(...).await` |

Para protocolos baseados em linhas, `BufReader::new(socket).lines()` e `next_line().await` dão uma linha por vez, e `socket.split()` (ou `into_split()`) separa a ponta de leitura da de escrita.

> [!NOTE]
> Por baixo, o tokio usa as APIs de *readiness* do sistema (epoll no Linux, kqueue no macOS, IOCP no Windows, via crate `mio`): o sistema avisa quando o socket tem dados, e só então a task é acordada. É isso que permite uma thread cuidar de milhares de conexões.

## Testando sem fixar porta

Fixar a porta 7878 num teste é pedir conflito (dois testes em paralelo, ou o servidor de verdade já rodando). Faça o `bind` na **porta 0**: o sistema escolhe uma livre, e `local_addr()` diz qual foi.

```rust
async fn subir() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endereco = listener.local_addr().unwrap();
    tokio::spawn(servir(listener));
    endereco
}

async fn eco(endereco: std::net::SocketAddr, msg: &str) -> String {
    let mut c = TcpStream::connect(endereco).await.unwrap();
    c.write_all(msg.as_bytes()).await.unwrap();
    c.shutdown().await.unwrap(); // avisa que não vamos mandar mais nada
    let mut resposta = String::new();
    c.read_to_string(&mut resposta).await.unwrap();
    resposta
}

#[tokio::test]
async fn devolve_o_que_recebe() {
    let endereco = subir().await;
    assert_eq!(eco(endereco, "olá, eco\n").await, "olá, eco\n");
}
```

O `shutdown()` fecha a ponta de escrita do cliente. O servidor lê `0`, sai do laço e fecha a conexão, e só então o `read_to_string` do cliente termina. Sem o `shutdown`, os dois ficariam esperando um pelo outro. O projeto tem também um teste com 50 clientes simultâneos.

## Arquivos e o resto

`tokio::fs` tem as versões assíncronas de `std::fs`: `tokio::fs::read_to_string(caminho).await`, `File::open(...).await`, e assim por diante. Elas existem para não bloquear o runtime, mas a maioria dos sistemas não tem E/S de arquivo assíncrona de verdade: o tokio roda a chamada da std num `spawn_blocking`. Para ler um arquivo de configuração na partida, a versão da std serve. Para ler muitos arquivos com a aplicação já atendendo clientes, use `tokio::fs`.

Outros módulos do tokio que seguem o mesmo molde: `tokio::net::UdpSocket`, `tokio::process::Command` (processos filhos), `tokio::signal::ctrl_c()` (esperar o Ctrl+C) e `tokio::io::stdin()`.

## Comparando com Python

O `asyncio` tem as duas camadas: `asyncio.start_server(callback, host, port)` cria o servidor e chama uma corrotina por cliente, com `StreamReader`/`StreamWriter`, e `loop.create_server` dá acesso de mais baixo nível. O desenho é o mesmo: uma corrotina por conexão, um `await reader.read(n)` que devolve `b""` no fim. A diferença em Rust é que a task por conexão é explícita (`tokio::spawn`), e o compilador confere que ela é `Send + 'static`.

## Armadilhas comuns

- **Não tratar `read` = 0.** O laço nunca termina quando o cliente sai.
- **`atender(...).await` direto no laço do `accept`.** O servidor atende um cliente por vez. Crie uma task por conexão.
- **`write` no lugar de `write_all`.** Parte dos dados pode não ser enviada.
- **`unwrap` no erro de uma conexão.** Um cliente mal-comportado derruba a task (e, se for a do `accept`, o servidor). Trate o erro da conexão dentro da task dela.
- **Esquecer o `use tokio::io::AsyncReadExt`.** O erro é "no method named `read` found": os métodos estão na trait.

## Resumo

- `TcpListener::bind` + `accept().await` em laço + `tokio::spawn` por conexão: o esqueleto de um servidor.
- `read` devolve `0` quando o outro lado fecha; `write_all` escreve tudo.
- `AsyncReadExt`/`AsyncWriteExt` são os `Read`/`Write` assíncronos; `BufReader::lines()` lê linha a linha.
- Em testes, porta `0` e `local_addr()`.
- `tokio::fs` evita bloquear o runtime, mas por baixo usa threads de bloqueio.

## Exercícios

Os exercícios deste módulo não usam rede (o Projeto 5 usa). Veja [`exercicios.md`](exercicios.md).

## Leitura complementar

- [Tutorial do Tokio: I/O](https://tokio.rs/tokio/tutorial/io) e [Framing](https://tokio.rs/tokio/tutorial/framing)
- [Documentação de `tokio::net`](https://docs.rs/tokio/latest/tokio/net/index.html) e de [`tokio::io`](https://docs.rs/tokio/latest/tokio/io/index.html)
- [Async Book: IO and issues with blocking](https://rust-lang.github.io/async-book/part-guide/io.html)

---

[← Lição anterior](03-tempo-e-bloqueio.md) · [Índice do módulo](README.md) · [Próxima lição →](05-canais-e-estado.md)
