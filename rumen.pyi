from typing import ClassVar, final

__all__ = [
    "Block",
    "ColorScheme",
    "DecodeError",
    "Facing",
    "Flag",
    "Fumen",
    "Location",
    "Page",
    "ViewMode",
    "parse",
]

class DecodeError(ValueError):
    """A fumen input could not be decoded; the message preserves the Rust error."""

@final
class Block:
    Empty: ClassVar[Block]
    I: ClassVar[Block]
    L: ClassVar[Block]
    O: ClassVar[Block]
    Z: ClassVar[Block]
    T: ClassVar[Block]
    J: ClassVar[Block]
    S: ClassVar[Block]
    Garbage: ClassVar[Block]
    def __int__(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Facing:
    Down: ClassVar[Facing]
    Right: ClassVar[Facing]
    Up: ClassVar[Facing]
    Left: ClassVar[Facing]
    def __int__(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __hash__(self) -> int: ...

@final
class ColorScheme:
    GuideLine: ClassVar[ColorScheme]
    Sega: ClassVar[ColorScheme]
    def __int__(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __hash__(self) -> int: ...

@final
class ViewMode:
    Normal: ClassVar[ViewMode]
    Mini: ClassVar[ViewMode]
    List: ClassVar[ViewMode]
    MiniList: ClassVar[ViewMode]
    def __int__(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Location:
    """Wire coordinate: x=0..9, y=0..23, with (0, 0) at the top-left."""
    @property
    def x(self) -> int: ...
    @property
    def y(self) -> int: ...
    def __eq__(self, other: object, /) -> bool: ...
    def __hash__(self) -> int: ...

@final
class Flag:
    @property
    def mino(self) -> Block: ...
    @property
    def face(self) -> Facing | None: ...
    @property
    def location(self) -> Location | None: ...
    @property
    def up(self) -> bool: ...
    @property
    def mirror(self) -> bool: ...
    @property
    def color_scheme(self) -> ColorScheme: ...
    @property
    def note(self) -> bool: ...
    @property
    def lock(self) -> bool: ...
    def positions(self) -> list[tuple[int, int]] | None:
        """The four occupied cells in top-down coordinates, or None for no mino."""
    def __eq__(self, other: object, /) -> bool: ...

@final
class Page:
    @property
    def field(self) -> list[list[Block]]:
        """A copy of the 24x10 pre-action field; row 23 is garbage."""
    @property
    def flag(self) -> Flag: ...
    @property
    def notes(self) -> str:
        """The resolved comment, including inheritance and quiz progression."""
    def __eq__(self, other: object, /) -> bool: ...

@final
class Fumen:
    @property
    def mode(self) -> ViewMode: ...
    @property
    def version(self) -> int: ...
    @property
    def pages(self) -> list[Page]:
        """A copy of the decoded page list."""
    def __eq__(self, other: object, /) -> bool: ...

def parse(input: str) -> Fumen:
    """Decode a v115 string or viewer URL; invalid input raises DecodeError."""
