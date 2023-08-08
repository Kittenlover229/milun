import wffle

def draw(renderer: wffle.Renderer):
    renderer.set_background_color((0xFF, 0xFF, 0xFF))

renderer = wffle.Renderer()
renderer.run(draw)

print("Rendering over, do a graceful shutdown")
