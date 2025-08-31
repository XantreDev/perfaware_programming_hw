// apple_arm_events_c.h
#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef struct apple_events_handle apple_events_handle; // opaque handle

apple_events_handle* apple_events_create(void);
void apple_events_destroy(apple_events_handle* h);

// returns 1 on success, 0 on failure
int apple_events_setup(apple_events_handle* h);

struct perf_counters_c {
  double cycles, branches, missed_branches, instructions;
};

// returns 1 on success, 0 on failure
int apple_events_get(const apple_events_handle* h, struct perf_counters_c* out);

#ifdef __cplusplus
}
#endif
