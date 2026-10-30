"""Inventory C, Java, and Python declaration surfaces against manifest v2.

The scanners are deliberately source-based and fail closed on constructs they
cannot classify. A successful parse is not sufficient to close T011: exact
manifest comparison must also succeed, and all reports must be complete.
"""

from __future__ import annotations

import argparse
import ast
from collections import Counter
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


_PYTHON_PUBLIC_DUNDER_METHODS = {
    "__init__", "__repr__", "__eq__", "__hash__", "__setattr__", "__delattr__",
    "__lt__", "__le__", "__gt__", "__ge__", "__enter__", "__exit__",
}
_PYTHON_PUBLIC_DUNDER_FIELDS = {
    "__hash__", "__match_args__", "__slots__", "__dataclass_fields__",
    "__dataclass_params__",
}


def _record(package: str, name: str, kind: str, location: str) -> dict[str, Any]:
    return {"packageId": package, "exportName": name, "kind": kind, "location": location}


def _report(records: list[dict[str, Any]], issues: list[dict[str, str]]) -> dict[str, Any]:
    unique: dict[tuple[str, str], dict[str, Any]] = {}
    for record in records:
        key = record["packageId"], record["exportName"]
        prior = unique.get(key)
        if prior is not None and prior["location"] != record["location"]:
            prior_signatures = set(prior.get("signatures", []))
            record_signatures = set(record.get("signatures", []))
            if (prior["kind"] == record["kind"] == "java-member"
                    and prior_signatures and record_signatures
                    and prior_signatures.isdisjoint(record_signatures)):
                prior["signatures"] = sorted(prior_signatures | record_signatures)
            else:
                issues.append({"packageId": key[0], "exportName": key[1],
                               "reason": "multiple-declarations-share-export-path"})
        unique.setdefault(key, record)
    ordered = sorted(unique.values(), key=lambda r: (r["packageId"], r["exportName"], r["kind"]))
    unique_issues = {(i["packageId"], i["exportName"], i["reason"]): i for i in issues}
    sorted_issues = sorted(unique_issues.values(), key=lambda r: (r["packageId"], r["exportName"], r["reason"]))
    return {"complete": not sorted_issues, "declarations": ordered, "issues": sorted_issues}


def _complete_include_guard(lines: list[str]) -> str | None:
    """Return a conventional guard only when it encloses the whole header."""
    significant = [(index, line.strip()) for index, line in enumerate(lines) if line.strip()]
    if len(significant) < 3:
        return None
    opening = re.fullmatch(r"#\s*ifndef\s+([A-Za-z_]\w*)", significant[0][1])
    if opening is None:
        return None
    guard = opening.group(1)
    definition = re.fullmatch(r"#\s*define\s+([A-Za-z_]\w*)", significant[1][1])
    if definition is None or definition.group(1) != guard:
        return None

    depth = 0
    for line_index, directive in significant:
        if re.match(r"#\s*(?:if|ifdef|ifndef)\b", directive):
            depth += 1
        elif re.match(r"#\s*endif\b", directive):
            depth -= 1
            if depth < 0:
                return None
            if depth == 0:
                return guard if line_index == significant[-1][0] else None
    return None


