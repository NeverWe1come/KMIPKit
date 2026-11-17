"""Path validation for C interoperability tooling inputs and outputs."""
from pathlib import Path


def resolve_within_root(path: Path, root: Path) -> Path:
    """Resolve a path and reject destinations outside the trusted tool root."""
    allowed_root = root.expanduser().resolve()
    candidate = path.expanduser().resolve()
    if candidate == allowed_root or allowed_root not in candidate.parents:
        raise ValueError('path_outside_allowed_root')
    return candidate
