# id57

Rust-backed helpers for generating Artemis-compatible base-57 identifiers.

The project packages a Rust extension that mirrors the existing Python
implementation of Artemis' identifier helpers: the `ALPHABET` constant plus the
`base57_encode`, `decode57`, and `generate_id57` functions. Identifiers preserve
lexicographic ordering by concatenating an encoded timestamp (11 base-57
characters) with an encoded UUID component (22 base-57 characters).

## Installation

Pre-built wheels are published to PyPI. Once available, install with:

```bash
pip install id57
```

If a wheel is not available for your platform the package falls back to a
pure-Python implementation that matches the Rust behavior, so the same API is
available everywhere.

## Usage

```python
>>> from id57 import ALPHABET, base57_encode, decode57, generate_id57
>>> ALPHABET
'23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'
>>> base57_encode(999_999)
'7Qnr'
>>> decode57('7Qnr')
999999
>>> len(generate_id57())
33
```

The default generator encodes the current UTC timestamp in microseconds and a
UUID4 into a 33 character string. You can also supply the parts explicitly:

```python
>>> generate_id57(timestamp=1_700_000_000_123_456, uuid=0x1234)
'22HGcpYU2og22222222222222222223Sm'
```

## Development

The repository uses [uv](https://github.com/astral-sh/uv) for Python dependency
management and [maturin](https://github.com/PyO3/maturin) for building the Rust
extension. Helpful targets are available in the `Makefile`:

```bash
make develop   # Build and install the extension into uv's virtual environment
make test      # Run the pytest suite
make build     # Produce wheels via maturin
make publish   # Publish wheels to PyPI (requires credentials)
```

The test suite exercises both the encode/decode helpers and the `generate_id57`
composition logic, comparing the Rust implementation against the bundled
pure-Python reference for parity.

## Publishing workflow

1. Build wheels locally: `make build`
2. Verify the wheel installs and the API works: `pip install dist/id57-*.whl`
3. Publish to TestPyPI or PyPI: `make publish`
4. Tag the release (e.g. `v1.0.0`) and record the changes in `CHANGELOG.md`

The CI workflow publishes signed wheels to PyPI whenever a commit lands on
`main`, using the `PYPI_TOKEN` secret and creating a matching GitHub Release.

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
