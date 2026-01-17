#include "app.h"
#include "api.h"

volatile bool READY = false;

int app() {
  while (true) {
    set_led(true);

    while (!READY)
      ;

    set_led(false);

    while (READY)
      ;
  }

  return 0;
}