def inventory_c_header(source: str, package: str = "c-kmipkit") -> dict[str, Any]:
    """Read exported declarations from the installed public C header."""
    records: list[dict[str, Any]] = []
    issues: list[dict[str, str]] = []
    cleaned = re.sub(r"/\*.*?\*/|//[^\n]*", " ", source, flags=re.S)
    lines = cleaned.splitlines()
    include_guard = _complete_include_guard(lines)
    for line_number, line in enumerate(lines, 1):
        macro = re.match(r"\s*#\s*define\s+([A-Za-z_]\w*)\b", line)
        if macro:
            name = macro.group(1)
            if name == include_guard:
                continue
            is_error_constant = name.startswith((
                "KMIPKIT_ERROR_",
                "KMIPKIT_TTLV_DECODE_ERROR_KIND_",
            ))
            records.append(_record(package, name, "constant" if is_error_constant else "macro", str(line_number)))
    parsed_aggregate_spans: set[tuple[int, int]] = set()
    aggregate_pattern = re.compile(r"\btypedef\s+(struct|union)(?:\s+[A-Za-z_]\w*)?\s*\{([^{}]*)\}\s*([A-Za-z_]\w*)\s*;", re.S)

    def record_aggregate_fields(body: str, name: str, line_number: int) -> None:
        for field in body.split(";"):
            field = field.strip()
            if not field:
                continue
            declarators: list[str] = []
            start = 0
            nesting = {"(": 0, "[": 0, "{": 0}
            closing = {")": "(", "]": "[", "}": "{"}
            balanced = True
            for index, character in enumerate(field):
                if character in nesting:
                    nesting[character] += 1
                elif character in closing:
                    opener = closing[character]
                    if nesting[opener] == 0:
                        balanced = False
                        break
                    nesting[opener] -= 1
                elif character == "," and not any(nesting.values()):
                    declarators.append(field[start:index].strip())
                    start = index + 1
            if any(nesting.values()):
                balanced = False
            declarators.append(field[start:].strip())
            field_names = []
            if balanced:
                for declarator in declarators:
                    field_name = re.search(r"([A-Za-z_]\w*)\s*(?::\s*\d+)?\s*(?:\[[^]]*\])?\s*$", declarator)
                    if not field_name or "(" in declarator:
                        field_names = []
                        break
                    field_names.append(field_name.group(1))
            if field_names:
                records.extend(_record(package, f"{name}.{field_name}", "field", str(line_number))
                               for field_name in field_names)
            else:
                issues.append({"packageId": package, "exportName": name, "reason": "unsupported-c-field"})

    for match in aggregate_pattern.finditer(cleaned):
        parsed_aggregate_spans.add((match.start(), match.end()))
        body, name = match.group(2), match.group(3)
        line_number = cleaned.count("\n", 0, match.start()) + 1
        records.append(_record(package, name, "type", str(line_number)))
        record_aggregate_fields(body, name, line_number)

    parsed_tagged_aggregate_spans: set[tuple[int, int]] = set()
    tagged_aggregate_pattern = re.compile(r"\b(?:struct|union)\s+([A-Za-z_]\w*)\s*\{([^{}]*)\}\s*;", re.S)
    for match in tagged_aggregate_pattern.finditer(cleaned):
        parsed_tagged_aggregate_spans.add((match.start(), match.end()))
        name, body = match.group(1), match.group(2)
        line_number = cleaned.count("\n", 0, match.start()) + 1
        if not any(row["exportName"] == name and row["kind"] == "type" for row in records):
            records.append(_record(package, name, "type", str(line_number)))
        record_aggregate_fields(body, name, line_number)
    for match in re.finditer(r"\btypedef\s+(?:struct|union)\b[^;]*\{", cleaned, re.S):
        if not any(start <= match.start() < end for start, end in parsed_aggregate_spans):
            issues.append({"packageId": package, "exportName": str(cleaned.count("\n", 0, match.start()) + 1),
                           "reason": "unsupported-c-type"})
    parsed_enum_spans: set[tuple[int, int]] = set()
    enum_pattern = re.compile(r"\btypedef\s+enum(?:\s+[A-Za-z_]\w*)?\s*\{([^{}]*)\}\s*([A-Za-z_]\w*)\s*;", re.S)
    for match in enum_pattern.finditer(cleaned):
        parsed_enum_spans.add((match.start(), match.end()))
        body, name = match.group(1), match.group(2)
        line_number = cleaned.count("\n", 0, match.start()) + 1
        records.append(_record(package, name, "type", str(line_number)))
        for enumerator in body.split(","):
            enumerator_name = enumerator.split("=", 1)[0].strip()
            if re.fullmatch(r"[A-Za-z_]\w*", enumerator_name):
                records.append(_record(package, enumerator_name, "constant", str(line_number)))
            elif enumerator_name:
                issues.append({"packageId": package, "exportName": name, "reason": "unsupported-c-enumerator"})
    parsed_type_spans = parsed_aggregate_spans | parsed_tagged_aggregate_spans | parsed_enum_spans
    for match in re.finditer(r"\b(?:struct|union|enum)\b[^;{]*\{", cleaned, re.S):
        if not any(start <= match.start() < end for start, end in parsed_type_spans):
            issues.append({"packageId": package, "exportName": str(cleaned.count("\n", 0, match.start()) + 1),
                           "reason": "unsupported-c-type"})
    for match in re.finditer(r"\btypedef\s+(?:struct\s+[A-Za-z_]\w*|[^;{}]+?)\s+([A-Za-z_]\w*)\s*;", cleaned, re.S):
        name = match.group(1)
        line_number = cleaned.count("\n", 0, match.start()) + 1
        if not any(row["exportName"] == name and row["kind"] == "type" for row in records):
            records.append(_record(package, name, "type", str(line_number)))
    for match in re.finditer(r"\b(kmipkit_[A-Za-z_]\w*)\s*\([^;{}]*\)\s*;", cleaned, re.S):
        line_number = cleaned.count("\n", 0, match.start()) + 1
        records.append(_record(package, match.group(1), "c-symbol", str(line_number)))
    return _report(records, issues)


def read_pe_exports(image: bytes) -> list[str]:
    """Read named exports from a PE image without relying on host utilities."""
    def unpack(fmt: str, offset: int) -> tuple[Any, ...]:
        size = struct.calcsize(fmt)
        if offset < 0 or offset + size > len(image):
            raise ValueError("truncated PE image")
        return struct.unpack_from(fmt, image, offset)

    if len(image) < 64 or image[:2] != b"MZ":
        raise ValueError("C ABI library is not a valid PE image")
    pe_offset, = unpack("<I", 0x3C)
    if image[pe_offset:pe_offset + 4] != b"PE\0\0":
        raise ValueError("C ABI library has an invalid PE signature")
    section_count, = unpack("<H", pe_offset + 6)
    optional_size, = unpack("<H", pe_offset + 20)
    optional = pe_offset + 24
    magic, = unpack("<H", optional)
    if magic == 0x10B:
        directory_count_offset, directories_offset = optional + 92, optional + 96
    elif magic == 0x20B:
        directory_count_offset, directories_offset = optional + 108, optional + 112
    else:
        raise ValueError("C ABI library uses an unsupported PE optional header")
    if optional_size < directories_offset - optional + 8:
        raise ValueError("PE optional header does not contain an export directory")
    directory_count, = unpack("<I", directory_count_offset)
    if directory_count == 0:
        return []
    export_rva, export_size = unpack("<II", directories_offset)
    if export_rva == 0 or export_size == 0:
        return []

    section_table = optional + optional_size
    sections: list[tuple[int, int, int, int]] = []
    for index in range(section_count):
        offset = section_table + index * 40
        virtual_size, virtual_address, raw_size, raw_offset = unpack("<IIII", offset + 8)
        sections.append((virtual_address, max(virtual_size, raw_size), raw_size, raw_offset))

    def rva_to_offset(rva: int, length: int = 1) -> int:
        for virtual_address, span, raw_size, raw_offset in sections:
            relative = rva - virtual_address
            if 0 <= relative and relative + length <= span and relative + length <= raw_size:
                return raw_offset + relative
        raise ValueError("PE export address is outside file-backed sections")

    def read_c_string(rva: int) -> str:
        offset = rva_to_offset(rva)
        end = image.find(b"\0", offset, min(len(image), offset + 4096))
        if end < 0:
            raise ValueError("unterminated PE export name")
        try:
            return image[offset:end].decode("ascii")
        except UnicodeDecodeError as error:
            raise ValueError("non-ASCII PE export name") from error

    export_offset = rva_to_offset(export_rva, 40)
    number_of_functions, = unpack("<I", export_offset + 20)
    number_of_names, = unpack("<I", export_offset + 24)
    address_of_names, = unpack("<I", export_offset + 32)
    if number_of_names > 1_000_000:
        raise ValueError("PE export name count exceeds audit limit")
    if number_of_functions != number_of_names:
        raise ValueError("PE export table has ordinal-only exports that cannot be reconciled by name")
    if number_of_names == 0:
        return []
    names_offset = rva_to_offset(address_of_names, number_of_names * 4)
    names = [read_c_string(name_rva) for (name_rva,) in
             (unpack("<I", names_offset + index * 4) for index in range(number_of_names))]
    return sorted(set(names))


