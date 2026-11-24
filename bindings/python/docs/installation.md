# Python Binding Installation & Build

> **Requirement:** CPython 3.12 (`requires-python = ">=3.12,<3.13"` in [`pyproject.toml`](../pyproject.toml)).

## Installing from Package Registry

```sh
pip install kmipkit
```

## Local Source Build with Maturin

Run commands from the repository root with Rust and pinned OpenSSL 3.5.9 configured:

```sh
python3.12 -m venv .venv
. .venv/bin/activate
python -m pip install -r bindings/python/requirements-coverage.txt
python -m pip install --no-build-isolation bindings/python
```

On Windows, create the environment with `py -3.12 -m venv .venv` and activate
`.venv\Scripts\Activate.ps1`.

To produce a wheel without installing directly:

```sh
python -m pip wheel --no-build-isolation --no-deps bindings/python --wheel-dir dist
```
