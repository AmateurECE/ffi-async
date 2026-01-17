#ifndef AWAIT_H
#define AWAIT_H

#include <stdbool.h>

// Status of a Future
typedef enum {
  PENDING,
  READY,
} PollStatus;

// The result of polling a Future created using `trampoling'
typedef struct {
  PollStatus status;
  // Only valid when status == READY
  int result;
} CoPoll;

// Yields back to the Embassy executor until awoken.
void await();

// Execute `app' as a coroutine, allowing it to await and be awoken. If
// `resume' is true, resumes the execution of `app' from the last await point.
// Otherwise, restarts app.
CoPoll trampoline(bool resume, int (*app)());

#endif // AWAIT_H