def audit_c_surface(header: str, binary_exports: list[str], package: str = "c-kmipkit",
                    approved_exclusions: list[dict[str, Any]] | None = None,
                    blocked_exports: set[str] | None = None) -> dict[str, Any]:
    """Reconcile header declarations with the symbols exported by the ABI library."""
    header_report = inventory_c_header(header, package)
    records = [dict(row) for row in header_report["declarations"]]
    issues = list(header_report["issues"])
    excluded_symbols = {row["exportName"] for row in approved_exclusions or []
                        if row.get("packageId") == package}
    binary_only_blocked = blocked_exports or set()
    header_symbols = {row["exportName"]: row for row in records if row["kind"] == "c-symbol"}
    export_names = set(binary_exports)
    issues.extend({"packageId": package, "exportName": name, "reason": "non-prefixed-c-abi-export"}
                  for name in sorted(export_names) if not name.startswith("kmipkit_") and name not in excluded_symbols)
    for name in sorted(export_names):
        if name in header_symbols:
            header_symbols[name]["location"] += "+binary"
        else:
            records.append(_record(package, name, "c-symbol", "binary"))
            if name not in excluded_symbols and name not in binary_only_blocked:
                issues.append({"packageId": package, "exportName": name, "reason": "exported-symbol-missing-from-header"})
    issues.extend({"packageId": package, "exportName": name, "reason": "header-symbol-not-exported"}
                  for name in sorted(set(header_symbols) - export_names))
    return _report(records, issues)


def read_native_exports(binary: Path) -> list[str]:
    """Read named exports from the freshly built platform ABI library."""
    if os.name == "nt":
        return read_pe_exports(binary.read_bytes())
    nm = shutil.which("nm")
    if nm is None:
        raise ValueError("nm is required to audit the C ABI shared-library exports")
    arguments = [nm, "-gU", str(binary)] if sys.platform == "darwin" else [
        nm, "-D", "--defined-only", "--format=posix", str(binary)]
    result = subprocess.run(arguments, check=False, capture_output=True, text=True, encoding="utf-8")
    if result.returncode != 0:
        raise ValueError(f"nm failed for C ABI library: {result.stderr.strip()}")
    names: set[str] = set()
    for line in result.stdout.splitlines():
        fields = line.split()
        if not fields:
            continue
        name = fields[0] if sys.platform != "darwin" else fields[-1]
        if sys.platform == "darwin" and name.startswith("_"):
            name = name[1:]
        names.add(name.split("@@", 1)[0])
    return sorted(names)


def inventory_javap_outputs(outputs: dict[str, str], package: str = "java-kmipkit") -> dict[str, Any]:
    """Inventory public and protected declarations from deterministic ``javap`` output."""
    records: list[dict[str, str]] = []
    issues: list[dict[str, str]] = []
    for class_file, output in sorted(outputs.items()):
        header = next((line.strip() for line in output.splitlines()
                       if re.search(r"\b(class|interface|enum|record)\s+[A-Za-z_$][\w.$]*", line)), None)
        if header is None:
            issues.append({"packageId": package, "exportName": class_file, "reason": "unrecognized-javap-header"})
            continue
        declaration = re.search(r"^(.*?)\b(?:class|interface|enum|record)\s+([A-Za-z_$][\w.$]*)", header)
        if declaration is None:
            issues.append({"packageId": package, "exportName": class_file, "reason": "unparsed-javap-header"})
            continue
        if not {"public", "protected"}.intersection(declaration.group(1).split()):
            continue
        owner = declaration.group(2).replace("$", ".")
        records.append(_record(package, owner, "type", class_file))
        for line_number, line in enumerate(output.splitlines(), 1):
            text = line.strip()
            if not re.match(r"^(?:public|protected)\b", text) or not text.endswith(";"):
                continue
            signature = text[:-1].strip()
            if "(" in signature:
                before_parameters = signature.split("(", 1)[0].split()
                if not before_parameters:
                    issues.append({"packageId": package, "exportName": owner, "reason": "unparsed-javap-method"})
                    continue
                member = before_parameters[-1]
                if member.replace("$", ".") == owner:
                    member = "<init>"
            else:
                member = signature.split()[-1]
            record = _record(package, f"{owner}.{member}", "java-member", f"{class_file}:{line_number}")
            record["signatures"] = [signature]
            records.append(record)
    return _report(records, issues)


