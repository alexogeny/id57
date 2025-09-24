"""Artemis-compatible base-57 identifier helpers.

The module exposes ``ALPHABET``, ``base57_encode``, ``decode57`` and
``generate_id57`` which mirror the original Python implementation while being
backed by a Rust extension when available.
"""
from __future__ import annotations

from importlib import import_module
from importlib.metadata import PackageNotFoundError, version

__all__ = [
    "ALPHABET",
    "base57_encode",
    "decode57",
    "generate_id57",
    "__version__",
]

try:  # pragma: no cover - exercised in environments without native wheel support
    _core = import_module("id57._core")
except ModuleNotFoundError:  # pragma: no cover - fallback path
    from . import _fallback as _impl
else:
    _impl = _core  # type: ignore[assignment]

ALPHABET = _impl.ALPHABET
base57_encode = _impl.base57_encode
decode57 = _impl.decode57
generate_id57 = _impl.generate_id57

try:
    __version__ = version("id57")
except PackageNotFoundError:  # pragma: no cover - local builds
    __version__ = "0.0.0"


def __getattr__(name: str):  # pragma: no cover - defensive programming
    if name in __all__:
        return globals()[name]
    raise AttributeError(f"module 'id57' has no attribute {name!r}")
