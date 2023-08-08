import wffle
import time

frame_counter = 0

renderer = wffle.Renderer()

begin = time.perf_counter()
renderer.set_background_color([0xFF, 0xFF, 0xFF])

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
    renderer.draw(frame_counter % 3, inputs.cursor_world_pos)
    renderer.set_title(f"{frame_counter}, {inputs.cursor_window_pos}")


end = time.perf_counter()

print(f"Naive FPS: {frame_counter / (end - begin)}")
