"""Sphinx configuration for KMIPKit Python API documentation (PyPI / Read the Docs)."""

import os
import sys

os.environ["SPHINX_BUILD"] = "1"
sys.path.insert(0, os.path.abspath("../src"))

project = "KMIPKit Python"
copyright = "2026, KMIPKit Contributors"
author = "KMIPKit Contributors"
release = "1.0.0"

extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.napoleon",       # Google & NumPy style docstrings
    "sphinx.ext.intersphinx",
    "sphinx.ext.viewcode",
    "sphinx_autodoc_typehints",
    "myst_parser",               # Allows including Markdown guides directly
]

autodoc_member_order = "bysource"
autodoc_typehints = "description"
napoleon_google_docstring = True
napoleon_numpy_docstring = True
autodoc_mock_imports = ["cffi", "kmipkit._native_ffi"]

intersphinx_mapping = {
    "python": ("https://docs.python.org/3", None),
}

html_theme = "furo"
html_title = f"KMIPKit Python {release}"
