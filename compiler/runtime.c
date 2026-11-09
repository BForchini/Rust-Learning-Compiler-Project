#include <stdio.h>
#include <stdlib.h>

double compiler_input_number(void) {
    double value;
    if (scanf("%lf", &value) != 1) {
        fputs("Input is not a valid number.\n", stderr);
        exit(EXIT_FAILURE);
    }
    return value;
}

void compiler_print_number(double value) {
    printf("%g\n", value);
    fflush(stdout);
}

void compiler_print_string(const char *value) {
    puts(value);
    fflush(stdout);
}
