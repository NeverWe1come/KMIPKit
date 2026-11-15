"""Helpers for keeping generated route tables out of handwritten coverage."""

import re
from collections.abc import Iterable

GENERATED_SOURCE_REGEX = re.compile(
    r"local_cases_generated[.]rs|route_calls_generated[.]rs"
)


def generated_source_files(file_paths: Iterable[str]) -> list[str]:
    """Return generated Rust source paths included in a coverage report."""
    return sorted(path for path in file_paths if GENERATED_SOURCE_REGEX.search(path))
