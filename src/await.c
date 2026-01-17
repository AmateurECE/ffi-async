#include <setjmp.h>
#include <stdbool.h>

#include "await.h"

//
// The implementation of the await primitives
//

static jmp_buf SYNC_POINT;
static jmp_buf RETURN_POINT;

void await() {
  const int result = setjmp(RETURN_POINT);
  if (0 == result) {
    longjmp(SYNC_POINT, 1);
  }
}

CoPoll trampoline(bool resume, int (*app)()) {
  const int awaiting = setjmp(SYNC_POINT);
  if (0 == awaiting) {
    if (resume) {
      longjmp(RETURN_POINT, 1);
    } else {
      const int result = app();
      return (CoPoll){.status = READY, .result = result};
    }
  }

  // If we got here, it was from a longjmp, and so we are awaiting.
  return (CoPoll){.status = PENDING, 0};
}