def _literal_all(tree: ast.Module, visible_names: set[str]) -> tuple[list[str] | None, str | None]:
    def evaluate(value: ast.expr) -> list[str] | None:
        if isinstance(value, (ast.List, ast.Tuple)) and all(
            isinstance(item, ast.Constant) and isinstance(item.value, str) for item in value.elts
        ):
            return [item.value for item in value.elts]
        if isinstance(value, ast.BinOp) and isinstance(value.op, ast.Add):
            left, right = evaluate(value.left), evaluate(value.right)
            return left + right if left is not None and right is not None else None
        if isinstance(value, ast.ListComp) and len(value.generators) == 1:
            generator = value.generators[0]
            if not isinstance(generator.target, ast.Name) or not isinstance(value.elt, ast.Name) or value.elt.id != generator.target.id:
                return None
            if not (isinstance(generator.iter, ast.Call) and isinstance(generator.iter.func, ast.Name)
                    and generator.iter.func.id == "globals"):
                return None
            prefixes: set[str] = set()
            for condition in generator.ifs:
                if not (isinstance(condition, ast.Call) and isinstance(condition.func, ast.Attribute)
                        and isinstance(condition.func.value, ast.Name) and condition.func.value.id == generator.target.id
                        and condition.func.attr == "startswith" and len(condition.args) == 1):
                    return None
                argument = condition.args[0]
                if not (isinstance(argument, (ast.List, ast.Tuple)) and all(
                    isinstance(item, ast.Constant) and isinstance(item.value, str) for item in argument.elts
                )):
                    return None
                prefixes.update(item.value for item in argument.elts)
            return sorted(name for name in visible_names if any(name.startswith(prefix) for prefix in prefixes))
        return None

    current: list[str] | None = None
    for statement in tree.body:
        if isinstance(statement, (ast.Assign, ast.AnnAssign)):
            targets = statement.targets if isinstance(statement, ast.Assign) else [statement.target]
            if any(isinstance(target, ast.Name) and target.id == "__all__" for target in targets):
                values = evaluate(statement.value)
                if values is None:
                    return None, "dynamic-all"
                current = values
        elif isinstance(statement, ast.AugAssign) and isinstance(statement.target, ast.Name) and statement.target.id == "__all__":
            if not isinstance(statement.op, ast.Add) or current is None:
                return None, "dynamic-all"
            values = evaluate(statement.value)
            if values is None:
                return None, "dynamic-all"
            current.extend(values)
    return current, None


def _python_class_members(node: ast.ClassDef) -> list[tuple[str, str, int]]:
    """Return the directly declared public members of one Python class."""
    members: list[tuple[str, str, int]] = []
    observed_fields = {
        child.target.id
        for child in node.body
        if isinstance(child, ast.AnnAssign) and isinstance(child.target, ast.Name)
    }
    for child in node.body:
        if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if not child.name.startswith("_") or child.name in _PYTHON_PUBLIC_DUNDER_METHODS:
                members.append((child.name, "python-export", child.lineno))
            if child.name == "__init__":
                for statement in ast.walk(child):
                    if isinstance(statement, ast.Assign):
                        targets = statement.targets
                    elif isinstance(statement, ast.AnnAssign):
                        targets = [statement.target]
                    else:
                        continue
                    for target in targets:
                        if (isinstance(target, ast.Attribute)
                                and isinstance(target.value, ast.Name)
                                and target.value.id == "self"
                                and not target.attr.startswith("_")
                                and target.attr not in observed_fields):
                            members.append((target.attr, "field", statement.lineno))
                            observed_fields.add(target.attr)
        elif isinstance(child, ast.AnnAssign) and isinstance(child.target, ast.Name):
            if not child.target.id.startswith("_") or child.target.id in _PYTHON_PUBLIC_DUNDER_FIELDS:
                members.append((child.target.id, "field", child.lineno))
        elif isinstance(child, ast.Assign):
            members.extend((target.id, "field" if target.id in _PYTHON_PUBLIC_DUNDER_FIELDS else "python-export", child.lineno)
                           for target in child.targets if isinstance(target, ast.Name)
                           and (not target.id.startswith("_") or target.id in _PYTHON_PUBLIC_DUNDER_FIELDS))
    return members


def _python_imported_path(
    expression: ast.expr,
    module: str,
    imported: dict[tuple[str, str], tuple[str, str]],
) -> str | None:
    """Resolve a dotted expression when its root is an imported package name."""
    parts: list[str] = []
    current = expression
    while isinstance(current, ast.Attribute):
        parts.append(current.attr)
        current = current.value
    if not isinstance(current, ast.Name):
        return None
    parts.append(current.id)
    parts.reverse()
    imported_target = imported.get((module, parts[0]))
    if imported_target is None:
        return None
    source_module, source_name = imported_target
    return ".".join(part for part in (source_module, source_name, *parts[1:]) if part)


