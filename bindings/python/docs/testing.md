# Python testing

Run commands from the repository root after completing
[installation](installation.md) in a Python 3.12 virtual environment.

```sh
python -m pytest -q bindings/python/tests
```

The pinned test and coverage tools are in
[requirements-coverage.txt](../requirements-coverage.txt). To collect coverage:

```sh
python -m pytest -q bindings/python/tests --cov=kmipkit \
  --cov-config=bindings/python/pyproject.toml \
  --cov-report=term-missing
```

The suite includes native loading, handle ownership, operation mapping,
validation, and controlled TLS integration checks. Local peers require
permission to bind loopback ports. Shared test data is under
[bindings/shared/fixtures](../../shared/fixtures/); the
[mTLS README](../../shared/fixtures/login-mtls/README.md) explains the committed
disposable identities.

Run the [local example](../examples/README.md) for an additional registry
consumer. [Verification tooling](../../../verification/README.md) provides
separate API and external-server interoperability checks.
