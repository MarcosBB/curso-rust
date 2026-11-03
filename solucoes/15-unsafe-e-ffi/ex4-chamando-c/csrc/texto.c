#include <stddef.h>

/* Conta as vogais ASCII (maiúsculas ou minúsculas) de uma string terminada em '\0'. */
size_t contar_vogais(const char *s) {
    size_t n = 0;
    for (; *s != '\0'; s++) {
        switch (*s) {
        case 'a': case 'e': case 'i': case 'o': case 'u':
        case 'A': case 'E': case 'I': case 'O': case 'U':
            n++;
        }
    }
    return n;
}

/* Inverte os `n` bytes de `buf`, no lugar. */
void inverter(char *buf, size_t n) {
    for (size_t i = 0; i < n / 2; i++) {
        char t = buf[i];
        buf[i] = buf[n - 1 - i];
        buf[n - 1 - i] = t;
    }
}

/* Grava em *saida o maior dos `n` valores. Devolve 0, ou -1 se n == 0. */
int maior(const int *v, size_t n, int *saida) {
    if (n == 0) return -1;
    int m = v[0];
    for (size_t i = 1; i < n; i++) if (v[i] > m) m = v[i];
    *saida = m;
    return 0;
}
