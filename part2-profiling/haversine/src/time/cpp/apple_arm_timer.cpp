#include "apple_arm_events.hpp"
#include "apple_arm_timer.hpp"
#include <cstddef>
#include <cstdlib>

#pragma once

struct apple_events_handle {
    struct AppleEvents impl;
};

apple_events_handle* apple_events_create() {
    struct AppleEvents events;
    apple_events_handle* ptr;

    if (events.setup_performance_counters()) {
        ptr = (apple_events_handle*) malloc(sizeof(apple_events_handle));
        if (ptr == NULL) {
            return ptr;
        }
        (*ptr) = (apple_events_handle) { events };

        return ptr;
    } else {
        return NULL;
    }
}
void apple_events_destroy(apple_events_handle* h) {
  delete h;
}
i32 apple_events_get(const apple_events_handle* h, struct perf_counters_c* out) {
    if (h == NULL) return 1;

    AppleEvents events = (*h).impl;
    performance_counters pc = events.get_counters();
    (*out) = (perf_counters_c) {
        pc.cycles, pc.branches, pc.missed_branches, pc.instructions
    };

    return 0;
}
