from typing import Callable, Any, Union

class Renderer:
    def __init__(self) -> None: ...
    def run(self, draw_callback: Callable[[Any, Any], None]) -> None: ...
    def set_title(self, title: str) -> None: ...
    def set_background_color(
        self, tuple: Tuple[int, int, int] | Tuple[float, float, float]
    ) -> None: ...
    def draw(
        self,
        sprite_index: int,
        position: Tuple[float, float],
        *,
        layer: Union[str, int] = 0,
        angle: 0.0,
        color: Tuple[int, int, int] | Tuple[float, float, float] = (1.0, 1.0, 1.0),
        scale: float | Tuple[float, float] = 1.0,
        opacity: float = 1.0,
    ) -> None: ...
    def add_sprite(self, buffer: bytes) -> int: ...

class Input:
    def __init__(self):
        self.cursor_pos: tuple[int, int]
    @property
    def cursor_window_pos(self) -> tuple[int, int]: ...
    @property
    def cursor_world_pos(self) -> tuple[float, float]: ...
