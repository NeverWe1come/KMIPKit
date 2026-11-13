"""Path validation helpers for generated Cargo matrix metadata."""
from pathlib import Path


def contained_output_path(output_path: str | Path, output_dir: str | Path) -> Path:
    """Resolve a generated-file path and require it to stay under Cargo OUT_DIR."""
    cargo_output_dir = Path(output_dir).resolve(strict=True)
    if not cargo_output_dir.is_dir():
        raise ValueError("Cargo OUT_DIR must be a directory")
    output = Path(output_path).resolve()
    try:
        output.relative_to(cargo_output_dir)
    except ValueError as error:
        raise ValueError("generated matrix hash path must stay inside Cargo OUT_DIR") from error
    if output == cargo_output_dir:
        raise ValueError("generated matrix hash path must name a file")
    return output
