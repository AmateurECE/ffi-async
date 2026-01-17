#include <setjmp.h>
#include <stdbool.h>

#include "await.h"

//
// The implementation of the await primitives
//

static jmp_buf SYNC_POINT;
// SAFETY: This pointer contains the CURRENT restore point. This allows for
// multiple CoFuts to be in flight at the same time (e.g when using
// select/join), and it is safe AS LONG AS there is only one executor that is
// allowed to create CoFuts. Interrupt executors create the possibility for
// pre-emption, which would overwrite RETURN_POINT in an unrecoverable way if
// the pre-empting task creates a CoFut while the lower-priority CoFut is in
// flight.
static jmp_buf* RETURN_POINT;

void await() {
  const int result = setjmp(*RETURN_POINT);
  if (0 == result) {
    longjmp(SYNC_POINT, 1);
  }
}

CoPoll trampoline(bool resume, jmp_buf* buffer, int (*app)()) {
  RETURN_POINT = buffer;
  const int awaiting = setjmp(SYNC_POINT);
  if (0 == awaiting) {
    if (resume) {
      longjmp(*RETURN_POINT, 1);
    } else {
      const int result = app();
      return (CoPoll){.status = READY, .result = result};
    }
  }

  // If we got here, it was from a longjmp, and so we are awaiting.
  return (CoPoll){.status = PENDING, 0};
}
