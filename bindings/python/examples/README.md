# Python examples

[vendor_extension_registry.py](vendor_extension_registry.py) registers and
inspects vendor extension definitions using the shared TTLV fixtures, validates
local values, and constructs a Discover Versions batch item. It uses non-secret
sample data and opens no transport.

After [installing the binding](../docs/installation.md), run from the
repository root in the active Python 3.12 environment:

```sh
python bindings/python/examples/vendor_extension_registry.py
```

The example reads [shared extension cases](../../shared/fixtures/extensions/cases.json)
from the checkout. It uses context managers to release native handles. Treat
application-owned Python strings and bytes separately from native zeroization.
See [native loading](../docs/native-library.md) for library resolution and
[testing](../docs/testing.md) for the binding suite.
