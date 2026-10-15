# 01 — Por que Rust

> Última revisão: 2026-10-15

**Objetivos.** Ao final, você vai saber:

- que problema Rust resolve e por que ele existe;
- o que Rust oferece em troca de um compilador mais exigente;
- o que Rust **não** resolve.

## O dilema: controle ou segurança

As linguagens mais usadas costumam cair em um de dois grupos:

- **Controle total** (C, C++): você decide onde cada byte vive e quando é liberado. O resultado é rápido e previsível, mas o compilador aceita código que acessa memória já liberada, lê fora dos limites de um array ou é alterado por duas threads ao mesmo tempo.
- **Segurança total** (Python, Java, Go, JavaScript): um *garbage collector* (GC) cuida da memória, e esses bugs desaparecem. Em troca, você paga com uso de memória maior, pausas do GC e menos controle sobre o desempenho.

Rust tenta ficar com os dois: **controle de baixo nível e segurança de memória, sem GC**. O truque é verificar as regras de uso da memória **em tempo de compilação**. Se o programa compila, uma classe inteira de bugs fica descartada, e nada disso custa tempo de execução.

## O problema é real

Não é preocupação acadêmica. Dois números de projetos grandes escritos em C/C++:

- O Chromium relata que [cerca de 70% dos seus bugs de segurança graves são problemas de segurança de memória](https://www.chromium.org/Home/chromium-security/memory-safety/).
- O Android relatou que, conforme a quantidade de código novo em linguagens *memory-safe* (como Rust) cresceu, [as vulnerabilidades de memória caíram de 76% para 35% do total entre 2019 e 2022](https://security.googleblog.com/2022/12/memory-safe-languages-in-android-13.html).

## Para quem vem de C++

Todo o código abaixo compila sem **nenhum aviso** com `clang++ -std=c++20 -Wall -Wextra`:

```cpp
std::string s1 = "oi";
std::string s2 = std::move(s1);
std::cout << s1;            // s1 está "válido, mas não especificado"

std::vector<int> v = {1, 2, 3};
int& primeiro = v[0];
v.push_back(4);             // pode realocar o buffer...
std::cout << primeiro;      // ...e isto é comportamento indefinido

unsigned char c = 255;
c = c + 1;                  // vira 0, em silêncio
```

Em Rust, cada um desses casos vira um **erro de compilação** ou um **erro garantido em tempo de execução** (o programa para com uma mensagem clara, em vez de seguir com lixo na memória):

| Bug em C++ | O que acontece em Rust | Onde o curso trata |
|---|---|---|
| usar valor depois do `std::move` | erro de compilação (E0382) | Módulo 02 |
| referência invalidada por `push_back` | erro de compilação (E0502) | Módulo 03 |
| overflow de inteiro | *panic* em modo debug; em release, só com operações explícitas | Módulo 01 |
| conversão implícita com perda | não existe conversão implícita entre números | Módulo 01 |

A ideia central vai parecer familiar: **RAII**. Em Rust, todo valor tem um dono, e o recurso é liberado quando o dono sai de escopo, como num destrutor. A diferença é que o compilador **prova** que ninguém usa o valor depois disso.

## Para quem vem de Python

O que você ganha:

- **Desempenho previsível:** código compilado para máquina, sem interpretador e sem pausas de GC.
- **Binário único:** sem `venv`, sem "na minha máquina funciona". Você distribui um executável.
- **Erros cedo:** o que em Python vira `AttributeError` ou `TypeError` às 3 da manhã em produção, em Rust é um erro de compilação.
- **Concorrência de verdade:** sem GIL, e com o compilador impedindo *data races*.

O que você paga:

- **Compilação:** existe uma etapa de build, e ela pode ser lenta em projetos grandes.
- **Mais coisas explícitas:** tipos, erros, mutabilidade, quem é dono de cada valor.
- **Curva de aprendizado:** as primeiras semanas têm muita "briga" com o compilador. Este curso existe para encurtar essa fase.

Você provavelmente já usa Rust sem saber: o [`uv`](https://github.com/astral-sh/uv) e o [`ruff`](https://github.com/astral-sh/ruff), ferramentas populares do mundo Python, são escritos em Rust. A biblioteca [Polars](https://pola.rs/) (DataFrames) e o núcleo do [Pydantic](https://github.com/pydantic/pydantic-core) também.

## O que Rust não resolve

Para ser justo:

- **Vazamento de memória** é possível (por exemplo, ciclos de `Rc`, no Módulo 10). Ele é considerado "seguro", porque não corrompe memória, mas continua sendo um bug.
- **Deadlocks** e **erros de lógica** continuam sendo problema seu.
- **`unsafe`** existe: dentro de um bloco `unsafe`, você assume garantias que o compilador não consegue verificar. Ele é necessário para falar com C e para algumas estruturas de baixo nível (Módulo 15). A diferença é que esse código fica **marcado e isolado**, em vez de espalhado pelo programa todo.
- **Tempo de compilação** e **curva de aprendizado** são custos reais.

## Onde Rust é usado

Alguns exemplos públicos e verificáveis:

- **Kernel do Linux:** aceita drivers escritos em Rust.
- **Android e Chromium:** código novo de sistema e de rede.
- **Windows:** partes do kernel e de bibliotecas do sistema.
- **Ferramentas de linha de comando:** `ripgrep`, `fd`, `bat`, `uv`, `ruff`.
- **Infraestrutura:** serviços de rede, bancos de dados, runtimes de WebAssembly.

## Resumo

- Rust entrega segurança de memória **sem GC**, verificando as regras na compilação.
- Para quem vem de C++: os bugs de memória que compilam sem aviso viram erros de compilação.
- Para quem vem de Python: desempenho previsível, binário único e menos erros em produção, em troca de mais coisas explícitas e de uma etapa de build.
- Rust não impede vazamentos, deadlocks nem erros de lógica, e tem `unsafe` para quando é preciso.

## Leitura complementar

- [The Rust Programming Language, Introdução](https://doc.rust-lang.org/stable/book/ch00-00-introduction.html)
- [Comprehensive Rust: Why Rust?](https://google.github.io/comprehensive-rust/why-rust.html)
- [Chromium: Memory safety](https://www.chromium.org/Home/chromium-security/memory-safety/)

---

[Índice do módulo](README.md) · [Próxima lição →](02-instalacao-e-toolchains.md)
