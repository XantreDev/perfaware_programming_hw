#include <stdio.h>
#include <time.h>

int main() {
    struct timespec tp;

    clock_gettime(CLOCK_MONOTONIC_RAW, &tp);

    printf("system time: %ld", tp.tv_nsec);

    return 0;
}
