from __future__ import annotations

from typing import Any, Optional

ALPHABET: str

def base57_encode(value: int, pad_to: Optional[int] = ...) -> str: ...

def decode57(value: str) -> int: ...

def generate_id57(*, timestamp: Optional[int] = ..., uuid: Optional[Any] = ...) -> str: ...
