# Testing the Python Bindings

Run commands from the repository root in a Python 3.12 virtual environment
after following [installation](installation.md):

```sh
python -m pytest -q bindings/python/tests
```

## Coverage Collection

The pinned test and coverage tools are declared in
`bindings/python/requirements-coverage.txt`:

```sh
python -m pytest -q bindings/python/tests --cov=kmipkit \
  --cov-config=bindings/python/pyproject.toml \
  --cov-report=term-missing
```

The suite covers native loading, handle ownership, operation mapping, profile
validation, and local mTLS integration checks using disposable fixtures in
`bindings/shared/fixtures`.
