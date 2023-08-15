#include "../../tangerine.h"

int main(void) {
  struct tangerine_renderer *renderer = tangerine_new(malloc);
  tangerine_set_title(renderer, "carrot");
  tangerine_run(renderer);
  tangerine_delete(renderer, free);
  return 0;
}