def _python_dataclass_members(
    node: ast.ClassDef,
    module: str,
    imported: dict[tuple[str, str], tuple[str, str]],
) -> tuple[list[tuple[str, str, int]], str | None, str | None]:
    """Return source-verifiable public declarations created by stdlib dataclass."""
    decorator: ast.Call | ast.expr | None = None
    for candidate in node.decorator_list:
        target = candidate.func if isinstance(candidate, ast.Call) else candidate
        if _python_imported_path(target, module, imported) == "dataclasses.dataclass":
            decorator = candidate
            break
    if decorator is None:
        return [], None, None

    options = {"init": True, "repr": True, "eq": True, "order": False,
               "frozen": False, "unsafe_hash": False, "match_args": True,
               "kw_only": False, "slots": False, "weakref_slot": False}
    if isinstance(decorator, ast.Call):
        for keyword in decorator.keywords:
            if keyword.arg not in options:
                continue
            if not isinstance(keyword.value, ast.Constant) or not isinstance(keyword.value.value, bool):
                return [], "dynamic-dataclass-options", None
            options[keyword.arg] = keyword.value.value

    declared: set[str] = set()
    for child in node.body:
        if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)):
            declared.add(child.name)
        elif isinstance(child, ast.AnnAssign) and isinstance(child.target, ast.Name):
            declared.add(child.target.id)
        elif isinstance(child, ast.Assign):
            declared.update(target.id for target in child.targets if isinstance(target, ast.Name))

    fields: list[tuple[str, str, bool, str | None, bool]] = []
    keyword_only_default = options["kw_only"]
    for child in node.body:
        if not isinstance(child, ast.AnnAssign) or not isinstance(child.target, ast.Name):
            continue
        field_name = child.target.id
        annotation_path = _python_imported_path(child.annotation, module, imported)
        if annotation_path == "dataclasses.KW_ONLY":
            keyword_only_default = True
            continue
        if annotation_path in {"typing.ClassVar", "typing_extensions.ClassVar"}:
            continue
        init = True
        kw_only = keyword_only_default
        default_text: str | None = None
        has_default = child.value is not None
        if isinstance(child.value, ast.Call) and _python_imported_path(child.value.func, module, imported) == "dataclasses.field":
            has_default = False
            for keyword in child.value.keywords:
                if keyword.arg in {"init", "kw_only"}:
                    if not isinstance(keyword.value, ast.Constant) or not isinstance(keyword.value.value, bool):
                        return [], "dynamic-dataclass-options", None
                    if keyword.arg == "init":
                        init = keyword.value.value
                    else:
                        kw_only = keyword.value.value
                elif keyword.arg == "default":
                    has_default = True
                    default_text = ast.unparse(keyword.value)
                elif keyword.arg == "default_factory":
                    has_default = True
                    default_text = "<factory>"
        elif child.value is not None:
            default_text = ast.unparse(child.value)
        fields.append((field_name, ast.unparse(child.annotation), init, default_text if has_default else None,
                       kw_only))

    generated: list[tuple[str, str, int]] = []

    def add_method(name: str) -> None:
        if name not in declared:
            generated.append((name, "python-export", node.lineno))

    def add_field(name: str) -> None:
        if name not in declared:
            generated.append((name, "field", node.lineno))

    signature: str | None = None
    if options["init"]:
        add_method("__init__")
        if "__init__" not in declared:
            parameters: list[str] = []
            keyword_only: list[str] = []
            for field_name, annotation, init, default, kw_only in fields:
                if not init:
                    continue
                parameter = f"{field_name}: {annotation}"
                if default is not None:
                    parameter += f" = {default}"
                (keyword_only if kw_only else parameters).append(parameter)
            if keyword_only:
                parameters.extend(("*", *keyword_only))
            signature = f"({', '.join(parameters)}) -> None"
    if options["repr"]:
        add_method("__repr__")
    if options["eq"]:
        add_method("__eq__")
    if options["order"]:
        for name in ("__lt__", "__le__", "__gt__", "__ge__"):
            add_method(name)
    if options["frozen"]:
        add_method("__setattr__")
        add_method("__delattr__")
    if options["unsafe_hash"] or (options["eq"] and options["frozen"]):
        add_method("__hash__")
    elif options["eq"] and not options["frozen"] and "__hash__" not in declared:
        add_field("__hash__")

    if options["match_args"] and "__match_args__" not in declared:
        add_field("__match_args__")
    if options["slots"]:
        add_field("__slots__")
    add_field("__dataclass_fields__")
    add_field("__dataclass_params__")
    if options["slots"] and options["weakref_slot"]:
        add_field("__weakref__")
    return generated, None, signature


