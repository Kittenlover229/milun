#ifndef TANGERINE_H
#define TANGERINE_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct tangerine_renderer tangerine_renderer;

typedef struct {
  unsigned int cursor_position_x;
  unsigned int cursor_position_y;
} tangerine_input;

typedef void (*tangerine_draw_fn)(tangerine_renderer *, tangerine_input);

struct tangerine_renderer *tangerine_new(void);

void tangerine_delete(struct tangerine_renderer *renderer);

void tangerine_set_title(struct tangerine_renderer *renderer,
                         const char *title);

void tangerine_set_background_color(struct tangerine_renderer *renderer,
                                    unsigned char *color_rgb);

void tangerine_run(struct tangerine_renderer *renderer, tangerine_draw_fn fn);

#ifdef __cplusplus
}
#endif

#endif // TANGERINE_H
