#include <stdio.h>
#include "pilha.h"

int main(void) {
    double v[] = {1.5, 2.5, 5.0};
    double m;
    if (ex_media(v, 3, &m) == 0) printf("media = %.2f\n", m);

    Pilha *p = ex_pilha_nova();
    for (int64_t i = 1; i <= 3; i++) ex_pilha_empilhar(p, i * 100);
    printf("tamanho = %zu\n", ex_pilha_tamanho(p));
    int64_t x;
    while (ex_pilha_desempilhar(p, &x)) printf("desempilhou %lld\n", (long long)x);
    ex_pilha_liberar(p);
    return 0;
}
