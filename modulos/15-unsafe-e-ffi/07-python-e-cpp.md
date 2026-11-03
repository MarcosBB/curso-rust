# 07 — Rust com Python e C++

**Objetivos.** Ao final, você vai saber:

- chamar uma biblioteca Rust a partir do Python com `ctypes`, e por que declarar os tipos é obrigatório;
- como o PyO3 transforma funções Rust em um módulo Python de verdade, sem `unsafe` no seu código;
- o que a crate `cxx` faz para C++ e quando usá-la no lugar de `extern "C"`;
- escolher a ferramenta certa para cada combinação de linguagens.

## Python: três jeitos de chamar código nativo

Python puro não tem UB: um índice errado vira `IndexError`, um objeto liberado não pode ser usado porque o contador de referências não deixa. O risco mora nas **extensões nativas**, onde mora também a velocidade (NumPy, pandas, cryptography, Pydantic). Há três jeitos de chegar lá:

| | API C do CPython | `ctypes` | PyO3 |
|---|---|---|---|
| linguagem da extensão | C | qualquer uma com ABI de C | Rust |
| quem converte os tipos | você, à mão | você, declarando `argtypes`/`restype` | macros, a partir dos tipos Rust |
| contagem de referências | à mão (`Py_INCREF`/`Py_DECREF`) | automática | automática (tipos `Bound`/`Py`) |
| erros | `return NULL` + `PyErr_SetString` | códigos de retorno | `PyResult<T>` vira exceção |
| onde pode haver UB | em toda a extensão | em cada chamada mal declarada | só no `unsafe` (que o código de exemplo não tem) |

### A API C do CPython

Uma extensão clássica é um arquivo C que inclui `Python.h` e manipula `PyObject*` diretamente. Um trecho de como seria `contar_palavras`:

```c
static PyObject *contar_palavras(PyObject *self, PyObject *args) {
    const char *texto;
    if (!PyArg_ParseTuple(args, "s", &texto))   /* converte o argumento */
        return NULL;                             /* exceção já definida */
    long n = 0;
    /* ... contar ... */
    return PyLong_FromLong(n);                   /* nova referência */
}
```

Funciona e é o que está por baixo de tudo, mas cada `PyObject*` tem regras de posse ("nova referência" ou "referência emprestada") que você precisa acompanhar à mão. Esquecer um `Py_DECREF` é vazamento; um a mais é *use-after-free*. É exatamente a classe de erro que o Rust elimina.

### `ctypes`: chamando a `cdylib` da lição 06

O `ctypes`, da biblioteca padrão do Python, carrega uma biblioteca dinâmica e chama funções de C sem compilar nada. A biblioteca da lição 06 serve como está. O script [`python/usar_ctypes.py`](codigo/rust-para-c/python/usar_ctypes.py):

```python
import ctypes
import pathlib
import sys

pasta = pathlib.Path(__file__).resolve().parent.parent / "target" / "release"
nome = {"darwin": "librust_para_c.dylib", "win32": "rust_para_c.dll"}.get(
    sys.platform, "librust_para_c.so"
)
lib = ctypes.CDLL(str(pasta / nome))

# Sem estas declarações, o ctypes supõe que tudo recebe e devolve `int`.
lib.rc_somar.argtypes = [ctypes.c_int32, ctypes.c_int32]
lib.rc_somar.restype = ctypes.c_int32
lib.rc_contar_palavras.argtypes = [ctypes.c_char_p]
lib.rc_contar_palavras.restype = ctypes.c_int64
lib.rc_contador_novo.restype = ctypes.c_void_p
lib.rc_contador_somar.argtypes = [ctypes.c_void_p, ctypes.c_uint64]
lib.rc_contador_somar.restype = ctypes.c_uint64
lib.rc_contador_liberar.argtypes = [ctypes.c_void_p]

print("somar:", lib.rc_somar(2, 40))
print("palavras:", lib.rc_contar_palavras("olá do Python".encode()))
print("nulo:", lib.rc_contar_palavras(None))

c = lib.rc_contador_novo()
lib.rc_contador_somar(c, 7)
print("contador:", lib.rc_contador_somar(c, 1))
lib.rc_contador_liberar(c)
```

```bash
cd codigo/rust-para-c
cargo build --release
python3 -u python/usar_ctypes.py
```

```text
somar: 42
palavras: 3
nulo: -1
contador: 8
[rust] liberando contador com valor 8
```

(O `-u` desliga o buffer do Python, pelo mesmo motivo do `setvbuf` da lição 06: num pipe, a linha do Rust sairia antes das do Python.) O `None` virou um ponteiro nulo, e o `rc_contar_palavras` tratou o caso, como prometido no contrato.

