#pragma once

#include "apple_arm_events.hpp"
#ifdef __cplusplus
extern "C" {
#endif

typedef struct apple_events_handle apple_events_handle; // opaque handle

apple_events_handle* apple_events_create(void);
void apple_events_destroy(apple_events_handle* h);

struct perf_counters_c {
  u64 cycles, branches, missed_branches, instructions;
};

// returns 1 on success, 0 on failure
i32 apple_events_get(const apple_events_handle* h, struct perf_counters_c* out);

#ifdef __cplusplus
}
#endif
