/* Biblioteca C pequena usada pela lição 05 do Módulo 15. */
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

typedef struct {
    double x;
    double y;
} Ponto;

/* Média de `n` valores. Devolve 0.0 se n == 0. */
double media(const double *valores, size_t n) {
    if (n == 0) return 0.0;
    double soma = 0.0;
    for (size_t i = 0; i < n; i++) soma += valores[i];
    return soma / (double)n;
}

/* Distância entre dois pontos, passados por valor. */
double distancia(Ponto a, Ponto b) {
    return hypot(b.x - a.x, b.y - a.y);
}

/* Aplica `f` a cada elemento de `v`, no lugar. */
void aplicar(int32_t *v, size_t n, int32_t (*f)(int32_t)) {
    for (size_t i = 0; i < n; i++) v[i] = f(v[i]);
}

/* Devolve uma string nova (alocada com malloc) com uma saudação.
   Quem chama precisa liberar com `liberar_texto`. Devolve NULL se faltar memória. */
char *saudacao(const char *nome) {
    const char *prefixo = "Olá, ";
    size_t tam = strlen(prefixo) + strlen(nome) + 2;
    char *saida = malloc(tam);
    if (saida == NULL) return NULL;
    snprintf(saida, tam, "%s%s!", prefixo, nome);
    return saida;
}

void liberar_texto(char *texto) {
    free(texto);
}