def inventory_python_sources(sources: dict[str, str], package: str = "python-kmipkit") -> dict[str, Any]:
    """Inventory package modules, public definitions and literal ``__all__`` exports."""
    records: list[dict[str, str]] = []
    issues: list[dict[str, str]] = []
    parsed: dict[str, ast.Module] = {}
    public_modules: set[str] = set()
    package_modules: set[str] = set()
    for relative, source in sorted(sources.items()):
        basename = relative.rsplit("/", 1)[-1]
        path_parts = relative.split("/")[:-1]
        if not relative.endswith(".py"):
            continue
        module = relative[:-3].replace("/", ".")
        if module.endswith(".__init__"):
            module = module[:-len(".__init__")]
        if module.endswith("."):
            module = module[:-1]
        if basename == "__init__.py":
            package_modules.add(module)
        try:
            parsed[module] = ast.parse(source, filename=relative)
        except SyntaxError as error:
            issues.append({"packageId": package, "exportName": relative, "reason": f"syntax-error-line-{error.lineno}"})
            continue
        if (basename.startswith("_") and basename != "__init__.py") or any(part.startswith("_") for part in path_parts):
            continue
        public_modules.add(module)
        records.append(_record(package, module, "module", relative))

    definitions: dict[tuple[str, str], str] = {}
    imported: dict[tuple[str, str], tuple[str, str]] = {}
    explicit_exports: dict[str, list[str] | None] = {}
    generated_signatures: dict[str, str] = {}
    classes: dict[str, tuple[str, str, list[ast.expr], list[tuple[str, str, int]]]] = {}
    function_groups: dict[tuple[str, str], list[ast.FunctionDef | ast.AsyncFunctionDef]] = {}
    for module, tree in sorted(parsed.items()):
        for node in tree.body:
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                function_groups.setdefault((module, node.name), []).append(node)
            elif isinstance(node, (ast.ImportFrom, ast.Import)):
                if isinstance(node, ast.ImportFrom) and node.module == "__future__":
                    continue
                for alias in node.names:
                    visible = alias.asname or alias.name
                    if isinstance(node, ast.ImportFrom):
                        if node.level == 0:
                            imported_module = node.module or ""
                        else:
                            package_parts = module.split(".") if module in package_modules else module.rsplit(".", 1)[0].split(".")
                            parent = package_parts[:len(package_parts) - node.level + 1]
                            imported_module = ".".join(parent + ([node.module] if node.module else []))
                        child_module = f"{imported_module}.{alias.name}"
                        if node.module is None and child_module in parsed:
                            imported[(module, visible)] = (child_module, "")
                        else:
                            imported[(module, visible)] = (imported_module, alias.name)
                    else:
                        imported[(module, visible)] = (alias.name, "")

    def is_overload(node: ast.FunctionDef | ast.AsyncFunctionDef) -> bool:
        return any((decorator.id if isinstance(decorator, ast.Name)
                    else decorator.attr if isinstance(decorator, ast.Attribute) else None) == "overload"
                   for decorator in node.decorator_list)

    overload_groups: set[tuple[str, str]] = set()
    for key, group in function_groups.items():
        overload_flags = [is_overload(node) for node in group]
        if len(group) > 1 and any(overload_flags) and all(overload_flags[:-1]) and not overload_flags[-1]:
            overload_groups.add(key)
    recorded_overloads: set[tuple[str, str]] = set()
    for module, tree in sorted(parsed.items()):
        visible_names = {node.name for node in tree.body if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef))}
        visible_names.update(alias.asname or alias.name for node in tree.body if isinstance(node, (ast.Import, ast.ImportFrom)) for alias in node.names)
        exports, issue = _literal_all(tree, visible_names) if module in public_modules else (None, None)
        explicit_exports[module] = exports
        if issue:
            issues.append({"packageId": package, "exportName": module, "reason": issue})
        for node in tree.body:
            if isinstance(node, ast.ClassDef):
                members = _python_class_members(node)
                generated_members, dataclass_issue, constructor_signature = _python_dataclass_members(
                    node, module, imported
                )
                members.extend(generated_members)
                if dataclass_issue:
                    issues.append({"packageId": package, "exportName": f"{module}.{node.name}",
                                   "reason": dataclass_issue})
                if constructor_signature is not None:
                    generated_signatures[f"{module}.{node.name}.__init__"] = constructor_signature
                classes[f"{module}.{node.name}"] = (module, node.name, list(node.bases), members)
                # Private helper modules are not emitted, but their class
                # definitions must remain available for import and inheritance
                # resolution in public modules.
                definitions[(module, node.name)] = "type"
                if module in public_modules and not node.name.startswith("_"):
                    records.append(_record(package, f"{module}.{node.name}", "type", f"{module}:{node.lineno}"))
                    for member, kind, line in members:
                        record = _record(package, f"{module}.{node.name}.{member}", kind, f"{module}:{line}")
                        signature = generated_signatures.get(f"{module}.{node.name}.{member}")
                        if signature is not None:
                            record["signature"] = signature
                        records.append(record)
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and not node.name.startswith("_"):
                kind = "python-export" if exports is not None else "function"
                definitions[(module, node.name)] = kind
                key = (module, node.name)
                if module in public_modules and (key not in overload_groups or key not in recorded_overloads):
                    records.append(_record(package, f"{module}.{node.name}", kind, f"{module}:{node.lineno}"))
                    if key in overload_groups:
                        recorded_overloads.add(key)
            elif isinstance(node, (ast.Assign, ast.AnnAssign)):
                targets = node.targets if isinstance(node, ast.Assign) else [node.target]
                for target in targets:
                    if not isinstance(target, ast.Name) or target.id.startswith("_"):
                        continue
                    definitions[(module, target.id)] = "python-export"
                    if module in public_modules and (exports is None or target.id in exports):
                        records.append(_record(package, f"{module}.{target.id}", "python-export",
                                               f"{module}:{node.lineno}"))
            elif isinstance(node, (ast.ImportFrom, ast.Import)):
                if isinstance(node, ast.ImportFrom) and node.module == "__future__":
                    continue
                for alias in node.names:
                    visible = alias.asname or alias.name
                    if not visible.startswith("_"):
                        definitions[(module, visible)] = "import"
                    if isinstance(node, ast.ImportFrom):
                        if node.level == 0:
                            imported_module = node.module or ""
                        else:
                            package_parts = module.split(".") if module in package_modules else module.rsplit(".", 1)[0].split(".")
                            parent = package_parts[:len(package_parts) - node.level + 1]
                            imported_module = ".".join(parent + ([node.module] if node.module else []))
                        child_module = f"{imported_module}.{alias.name}"
                        if node.module is None and child_module in parsed:
                            imported[(module, visible)] = (child_module, "")
                        else:
                            imported[(module, visible)] = (imported_module, alias.name)
                    else:
                        imported[(module, visible)] = (alias.name, "")

    def resolve_kind(module: str, name: str) -> str | None:
        current = (module, name)
        visited: set[tuple[str, str]] = set()
        while current not in visited:
            visited.add(current)
            kind = definitions.get(current)
            if kind != "import":
                return kind
            target = imported.get(current)
            if target is None:
                return None
            source_module, source_name = target
            if not source_name:
                return "module" if source_module in parsed else None
            if f"{source_module}.{source_name}" in parsed and (source_module, source_name) not in definitions:
                return "module"
            current = (source_module, source_name)
        return None

    def class_path(base: ast.expr, module: str) -> str | None:
        if isinstance(base, ast.Subscript):
            return class_path(base.value, module)
        parts: list[str] = []
        current = base
        while isinstance(current, ast.Attribute):
            parts.append(current.attr)
            current = current.value
        if not isinstance(current, ast.Name):
            return None
        parts.append(current.id)
        parts.reverse()
        imported_base = imported.get((module, parts[0]))
        if imported_base is None:
            return f"{module}.{'.'.join(parts)}"
        source_module, source_name = imported_base
        prefix = ".".join(part for part in (source_module, source_name) if part)
        return ".".join([prefix, *parts[1:]])

    for qualified, (module, name, bases, direct_members) in sorted(classes.items()):
        if module not in public_modules or name.startswith("_"):
            continue
        declared = {member for member, _, _ in direct_members}
        pending = list(bases)
        visited: set[str] = set()
        while pending:
            base = pending.pop(0)
            base_path = class_path(base, module)
            if base_path is None:
                continue
            if base_path in visited:
                continue
            visited.add(base_path)
            base_info = classes.get(base_path)
            if base_info is None:
                continue
            _, _, base_bases, base_members = base_info
            for member, kind, line in base_members:
                if member not in declared and (not member.startswith("_")
                                               or member in {"__init__", "__enter__", "__exit__"}):
                    record = _record(package, f"{qualified}.{member}", kind, f"inherited:{base_path}:{line}")
                    inherited_signature = generated_signatures.get(f"{base_path}.{member}")
                    if inherited_signature is not None:
                        generated_signatures[f"{qualified}.{member}"] = inherited_signature
                        record["signature"] = inherited_signature
                    records.append(record)
                    declared.add(member)
            pending.extend(base_bases)

    for module, names in sorted(explicit_exports.items()):
        if names is None:
            continue
        for name in names:
            if (module, name) not in definitions:
                issues.append({"packageId": package, "exportName": f"{module}.{name}", "reason": "unresolved-python-export"})
    for (module, name), (source_module, source_name) in sorted(imported.items()):
        if module not in public_modules or name.startswith("_"):
            continue
        if explicit_exports.get(module) is not None and name not in (explicit_exports.get(module) or []):
            continue
        target_kind = resolve_kind(module, name)
        export_name = f"{module}.{name}"
        if target_kind is None:
            records.append(_record(package, export_name, "python-export", module))
            issues.append({"packageId": package, "exportName": export_name, "reason": "unresolved-import-target"})
            continue
        source_module, source_name = imported[(module, name)]
        if target_kind == "module" and not source_name and export_name == source_module:
            continue
        records.append(_record(package, export_name, target_kind, module))
    return _report(records, issues)


