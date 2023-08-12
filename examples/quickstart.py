import wffle
import time
from math import sin, cos

frame_counter = 0

renderer = wffle.Renderer()

begin = time.perf_counter()
renderer.set_background_color(None)

sprite8x8 = renderer.add_sprite(open("examples/8x8.png", "rb").read())
sprite8x16 = renderer.add_sprite(open("examples/8x16.png", "rb").read())

sett = False
sprite16x16 = 0

@renderer.run
def draw(renderer: wffle.Renderer, inputs):
    global frame_counter, sett, sprite16x16
    
    if not sett:
        sprite16x16 = renderer.add_sprite(open("examples/16x16.png", "rb").read())
        sett = True

    frame_counter += 1
    renderer.draw(2, [cos(6 * time.time()) + 1, 0.33 * sin(3 * time.time()) + 1], layer = 1)
    renderer.draw(0, [0, 0], angle = 180 * (sin(time.time()) + 1) )

    renderer.draw(1, inputs.cursor_world_pos, layer=2)
    renderer.set_title(f"{frame_counter}, {inputs.cursor_window_pos}")


end = time.perf_counter()

print(f"Naive FPS: {frame_counter / (end - begin)}")
