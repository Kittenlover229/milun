#include <stdio.h>

#include "../../tangerine.h"

void draw_callback(tangerine_renderer *renderer, tangerine_input input) {
  printf("(%d, %d)\n", input.cursor_position_x, input.cursor_position_y);
}

int main(void) {
  struct tangerine_renderer *renderer = tangerine_new();

  unsigned char colors[3] = {0x00, 0xFF, 0xFF};
  tangerine_set_background_color(renderer, colors);

  tangerine_set_title(renderer, "Tangerine");
  tangerine_run(renderer, draw_callback);

  tangerine_delete(renderer);
  return 0;
}
