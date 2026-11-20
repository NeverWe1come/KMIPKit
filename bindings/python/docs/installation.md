# Python installation and build

Run commands from the repository root. The current
[pyproject.toml](../pyproject.toml) requires CPython 3.12 (`>=3.12,<3.13`).
Source builds also require the repository's Rust toolchain and its native
crypto build prerequisites; see the
[Rust workspace guide](../../../docs/development/rust-workspace.md).

Create and activate an environment:

```sh
python3.12 -m venv .venv
. .venv/bin/activate
```

On Windows create it with `py -3.12 -m venv .venv` and activate it with
`.venv\Scripts\Activate.ps1` in PowerShell.

Install the pinned development tools and build the binding:

```sh
python -m pip install -r bindings/python/requirements-coverage.txt
python -m pip install --no-build-isolation bindings/python
```

Maturin builds the Rust FFI library and CFFI declarations. To create a wheel
from source instead of installing directly:

```sh
python -m pip wheel --no-build-isolation --no-deps bindings/python --wheel-dir dist
```

Build each wheel for its intended interpreter, operating system, and
architecture. Runtime dependencies such as CFFI are installed by pip when
installing the resulting wheel. This guide describes local builds; it does
not establish public package availability.

Continue with [native loading](native-library.md), [testing](testing.md),
and the [local example](../examples/README.md).
