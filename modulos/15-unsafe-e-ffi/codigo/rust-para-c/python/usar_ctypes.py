"""Usa a biblioteca Rust a partir do Python, com ctypes (lição 07).

Antes, rode `cargo build --release` na pasta rust-para-c.
"""

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
