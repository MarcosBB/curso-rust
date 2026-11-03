#ifndef PILHA_H
#define PILHA_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef struct Pilha Pilha;

int32_t ex_media(const double *v, size_t n, double *saida);

Pilha *ex_pilha_nova(void);
void ex_pilha_empilhar(Pilha *p, int64_t x);
bool ex_pilha_desempilhar(Pilha *p, int64_t *saida);
size_t ex_pilha_tamanho(const Pilha *p);
void ex_pilha_liberar(Pilha *p);

#endif
