#include <stdio.h>
#include <time.h>
#include "../c_headers/sdop.h"


int main() {
    SdopTimestamp* ts = sdop_timestamp_new(2025, 10, 14, 7, 20, 0);
    SdopGame* game = sdop_game_blank(ts);
    sdop_timestamp_free(ts);

    struct timespec last_time;
    clock_gettime(CLOCK_MONOTONIC, &last_time);

    while (1) { // main loop
        struct timespec current_time;
        clock_gettime(CLOCK_MONOTONIC, &current_time);

        // Calculate elapsed time in nanoseconds
        uint64_t delta_nanos = (current_time.tv_sec - last_time.tv_sec) * 1000000000ULL
                              + (current_time.tv_nsec - last_time.tv_nsec);

        sdop_game_tick(game, delta_nanos);
        sdop_game_refresh_display(game, delta_nanos);
        last_time = current_time;

        printf("Tick advanced by %llu ns\n", delta_nanos);
    }

    sdop_game_free(game);
    return 0;
}
