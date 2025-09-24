from __future__ import annotations

import random
from uuid import UUID

import pytest

from id57 import ALPHABET, base57_encode, decode57, generate_id57
from id57 import _fallback as reference


def test_alphabet_matches_reference() -> None:
    assert ALPHABET == reference.ALPHABET
    assert len(ALPHABET) == 57
    assert len(set(ALPHABET)) == 57
    for forbidden in "O0lI1":
        assert forbidden not in ALPHABET


def test_base57_known_values() -> None:
    cases = {
        0: "2",
        1: "3",
        56: "z",
        57: "32",
        58: "33",
        999_999: "7Qnr",
        2**64: "txLqViLENDy",
        2**128 - 1: "oZEq7ovRbLq6UnGMPwc8B5",
    }
    big_values = [
        1 << 200,
        (1 << 400) + 123_456_789,
    ]
    for value in big_values:
        cases[value] = reference.base57_encode(value)
    for value, expected in cases.items():
        assert base57_encode(value) == expected
        assert decode57(expected) == value


@pytest.mark.parametrize(
    ("value", "width", "expected"),
    [
        (1, 4, "2223"),
        (0, 3, "222"),
        (57, 1, "32"),
        (58, 2, "33"),
    ],
)
def test_base57_padding_behavior(value: int, width: int, expected: str) -> None:
    assert base57_encode(value, pad_to=width) == expected


def test_base57_negative_rejected() -> None:
    with pytest.raises(ValueError, match="Value must be non-negative"):
        base57_encode(-1)


def test_decode_empty_string_rejected() -> None:
    with pytest.raises(ValueError, match="Value cannot be empty"):
        decode57("")


def test_decode_invalid_character() -> None:
    with pytest.raises(ValueError, match="Invalid base57 character: '0' at position 0"):
        decode57("0")


def test_round_trip_random_values() -> None:
    rng = random.Random(42)
    for _ in range(200):
        value = rng.getrandbits(128)
        encoded = base57_encode(value)
        assert decode57(encoded) == value


def test_round_trip_with_padding() -> None:
    rng = random.Random(99)
    for width in (1, 10, 22):
        for _ in range(25):
            value = rng.getrandbits(64)
            encoded = base57_encode(value, pad_to=width)
            assert len(encoded) >= width
            assert decode57(encoded) == value


def test_decode_preserves_arbitrary_precision() -> None:
    value = (1 << 512) + 987_654_321
    encoded = reference.base57_encode(value)
    decoded = decode57(encoded)
    assert isinstance(decoded, int)
    assert decoded == value
    assert decoded.bit_length() == value.bit_length()


def test_generate_id57_composition() -> None:
    timestamp = 1_700_000_000_123_456
    uuid_value = 0x1234
    expected = reference.base57_encode(timestamp, pad_to=11) + reference.base57_encode(uuid_value, pad_to=22)
    assert generate_id57(timestamp=timestamp, uuid=uuid_value) == expected


def test_generate_id57_accepts_uuid_instances() -> None:
    timestamp = 1_700_000_000_654_321
    uuid_obj = UUID(int=987_654_321)
    value = generate_id57(timestamp=timestamp, uuid=uuid_obj)
    expected = reference.base57_encode(timestamp, pad_to=11) + reference.base57_encode(uuid_obj.int, pad_to=22)
    assert value == expected


def test_generate_id57_negative_parts_rejected() -> None:
    with pytest.raises(ValueError, match="timestamp must be non-negative"):
        generate_id57(timestamp=-1)
    with pytest.raises(ValueError, match="uuid must be non-negative"):
        generate_id57(timestamp=0, uuid=-1)


def test_generate_id57_ordering() -> None:
    earlier = generate_id57(timestamp=1, uuid=0)
    later_same_ts = generate_id57(timestamp=1, uuid=1)
    later_ts = generate_id57(timestamp=2, uuid=0)
    assert earlier < later_same_ts < later_ts


def test_generate_id57_default_length() -> None:
    identifier = generate_id57()
    assert len(identifier) == 33
    ts_part = identifier[:11]
    assert ts_part == reference.base57_encode(reference.decode57(ts_part), pad_to=11)
