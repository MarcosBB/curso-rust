/* Cabeçalho escrito à mão para a biblioteca rust-para-c.
   Em projetos maiores, o cbindgen gera este arquivo a partir do código Rust. */
#ifndef RUST_PARA_C_H
#define RUST_PARA_C_H

#include <stdint.h>

/* Tipo opaco: o C só manipula ponteiros para ele. */
typedef struct Contador Contador;

int32_t rc_somar(int32_t a, int32_t b);
int64_t rc_contar_palavras(const char *texto);

Contador *rc_contador_novo(void);
uint64_t rc_contador_somar(Contador *c, uint64_t n);
void rc_contador_liberar(Contador *c);

int32_t rc_dividir(int32_t a, int32_t b, int32_t *saida);
void rc_panico(void);

#endif
