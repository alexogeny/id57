"""Pure-Python reference implementation for id57.

The fallback mirrors the behavior of the Rust extension so that environments
without native wheel support can still generate identifiers.
"""
from __future__ import annotations

from time import time_ns
from typing import Any, Final
from uuid import UUID, uuid4

ALPHABET: Final[str] = "23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
_BASE: Final[int] = len(ALPHABET)
_TIMESTAMP_WIDTH: Final[int] = 11
_UUID_WIDTH: Final[int] = 22
_CHAR_TO_INDEX: Final[dict[str, int]] = {char: index for index, char in enumerate(ALPHABET)}


def _ensure_non_negative(value: int, *, name: str = "value") -> int:
    if value < 0:
        if name == "value":
            raise ValueError("Value must be non-negative")
        raise ValueError(f"{name} must be non-negative")
    return value


def base57_encode(value: int, pad_to: int | None = None) -> str:
    """Encode ``value`` in base-57, optionally left-padding to ``pad_to``."""
    _ensure_non_negative(value)

    if value == 0:
        digits = [ALPHABET[0]]
    else:
        digits: list[str] = []
        current = value
        while current:
            current, remainder = divmod(current, _BASE)
            digits.append(ALPHABET[remainder])
        digits.reverse()

    result = "".join(digits)
    if pad_to and pad_to > len(result):
        result = ALPHABET[0] * (pad_to - len(result)) + result
    return result


def decode57(value: str) -> int:
    """Decode a base-57 string into an integer."""
    if not value:
        raise ValueError("Value cannot be empty")

    total = 0
    for index, char in enumerate(value):
        try:
            digit = _CHAR_TO_INDEX[char]
        except KeyError:
            raise ValueError(f"Invalid base57 character: {char!r} at position {index}") from None
        total = total * _BASE + digit
    return total


def _normalise_uuid(value: Any) -> int:
    if isinstance(value, int):
        _ensure_non_negative(value, name="uuid")
        return value
    if isinstance(value, UUID):
        return value.int
    try:
        candidate = int(value)
    except Exception as exc:  # pragma: no cover - exercised via tests
        raise ValueError("uuid must be an int or expose an __int__ method") from exc
    return _ensure_non_negative(candidate, name="uuid")


def generate_id57(*, timestamp: int | None = None, uuid: Any | None = None) -> str:
    """Generate an Artemis-style identifier.

    The identifier concatenates a timestamp encoded in base-57 with width 11
    and a UUID component encoded with width 22.
    """
    ts_value = _ensure_non_negative(timestamp, name="timestamp") if timestamp is not None else int(
        time_ns() // 1_000
    )
    uuid_value = _normalise_uuid(uuid4() if uuid is None else uuid)
    return base57_encode(ts_value, pad_to=_TIMESTAMP_WIDTH) + base57_encode(
        uuid_value, pad_to=_UUID_WIDTH
    )


__all__ = [
    "ALPHABET",
    "base57_encode",
    "decode57",
    "generate_id57",
]