def compare(report: dict[str, Any], manifest: dict[str, Any], package: str) -> dict[str, Any]:
    package_rows = [row for row in manifest.get("publicDeclarations", []) if row.get("packageId") == package]
    if package == "c-kmipkit":
        # The C header intentionally omits blocked non-symbol declarations.
        # Blocked function symbols remain audited from the native binary ABI.
        package_rows = [
            row for row in package_rows
            if row.get("declarationState") != "blocked" or row.get("kind") == "c-symbol"
        ]
    declared = {(row["packageId"], row["exportName"]): row for row in package_rows}
    duplicate_names = sorted(name for name, count in Counter(row["exportName"] for row in package_rows).items() if count > 1)
    package_exclusions = [row for row in manifest.get("approvedExclusions", [])
                          if row.get("packageId") == package]
    exclusion_counts = Counter((row["packageId"], row["exportName"]) for row in package_exclusions)
    duplicate_exclusions = sorted(key for key, count in exclusion_counts.items() if count > 1)
    exclusions = {(row["packageId"], row["exportName"]): row for row in package_exclusions}
    declaration_exclusion_overlaps = sorted(set(declared) & set(exclusions))
    observed = {(row["packageId"], row["exportName"]): row for row in report["declarations"]}
    missing = sorted(set(observed) - set(declared) - set(exclusions))
    stale = sorted(set(declared) - set(observed))
    def canonical_kind(kind: str) -> str:
        if package == "python-kmipkit" and kind in {"function", "python-export"}:
            return "python-callable"
        return kind

    mismatches = sorted(key for key in set(observed) & set(declared)
                        if canonical_kind(observed[key]["kind"]) != canonical_kind(declared[key]["kind"]))
    path_matches = set(observed) & set(declared)
    signature_issues: list[dict[str, str]] = []
    if package == "java-kmipkit":
        for key in sorted(path_matches):
            observed_signatures = observed[key].get("signatures")
            declared_signatures = declared[key].get("javaSignatures")
            if not observed_signatures:
                continue
            if len(observed_signatures) > 1 and declared_signatures is None:
                signature_issues.append({
                    "packageId": package,
                    "exportName": key[1],
                    "reason": "java-overload-signatures-not-declared",
                })
            elif declared_signatures is not None and sorted(observed_signatures) != sorted(declared_signatures):
                signature_issues.append({
                    "packageId": package,
                    "exportName": key[1],
                    "reason": "java-signature-mismatch",
                })
    exact_matches = len(path_matches) - len(mismatches) - len(signature_issues)
    issues = list(report["issues"])
    issues.extend(signature_issues)
    issues.extend({"packageId": package, "exportName": name, "reason": "duplicate-manifest-export-name"}
                  for name in duplicate_names)
    issues.extend({"packageId": key[0], "exportName": key[1],
                   "reason": "duplicate-manifest-approved-exclusion"}
                  for key in duplicate_exclusions)
    issues.extend({"packageId": key[0], "exportName": key[1],
                   "reason": "manifest-export-also-excluded"}
                  for key in declaration_exclusion_overlaps)
    invalid_identities = bool(duplicate_names or duplicate_exclusions or declaration_exclusion_overlaps)
    return {"packageId": package, "complete": report["complete"] and not invalid_identities and not signature_issues,
            "matchesManifest": not missing and not stale and not mismatches and not invalid_identities
            and not signature_issues and report["complete"],
            "counts": {"observed": len(observed), "manifest": len(package_rows), "matched": exact_matches,
                       "pathMatches": len(path_matches), "exactMatches": exact_matches,
                       "missingFromManifest": len(missing), "notObserved": len(stale), "kindMismatches": len(mismatches)},
            "missingFromManifest": [observed[k] for k in missing],
            "notObserved": [declared[k] for k in stale],
            "kindMismatches": [{"observed": observed[k], "manifest": declared[k]} for k in mismatches],
            "issues": sorted(issues, key=lambda row: (row["packageId"], row["exportName"], row["reason"]))}