As declarações de tipos são o equivalente ao `unsafe extern` da lição 05, só que conferidas por ninguém. Veja o que acontece sem elas:

```python
import ctypes
lib = ctypes.CDLL("target/release/librust_para_c.dylib")
c = lib.rc_contador_novo()
print(hex(c))
lib.rc_contador_somar(c, 1)
```

```text
0x2f73a50
```

E o processo morre com *segmentation fault* (código de saída 139), sem *traceback*. Sem `restype`, o ctypes leu o ponteiro de 64 bits como um `int` de 32 e jogou fora a metade de cima. O endereço `0x2f73a50` não aponta para o contador, e a chamada seguinte o desreferenciou. É UB a partir de Python puro, sem nenhuma linha de C escrita por você.

O `ctypes` é ótimo para chamar uma biblioteca C pronta, ou para um protótipo rápido. Para escrever uma extensão nova em Rust, há um caminho melhor.

### PyO3: um módulo Python escrito em Rust

O [PyO3](https://pyo3.rs/) é um conjunto de macros e tipos que gera toda a camada da API C do CPython a partir de funções Rust comuns:

```rust
use pyo3::prelude::*;

/// Conta as palavras de um texto.
#[pyfunction]
fn contar_palavras(texto: &str) -> usize {
    texto.split_whitespace().count()
}

/// Soma uma lista de inteiros. Levanta OverflowError se estourar.
#[pyfunction]
fn somar(valores: Vec<i64>) -> PyResult<i64> {
    valores
        .iter()
        .try_fold(0i64, |acc, &v| acc.checked_add(v))
        .ok_or_else(|| pyo3::exceptions::PyOverflowError::new_err("soma estourou i64"))
}

#[pymodule]
fn contapy(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(contar_palavras, m)?)?;
    m.add_function(wrap_pyfunction!(somar, m)?)?;
    Ok(())
}
```

O `Cargo.toml` pede `crate-type = ["cdylib"]` e `pyo3 = { version = "0.30", features = ["extension-module"] }`. Os atributos `#[pyfunction]` e `#[pymodule]` são macros procedurais ([Módulo 14](../14-macros/README.md)) que geram as funções `extern "C"` com a assinatura que o CPython espera, a conversão dos argumentos e a tradução de `Err` em exceção.

```python
import contapy
print(contapy.contar_palavras('Rust dentro do Python'))
print(contapy.somar([1, 2, 3]))
print(contapy.contar_palavras.__doc__)
try:
    contapy.somar([2**62, 2**62])
except OverflowError as e:
    print('OverflowError:', e)
try:
    contapy.somar(['a'])
except TypeError as e:
    print('TypeError:', e)
```

```text
4
6
Conta as palavras de um texto.
OverflowError: soma estourou i64
TypeError: 'str' object cannot be interpreted as an integer
```

Repare no que veio de graça: a *docstring* saiu do comentário `///`, o `PyResult` virou exceção Python, e uma lista com uma `str` foi recusada com `TypeError` antes de o código Rust rodar, porque o PyO3 tentou converter o argumento para `Vec<i64>`. Não há nenhum `unsafe` no código acima.

Para compilar e instalar, o caminho recomendado é o [maturin](https://www.maturin.rs/):

```bash
pip install maturin
maturin new contapy          # ou maturin init numa pasta existente
maturin develop --release    # compila e instala no virtualenv atual
```

Ele cuida das flags de ligação de cada plataforma (para esta lição, a extensão foi compilada com `cargo build` e as flags de macOS à mão, para mostrar que não há mágica) e gera *wheels* para publicar no PyPI. É assim que Pydantic (pydantic-core), Polars, ruff, uv e tokenizers distribuem o núcleo em Rust.

## C++: `extern "C"` ou `cxx`

Rust e C++ conversam pela ABI de C desde o primeiro dia: o C++ declara `extern "C"` do lado dele, e tudo o que as lições 05 e 06 fizeram funciona igual. O limite é que só tipos de C atravessam. Uma `std::string`, um `std::vector`, um `std::unique_ptr` ou um `String` do Rust precisam virar ponteiro + tamanho, e a posse precisa ser combinada à mão.

A crate [`cxx`](https://cxx.rs/), de David Tolnay, ataca isso. Você descreve a fronteira uma vez, num módulo `#[cxx::bridge]`, e ela gera os dois lados, com tipos que fazem sentido em cada linguagem. Um trecho do estilo da documentação dela:

```rust
#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type Indice;
        fn buscar(indice: &Indice, termo: &str) -> Vec<String>;
    }

    unsafe extern "C++" {
        include!("motor/include/compressor.h");
        type Compressor;
        fn novo_compressor() -> UniquePtr<Compressor>;
        fn comprimir(self: &Compressor, dados: &[u8]) -> Vec<u8>;
    }
}
```

- `&str`, `&[u8]`, `String`, `Vec<u8>` e `Box<T>` aparecem no C++ como `rust::Str`, `rust::Slice`, `rust::String`, `rust::Vec` e `rust::Box`; `std::unique_ptr` e `std::string` aparecem no Rust como `UniquePtr` e `CxxString`.
- O código gerado confere estaticamente que as declarações batem com o cabeçalho C++, o que elimina a classe de bug da "assinatura errada" da lição 05.
- O `unsafe extern "C++"` segue a mesma ideia do `unsafe extern "C"`: o bloco declara coisas que o Rust não verifica sozinho. Mas, uma vez declaradas, as funções podem ser chamadas sem `unsafe`, porque a ponte garante a correspondência dos tipos.

Para bibliotecas C++ grandes já existentes, há o `autocxx`, que gera a ponte do `cxx` a partir dos cabeçalhos (como o `bindgen` faz para C). É a abordagem do Chromium, que adotou Rust em partes novas do navegador.

## Qual usar

| Situação | Ferramenta |
|---|---|
| chamar uma biblioteca C a partir de Rust | `bindgen` (crate `-sys`) + wrapper seguro |
| compilar um pouco de C junto com o crate | `cc` no `build.rs` |
| oferecer uma biblioteca Rust para C (ou Go, Swift, C#...) | `cdylib`/`staticlib` + `cbindgen` |
| integrar Rust e C++ no mesmo projeto | `cxx` (ou `autocxx` para muitos cabeçalhos) |
| acelerar uma parte de um programa Python | PyO3 + maturin |
| chamar de Python uma biblioteca C que já existe | `ctypes` ou `cffi` |

Em todos os casos, a estrutura é a deste módulo: o `unsafe` fica concentrado numa camada fina e bem testada, e o resto do código (o Rust que usa a biblioteca, ou o Python que usa a extensão) não precisa saber que ele existe.

## Armadilhas comuns

- **`ctypes` sem `argtypes`/`restype`.** O padrão é `int`, e um ponteiro de 64 bits é truncado sem aviso. Declare todas as funções, sempre.
- **Strings no `ctypes`.** `c_char_p` espera `bytes`, não `str`: use `.encode()` (UTF-8) ao chamar, como no exemplo.
- **Panic numa função do PyO3.** O PyO3 captura o panic e levanta `PanicException` no Python, sem abortar. Mesmo assim, para erros esperados, devolva `PyResult` com a exceção certa; o panic é para bugs.
- **Segurar o GIL em trabalho longo.** Uma função PyO3 que calcula por segundos trava as outras threads do Python. O PyO3 tem uma API para soltar o GIL durante o cálculo (consulte o guia, seção sobre paralelismo).
- **Achar que `cxx` dispensa cuidado.** Ele elimina a classe "assinatura errada" e converte tipos, mas posse e lifetimes através da fronteira continuam exigindo atenção, principalmente com referências guardadas do lado C++.

## Resumo

- Extensões em C para Python exigem contagem de referências manual; `ctypes` chama bibliotecas prontas, mas tudo depende de declarar os tipos (sem `restype`, o ponteiro foi truncado e o processo caiu).
- PyO3 gera a camada da API do CPython a partir de funções Rust: tipos convertidos, `PyResult` vira exceção, docstrings vêm do `///`, e o seu código não tem `unsafe`. Use maturin para compilar e publicar.
- C++ fala com Rust por `extern "C"`; para tipos ricos (`String`, `Vec`, `unique_ptr`), use `cxx` (ou `autocxx`).
- Em qualquer combinação, concentre o `unsafe` numa camada fina e ofereça uma API segura em cima.

## Exercícios

Os exercícios do módulo estão em [`exercicios.md`](exercicios.md). O exercício 5 tem um desafio opcional: chamar a sua biblioteca a partir de C.

## Leitura complementar

- [Documentação do `ctypes`](https://docs.python.org/3/library/ctypes.html)
- [Python: Extending and Embedding the Python Interpreter](https://docs.python.org/3/extending/extending.html)
- [PyO3 User Guide](https://pyo3.rs/) e [maturin](https://www.maturin.rs/)
- [CXX: safe interop between Rust and C++](https://cxx.rs/)
- [Comprehensive Rust: Interoperability](https://google.github.io/comprehensive-rust/)

---

[← Lição anterior](06-expondo-rust-para-c.md) · [Índice do módulo](README.md) · [Próximo módulo →](../16-ecossistema-na-pratica/README.md)
