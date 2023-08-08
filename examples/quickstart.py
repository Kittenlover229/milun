import wffle
import time

frame_counter = 0

renderer = wffle.Renderer()

begin = time.perf_counter()
renderer.set_background_color([0xFF, 0xFF, 0xFF])

renderer.add_sprite(open("examples/8x8.png", "rb").read())
renderer.add_sprite(open("examples/8x16.png", "rb").read())
renderer.add_sprite(open("examples/16x16.png", "rb").read())


@renderer.run
def draw(renderer: wffle.Renderer, inputs):
    global frame_counter
    frame_counter += 1
    renderer.draw(2, inputs.cursor_world_pos)
    renderer.set_title(f"{frame_counter}, {inputs.cursor_window_pos}")


end = time.perf_counter()

print(f"Naive FPS: {frame_counter / (end - begin)}")