def build_c_binary_exports(root: Path) -> list[str]:
    """Build the C cdylib and read its exports before the temporary directory is removed."""
    with tempfile.TemporaryDirectory(prefix="kmipkit-api-cargo-") as temp_output:
        target_root = Path(temp_output) / "target"
        build_result = subprocess.run(["cargo", "build", "--locked", "--offline", "--package", "kmipkit-ffi",
                                       "--lib", "--target-dir", str(target_root)], cwd=root,
                                      check=False, capture_output=True, text=True, encoding="utf-8")
        if build_result.returncode != 0:
            raise ValueError(f"cargo build for C ABI audit failed: {build_result.stderr.strip()}")
        suffix = ".dll" if os.name == "nt" else ".dylib" if sys.platform == "darwin" else ".so"
        binaries = sorted(path for path in (target_root / "debug").glob(f"*kmipkit_ffi{suffix}") if path.is_file())
        if not binaries:
            raise ValueError("cargo build completed without producing the kmipkit_ffi shared library")
        return read_native_exports(binaries[0])


def audit_c_package(root: Path, manifest: dict[str, Any], binary: Path | None = None) -> dict[str, Any]:
    c_header = (root / "bindings/c/include/kmipkit.h").read_text(encoding="utf-8")
    native_exports = read_native_exports(binary) if binary is not None else build_c_binary_exports(root)
    c_exclusions = [row for row in manifest.get("approvedExclusions", [])
                    if row.get("packageId") == "c-kmipkit"]
    blocked_c_exports = {
        row["exportName"]
        for row in manifest.get("publicDeclarations", [])
        if row.get("packageId") == "c-kmipkit"
        and row.get("kind") == "c-symbol"
        and row.get("declarationState") == "blocked"
    }
    c = audit_c_surface(
        c_header,
        native_exports,
        approved_exclusions=c_exclusions,
        blocked_exports=blocked_c_exports,
    )
    return compare(c, manifest, "c-kmipkit")


def resolve_java_compile_classpath(root: Path) -> str:
    """Resolve Java's configured compile classpath through offline Gradle."""
    java_root = root / "bindings/java"
    wrapper_name = "gradlew.bat" if os.name == "nt" else "gradlew"
    wrapper = java_root / wrapper_name
    if not wrapper.is_file():
        raise ValueError(f"Gradle wrapper is missing: {wrapper}")
    command = [str(wrapper), "--offline", "--console=plain", "--quiet", "apiAuditClasspath"]
    if os.name == "nt":
        command = [os.environ.get("COMSPEC", "cmd.exe"), "/d", "/s", "/c",
                   subprocess.list2cmdline(command)]
    result = subprocess.run(command, cwd=java_root, check=False, capture_output=True, text=True,
                            encoding="utf-8")
    if result.returncode != 0:
        raise ValueError(f"Gradle could not resolve the offline Java compile classpath: {result.stderr.strip()}")
    marker = "KMIPKIT_API_AUDIT_CLASSPATH="
    classpath_lines = [line[len(marker):] for line in result.stdout.splitlines() if line.startswith(marker)]
    if len(classpath_lines) != 1 or not classpath_lines[0]:
        raise ValueError("Gradle did not emit exactly one Java API audit classpath")
    return classpath_lines[0]


def run(root: Path, *, c_only: bool = False, c_binary: Path | None = None) -> dict[str, Any]:
    manifest = json.loads((root / "specification/api/public-api.json").read_text(encoding="utf-8"))
    packages = [audit_c_package(root, manifest, c_binary)]
    if c_only:
        return {"formatVersion": 2, "packages": packages}
    java_root = root / "bindings/java/src/main/java"
    java_sources = sorted(java_root.rglob("*.java"))
    if not java_sources:
        raise ValueError("Java sources are missing")
    java_classpath = resolve_java_compile_classpath(root)
    java_outputs: dict[str, str] = {}
    with tempfile.TemporaryDirectory(prefix="kmipkit-api-javac-") as temp_output:
        compile_result = subprocess.run(["javac", "--release", "17", "-classpath", java_classpath,
                                         "-d", temp_output,
                                         *(str(path) for path in java_sources)],
                                        check=False, capture_output=True, text=True, encoding="utf-8")
        if compile_result.returncode != 0:
            raise ValueError(f"javac --release 17 failed: {compile_result.stderr.strip()}")
        java_classes = Path(temp_output)
        for class_file in sorted(java_classes.rglob("*.class")):
            binary_name = class_file.relative_to(java_classes).with_suffix("").as_posix().replace("/", ".")
            if binary_name.endswith("module-info") or binary_name.endswith("package-info"):
                continue
            result = subprocess.run(["javap", "-protected", "-classpath", str(java_classes), binary_name],
                                    check=False, capture_output=True, text=True, encoding="utf-8")
            if result.returncode != 0:
                raise ValueError(f"javap failed for {binary_name}: {result.stderr.strip()}")
            java_outputs[class_file.relative_to(java_classes).as_posix()] = result.stdout
    java = inventory_javap_outputs(java_outputs)
    python_root = root / "bindings/python/src/kmipkit"
    python_sources = {path.relative_to(python_root.parent).as_posix(): path.read_text(encoding="utf-8")
                      for path in sorted(python_root.rglob("*.py"))}
    python = inventory_python_sources(python_sources)
    packages.extend([compare(java, manifest, "java-kmipkit"), compare(python, manifest, "python-kmipkit")])
    return {"formatVersion": 2, "packages": packages}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path)
    parser.add_argument("--c-only", action="store_true",
                        help="audit only C and optionally reuse a previously built native library")
    parser.add_argument("--c-binary", type=Path,
                        help="read C ABI exports from this library instead of building a temporary copy")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        if args.c_binary is not None and not args.c_only:
            parser.error("--c-binary requires --c-only")
        result = run(args.repo_root.resolve(), c_only=args.c_only,
                     c_binary=args.c_binary.resolve(strict=True) if args.c_binary is not None else None)
        data = json.dumps(result, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
        if args.report:
            args.report.write_text(data, encoding="utf-8", newline="\n")
        else:
            print(data, end="")
        return 0 if not args.check or all(p["matchesManifest"] for p in result["packages"]) else 1
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(f"Binding export audit failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
