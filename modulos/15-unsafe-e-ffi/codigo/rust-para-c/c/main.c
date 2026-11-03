/* Programa C que usa a biblioteca Rust. Veja a lição 06 para compilar. */
#include <stdio.h>
#include <string.h>
#include "rust_para_c.h"

int main(int argc, char **argv) {
    /* O C e o Rust têm buffers de saída separados. Sem buffer no lado C,
       as linhas dos dois aparecem na ordem em que foram escritas. */
    setvbuf(stdout, NULL, _IONBF, 0);

    printf("rc_somar(2, 40) = %d\n", rc_somar(2, 40));
    printf("palavras = %lld\n", (long long)rc_contar_palavras("Rust chamado de C"));

    Contador *c = rc_contador_novo();
    rc_contador_somar(c, 10);
    printf("contador = %llu\n", (unsigned long long)rc_contador_somar(c, 5));
    rc_contador_liberar(c);

    int32_t q = 0;
    int32_t r = rc_dividir(9, 2, &q);
    printf("dividir(9, 2) -> %d, q = %d\n", r, q);
    r = rc_dividir(9, 0, &q);
    printf("dividir(9, 0) -> %d\n", r);

    if (argc > 1 && strcmp(argv[1], "panico") == 0) {
        rc_panico();
        printf("esta linha nunca aparece\n");
    }
    return 0;
}
