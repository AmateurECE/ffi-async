#include "app.h"
#include "api.h"
#include "await.h"

int app() {
  while (true) {
    set_led(true);
    await();

    set_led(false);
    await();
  }

  return 0;
}
