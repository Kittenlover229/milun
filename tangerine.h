#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Standalone renderer that instead of taking ownership of an existing window creates its own.
 */
typedef struct tangerine_renderer tangerine_renderer;

struct tangerine_renderer *tangerine_new(void *(*alloc)(size_t));

void tangerine_delete(struct tangerine_renderer *renderer, void (*free)(void*));

void tangerine_set_title(struct tangerine_renderer *renderer, const char *title);

void tangerine_run(struct tangerine_renderer *renderer);
