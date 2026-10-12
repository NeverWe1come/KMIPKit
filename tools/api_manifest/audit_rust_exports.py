"""Audit public Rust declaration paths against manifest v2, without changing it.

The runner uses the existing Rust 1.94.0 toolchain, locked offline dependencies,
and rustdoc JSON format 57. Trait implementations and type aliases are expanded
only when rustdoc supplies exact owner, trait, and target identities. Unknown
targets stay as explicit issues, never as a complete inventory. See
``rust-export-audit.md``.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


TOOLCHAIN = "1.94.0"
FORMAT_VERSION = 57
_UNRESOLVED_IDENTITY = object()
_UNRESOLVED_LIFETIME = object()
PACKAGES = (
    "kmipkit-ttlv", "kmipkit-protocol", "kmipkit-transport", "kmipkit-client", "kmipkit",
)
KINDS = {
    "module": "module", "struct": "type", "enum": "type", "union": "type",
    "type_alias": "type", "trait": "trait", "function": "function",
    "constant": "constant", "static": "constant", "macro": "macro", "proc_macro": "macro",
    "variant": "variant", "struct_field": "field",
    "assoc_type": "associated-item", "assoc_const": "associated-item",
}


def _kind(item: dict[str, Any]) -> str:
    return next(iter(item["inner"]))


def _location(item: dict[str, Any]) -> dict[str, Any] | None:
    span = item.get("span")
    if span is None:
        return None
    return {
        "path": span["filename"].replace("\\", "/"),
        "line": span["begin"][0], "column": span["begin"][1],
    }


def _key(record: dict[str, Any]) -> tuple[str, str]:
    return record["packageId"], record["exportName"]


def _canonical_export_name(name: str) -> str:
    """Normalize lifetime elision and field separators in a Rust declaration path."""
    def split_arguments(value: str) -> list[str]:
        arguments: list[str] = []
        depth = 0
        start = 0
        for index, character in enumerate(value):
            if character == "<":
                depth += 1
            elif character == ">":
                depth -= 1
            elif character == "," and depth == 0:
                arguments.append(value[start:index].strip())
                start = index + 1
        arguments.append(value[start:].strip())
        return arguments

    def normalize(value: str) -> str:
        output: list[str] = []
        index = 0
        while index < len(value):
            if value[index] != "<":
                output.append(value[index])
                index += 1
                continue
            depth = 1
            end = index + 1
            while end < len(value) and depth:
                if value[end] == "<":
                    depth += 1
                elif value[end] == ">":
                    depth -= 1
                end += 1
            if depth:
                output.append(value[index:])
                break
            arguments = []
            for argument in split_arguments(value[index + 1:end - 1]):
                if re.fullmatch(r"'[A-Za-z_][A-Za-z0-9_]*", argument) and argument != "'static":
                    continue
                normalized = normalize(argument)
                normalized = re.sub(r"(?<![A-Za-z0-9_])'([A-Za-z_][A-Za-z0-9_]*)",
                                    lambda match: match.group(0) if match.group(1) == "static" else "'_",
                                    normalized)
                arguments.append(normalized)
            if arguments:
                output.append("<" + ", ".join(arguments) + ">")
            index = end
        return "".join(output).replace(".", "::")

    return normalize(name)


def _comparison_key(record: dict[str, Any]) -> tuple[str, str]:
    return record["packageId"], _canonical_export_name(record["exportName"])


class _Inventory:
    def __init__(self, documents: dict[str, dict[str, Any]]) -> None:
        self.documents = documents
        self.records: dict[tuple[str, str], dict[str, Any]] = {}
        self.conditional_records: dict[tuple[str, str], dict[str, Any]] = {}
        self.issues: list[dict[str, Any]] = []
        self.definitions: dict[tuple[str, ...], tuple[str, str]] = {}
        self.member_origins: dict[tuple[str, str], tuple[str, ...]] = {}
        self.reachable_items: set[tuple[str, str]] = set()
        self.reachability_prepass = False
        self._trait_proof_stack: set[tuple[tuple[str, ...], str]] = set()
        for crate, doc in sorted(documents.items()):
            if doc.get("format_version") != FORMAT_VERSION or doc.get("includes_private") is not True:
                raise ValueError(f"{crate}: expected rustdoc format {FORMAT_VERSION} with private items")
            root = doc["index"][str(doc["root"])]
            if root["name"] != crate or _kind(root) != "module":
                raise ValueError(f"{crate}: unexpected rustdoc root")
            self.definitions[(crate,)] = crate, str(doc["root"])
            for identifier, path in doc["paths"].items():
                if identifier in doc["index"] and doc["index"][identifier].get("crate_id") == 0:
                    self.definitions[tuple(path["path"])] = crate, identifier
            for identifier, owner in doc["index"].items():
                if owner.get("crate_id") != 0 or _kind(owner) not in {"struct", "union", "enum", "trait"}:
                    continue
                canonical = tuple(doc["paths"].get(identifier, {}).get("path", [crate, owner["name"]]))
                self.index_member_origins(crate, owner, canonical)

    def index_member_origins(self, crate: str, owner: dict[str, Any], canonical: tuple[str, ...]) -> None:
        """Keep macro-expanded members distinct even when their source spans coincide."""
        kind = _kind(owner)
        inner = owner["inner"][kind]
        children = []
        if kind in {"struct", "variant"}:
            children = list(self.fields(inner["kind"]))
        elif kind == "union":
            children = list(inner["fields"])
        elif kind == "enum":
            children = list(inner["variants"])
        elif kind == "trait":
            children = list(inner["items"])
        for impl_id in inner.get("impls", []):
            impl_item = self.documents[crate]["index"].get(str(impl_id))
            if impl_item is not None:
                children.extend(impl_item.get("inner", {}).get("impl", {}).get("items", []))
        for identifier in children:
            if identifier is None:
                continue
            member = self.documents[crate]["index"].get(str(identifier))
            if member is None:
                continue
            origin = (*canonical, member["name"])
            self.member_origins[(crate, str(identifier))] = origin
            if _kind(member) == "variant":
                self.index_member_origins(crate, member, origin)

    def issue(self, package: str, name: str, reason: str, item: dict[str, Any], **details: Any) -> None:
        self.issues.append({
            "packageId": package, "exportName": name, "reason": reason,
            "location": _location(item), **details,
        })

    def resolve(self, crate: str, identifier: Any) -> tuple[str, str] | None:
        doc = self.documents[crate]
        identifier = str(identifier)
        local = doc["index"].get(identifier)
        if local is not None and local.get("crate_id") == 0:
            return crate, identifier
        path = doc["paths"].get(identifier)
        return None if path is None else self.definitions.get(tuple(path["path"]))

    def add(self, crate: str, item: dict[str, Any], package: str, path: tuple[str, ...],
            exports: tuple[dict[str, Any] | None, ...], associated: bool = False,
            identity_components: list[Any] | None = None) -> None:
        kind = _kind(item)
        location = _location(item)
        origin_path = self.member_origins.get((crate, str(item["id"])))
        if origin_path is None:
            origin_path = self.documents[crate]["paths"].get(str(item["id"]), {}).get("path", [crate, item["name"]])
        identity = json.dumps(
            identity_components if identity_components is not None else [crate, location, kind, origin_path],
            sort_keys=True, separators=(",", ":"),
        )
        name = "::".join(path)
        record = {
            "packageId": package, "exportName": name,
            "kind": "associated-item" if associated else KINDS[kind],
            "identity": identity, "location": location,
            "exportLocations": [e for e in exports if e is not None],
        }
        previous = self.records.get((package, name))
        if previous is not None:
            if previous["identity"] != identity:
                self.issue(package, name, "multiple-definitions-for-export-path", item)
            previous["exportLocations"].extend(record["exportLocations"])
        else:
            self.records[(package, name)] = record

    def children(self, crate: str, ids: list[Any], package: str, path: tuple[str, ...],
                 ancestors: frozenset[tuple[str, str]], exports: tuple[dict[str, Any] | None, ...],
                 default_public: bool = False, associated: bool = False) -> None:
        for identifier in ids:
            if identifier is None:
                continue
            item = self.documents[crate]["index"].get(str(identifier))
            if item is None:
                self.issue(package, "::".join(path), "missing-rustdoc-item", {"span": None}, itemId=str(identifier))
                continue
            if item["visibility"] == "public" or (default_public and item["visibility"] == "default"):
                self.visit(crate, str(identifier), package, path, ancestors, exports, associated=associated)

    def visit(self, crate: str, identifier: str, package: str, parent: tuple[str, ...],
              ancestors: frozenset[tuple[str, str]], exports: tuple[dict[str, Any] | None, ...],
              alias: str | None = None, associated: bool = False) -> None:
        item = self.documents[crate]["index"][identifier]
        kind = _kind(item)
        inner = item["inner"][kind]
        if kind == "use":
            target = self.resolve(crate, inner.get("id"))
            name = "::".join((*parent, inner["name"]))
            if target is None:
                self.issue(package, name, "unresolved-reexport", item)
                return
            target_crate, target_id = target
            target_item = self.documents[target_crate]["index"][target_id]
            export_sites = (*exports, _location(item))
            if inner["is_glob"]:
                if (target_crate, target_id) in ancestors:
                    return
                target_kind = _kind(target_item)
                if target_kind not in {"module", "enum"}:
                    self.issue(package, name, "unsupported-glob-target", item)
                    return
                values = target_item["inner"][target_kind]
                self.children(target_crate, values["items" if target_kind == "module" else "variants"],
                              package, parent, ancestors | {(target_crate, target_id)}, export_sites,
                              default_public=target_kind == "enum")
            else:
                self.visit(target_crate, target_id, package, parent, ancestors, export_sites, alias=inner["name"])
            return
        path = (*parent, alias if alias is not None else item["name"])
        if kind not in KINDS:
            self.issue(package, "::".join(path), "unsupported-item-kind", item, itemKind=kind)
            return
        self.reachable_items.add((crate, identifier))
        if not self.reachability_prepass:
            self.add(crate, item, package, path, exports, associated)
        if (crate, identifier) in ancestors:
            return
        ancestors = ancestors | {(crate, identifier)}
        if kind == "module":
            self.children(crate, inner["items"], package, path, ancestors, exports)
        elif kind in {"struct", "union"}:
            fields = inner["fields"] if kind == "union" else self.fields(inner["kind"])
            self.children(crate, fields, package, path, ancestors, exports)
        elif kind == "enum":
            self.children(crate, inner["variants"], package, path, ancestors, exports, default_public=True)
        elif kind == "variant":
            self.children(crate, self.fields(inner["kind"]), package, path, ancestors, exports, default_public=True)
        elif kind == "trait":
            self.children(crate, inner["items"], package, path, ancestors, exports, default_public=True, associated=True)
        elif kind == "type_alias":
            if not self.reachability_prepass:
                self.expand_type_alias(crate, item, package, path, ancestors, exports)
        if isinstance(inner, dict) and not self.reachability_prepass:
            owner_type = self.type_expression(crate, item) if kind in {"struct", "union", "enum"} else None
            owner_display = "::".join(path)
            if owner_type is not None:
                parameters = self.generic_parameters(inner.get("generics", {}))
                parameter_names = [parameter.get("name") for parameter in parameters if parameter.get("name")]
                if parameter_names:
                    owner_display += f"<{', '.join(parameter_names)}>"
            self.implementation_members(crate, inner.get("impls", []), package, path, ancestors, exports, item,
                                        owner_type=owner_type, owner_display=owner_display)

    def expand_type_alias(self, crate: str, alias: dict[str, Any], package: str,
                          path: tuple[str, ...], ancestors: frozenset[tuple[str, str]],
                          exports: tuple[dict[str, Any] | None, ...]) -> None:
        """Expose members reachable through an exact, locally documented alias target."""
        alias_chain: list[tuple[str, str]] = [(crate, str(alias["id"]))]
        target_crate = crate
        target_id: str | None = None
        alias_parameters = self.generic_parameters(alias["inner"]["type_alias"].get("generics", {}))
        alias_arguments = self.parameter_arguments(alias_parameters)
        alias_bindings = {
            parameter["name"]: argument["type"] if "type" in argument else argument
            for parameter, argument in zip(alias_parameters, alias_arguments, strict=True)
            if parameter.get("name")
        }
        alias_display = "::".join(path)
        if alias_arguments:
            formatted_alias_arguments = [
                self.format_type(crate, argument["type"]) if "type" in argument else
                str(argument.get("lifetime", argument.get("const", {}).get("value", "")))
                for argument in alias_arguments
            ]
            if any(argument is None for argument in formatted_alias_arguments):
                self.issue(package, alias_display, "unsupported-type-alias-parameters", alias,
                           aliasParameters=[parameter.get("name") for parameter in alias_parameters])
                return
            alias_display += f"<{', '.join(formatted_alias_arguments)}>"

        target_inner = alias["inner"]["type_alias"]["type"]
        target_path: list[str] | None = None
        final_expression: Any = None
        while isinstance(target_inner, dict) and "resolved_path" in target_inner:
            current_expression = self.substitute_type(target_inner, alias_bindings)
            resolved = current_expression["resolved_path"]
            target_path = resolved.get("path", "").split("::") if resolved.get("path") else None
            target = self.resolve(target_crate, resolved.get("id"))
            if target is None:
                self.issue(package, "::".join(path), "unresolved-type-alias-target", alias,
                           targetId=str(resolved.get("id")), targetPath=target_path,
                           aliasChain=[list(self.definition_path(c, i)) for c, i in alias_chain])
                return
            target_crate, target_id = target
            target_key = (target_crate, target_id)
            if target_key in alias_chain:
                self.issue(package, "::".join(path), "cyclic-type-alias", alias,
                           targetPath=target_path,
                           aliasChain=[list(self.definition_path(c, i)) for c, i in alias_chain])
                return
            target_item = self.documents[target_crate]["index"][target_id]
            if _kind(target_item) != "type_alias":
                final_expression = current_expression
                break
            alias_chain.append(target_key)
            next_parameters = self.generic_parameters(target_item["inner"]["type_alias"].get("generics", {}))
            next_arguments = self.parameter_arguments(next_parameters)
            supplied_arguments = resolved.get("args")
            actual_arguments = supplied_arguments.get("angle_bracketed", {}).get("args", []) \
                if supplied_arguments is not None else []
            if len(actual_arguments) != len(next_parameters):
                self.issue(package, "::".join(path), "unresolved-type-alias-arguments", alias,
                           targetId=target_id, targetPath=target_path,
                           expectedArgumentCount=len(next_parameters), actualArgumentCount=len(actual_arguments),
                           aliasChain=[list(self.definition_path(c, i)) for c, i in alias_chain])
                return
            alias_bindings = {
                parameter["name"]: self.substitute_type(argument.get("type", argument), alias_bindings)
                for parameter, argument in zip(next_parameters, actual_arguments, strict=True)
                if parameter.get("name")
            }
            target_inner = target_item["inner"]["type_alias"]["type"]
        else:
            self.issue(package, "::".join(path), "unsupported-type-alias-target", alias,
                       targetPath=target_path,
                       aliasChain=[list(self.definition_path(c, i)) for c, i in alias_chain])
            return

        if target_id is None:
            self.issue(package, "::".join(path), "unresolved-type-alias-target", alias,
                       targetPath=target_path,
                       aliasChain=[list(self.definition_path(c, i)) for c, i in alias_chain])
            return
        concrete = self.documents[target_crate]["index"][target_id]
        concrete_kind = _kind(concrete)
        concrete_inner = concrete["inner"][concrete_kind]
        alias_ancestors = ancestors | set(alias_chain) | {(target_crate, target_id)}
        if concrete_kind in {"struct", "union"}:
            fields = concrete_inner["fields"] if concrete_kind == "union" else self.fields(concrete_inner["kind"])
            self.children(target_crate, fields, package, path, frozenset(alias_ancestors), exports)
        elif concrete_kind == "enum":
            self.children(target_crate, concrete_inner["variants"], package, path,
                          frozenset(alias_ancestors), exports, default_public=True)
        elif concrete_kind not in {"struct", "union", "enum"}:
            self.issue(package, "::".join(path), "unsupported-type-alias-target-kind", alias,
                       targetKind=concrete_kind, targetPath=list(self.definition_path(target_crate, target_id)),
                       aliasChain=[list(self.definition_path(c, i)) for c, i in alias_chain])
            return
        self.implementation_members(target_crate, concrete_inner.get("impls", []), package, path,
                                    frozenset(alias_ancestors), exports, concrete,
                                    owner_type=final_expression, owner_display=alias_display,
                                    filter_to_owner=True)

    def definition_path(self, crate: str, identifier: str) -> tuple[str, ...]:
        """Return rustdoc's canonical path for an item, including external items."""
        doc = self.documents[crate]
        path = doc["paths"].get(str(identifier))
        if path is not None:
            return tuple(path["path"])
        item = doc["index"].get(str(identifier))
        if item is not None and item.get("name") is not None:
            return crate, item["name"]
        return crate, f"<unknown:{identifier}>"

    @staticmethod
    def generic_parameters(generics: dict[str, Any]) -> list[dict[str, Any]]:
        return list(generics.get("params", []))

    @staticmethod
    def parameter_arguments(parameters: list[dict[str, Any]]) -> list[dict[str, Any]]:
        arguments = []
        for parameter in parameters:
            kind = parameter.get("kind", {})
            name = parameter.get("name")
            if not name:
                continue
            if "type" in kind:
                arguments.append({"type": {"generic": name}})
            elif "lifetime" in kind:
                arguments.append({"lifetime": name})
            elif "const" in kind:
                arguments.append({"const": {"value": name}})
        return arguments

    def type_expression(self, crate: str, item: dict[str, Any]) -> dict[str, Any]:
        kind = _kind(item)
        generics = item["inner"][kind].get("generics", {})
        arguments = self.parameter_arguments(self.generic_parameters(generics))
        resolved = {"id": item["id"], "path": "::".join(self.definition_path(crate, str(item["id"]))) }
        if arguments:
            resolved["args"] = {"angle_bracketed": {"args": arguments, "constraints": []}}
        else:
            resolved["args"] = None
        return {"resolved_path": resolved}

    def format_type(self, crate: str, expression: Any) -> str | None:
        """Render the rustdoc type forms used in public owner and trait identities."""
        if not isinstance(expression, dict):
            return None
        if "primitive" in expression:
            return str(expression["primitive"])
        if "generic" in expression:
            return str(expression["generic"])
        if "resolved_path" in expression:
            value = expression["resolved_path"]
            target = self.resolve(crate, value.get("id")) if value.get("id") is not None else None
            if target is not None:
                base = "::".join(self.definition_path(*target))
            else:
                base = str(value.get("path", ""))
            if not base:
                return None
            raw_args = value.get("args")
            if raw_args is None:
                return base
            angle = raw_args.get("angle_bracketed")
            if angle is None:
                return None
            rendered = []
            for argument in angle.get("args", []):
                if "type" in argument:
                    value_text = self.format_type(crate, argument["type"])
                elif "lifetime" in argument:
                    value_text = str(argument["lifetime"])
                elif "const" in argument:
                    const_value = argument["const"].get("value")
                    value_text = None if const_value is None else str(const_value)
                else:
                    value_text = None
                if value_text is None:
                    return None
                rendered.append(value_text)
            if not rendered:
                return base
            return f"{base}<{', '.join(rendered)}>"
        if "tuple" in expression:
            values = [self.format_type(crate, element) for element in expression["tuple"]]
            if any(value is None for value in values):
                return None
            if len(values) == 1:
                return f"({values[0]},)"
            return f"({', '.join(values)})"
        if "slice" in expression:
            inner = self.format_type(crate, expression["slice"])
            return None if inner is None else f"[{inner}]"
        if "borrowed_ref" in expression:
            value = expression["borrowed_ref"]
            inner = self.format_type(crate, value.get("type"))
            if inner is None:
                return None
            lifetime = value.get("lifetime")
            mutable = "mut " if value.get("is_mutable") else ""
            lifetime_prefix = f"{lifetime} " if lifetime else ""
            return f"&{lifetime_prefix}{mutable}{inner}"
        if "raw_pointer" in expression:
            value = expression["raw_pointer"]
            inner = self.format_type(crate, value.get("type"))
            if inner is None:
                return None
            pointer = "mut" if value.get("is_mutable") else "const"
            return f"*{pointer} {inner}"
        return None

    @classmethod
    def substitute_type(cls, expression: Any, bindings: dict[str, Any]) -> Any:
        if isinstance(expression, list):
            return [cls.substitute_type(value, bindings) for value in expression]
        if isinstance(expression, str) and expression.startswith("'") and expression in bindings:
            bound_lifetime = bindings[expression]
            if isinstance(bound_lifetime, dict) and set(bound_lifetime) == {"lifetime"}:
                return bound_lifetime["lifetime"]
        if not isinstance(expression, dict):
            return expression
        if set(expression) == {"generic"} and expression["generic"] in bindings:
            return bindings[expression["generic"]]
        if set(expression) == {"lifetime"} and isinstance(expression["lifetime"], str) \
                and expression["lifetime"] in bindings:
            return bindings[expression["lifetime"]]
        return {key: cls.substitute_type(value, bindings) for key, value in expression.items()}

    @classmethod
    def substitute_lifetime(cls, lifetime: str, bindings: dict[str, Any]) -> Any:
        """Resolve a lifetime through generic bindings when rustdoc provides one."""
        resolved = cls.substitute_type({"lifetime": lifetime}, bindings)
        if isinstance(resolved, dict) and set(resolved) == {"lifetime"}:
            return resolved["lifetime"]
        return _UNRESOLVED_LIFETIME

    @staticmethod
    def generic_parameter_names(generics: dict[str, Any]) -> set[str]:
        return {parameter["name"] for parameter in generics.get("params", []) if parameter.get("name")}

    def unify_type(self, crate: str, pattern: Any, actual: Any, variables: set[str],
                   bindings: dict[str, Any]) -> bool | None:
        if isinstance(pattern, str) and pattern.startswith("'"):
            if pattern == "'_":
                return isinstance(actual, str) and actual.startswith("'")
            if pattern in variables:
                if not isinstance(actual, str) or not actual.startswith("'"):
                    return False
                previous = bindings.get(pattern)
                if previous is None:
                    bindings[pattern] = {"lifetime": actual}
                    return True
                return isinstance(previous, dict) and previous.get("lifetime") == actual
        if isinstance(pattern, dict) and pattern == {"lifetime": "'_"}:
            # rustdoc encodes an elided lifetime in an impl target as an inferred
            # binder that can match the owner's named lifetime parameter.
            return isinstance(actual, dict) and set(actual) == {"lifetime"}
        if isinstance(pattern, dict) and set(pattern) == {"lifetime"} \
                and isinstance(pattern["lifetime"], str) and pattern["lifetime"] in variables:
            name = pattern["lifetime"]
            previous = bindings.get(name)
            if previous is None:
                if not isinstance(actual, dict) or set(actual) != {"lifetime"}:
                    return False
                bindings[name] = actual
                return True
            return self.unify_type(crate, previous, actual, set(), {})
        if isinstance(pattern, dict) and set(pattern) == {"generic"} and pattern["generic"] in variables:
            name = pattern["generic"]
            previous = bindings.get(name)
            if previous is None:
                bindings[name] = actual
                return True
            return self.unify_type(crate, previous, actual, set(), {})
        if isinstance(pattern, list) or isinstance(actual, list):
            if not isinstance(pattern, list) or not isinstance(actual, list):
                return False
            if len(pattern) != len(actual):
                return False
            results = [self.unify_type(crate, left, right, variables, bindings)
                       for left, right in zip(pattern, actual, strict=True)]
            return None if None in results else all(results)
        if isinstance(pattern, dict) or isinstance(actual, dict):
            if not isinstance(pattern, dict) or not isinstance(actual, dict) or set(pattern) != set(actual):
                return False
            if set(pattern) == {"resolved_path"}:
                return self.unify_resolved_path(crate, pattern["resolved_path"], actual["resolved_path"],
                                                variables, bindings)
            type_forms = {
                "primitive", "generic", "tuple", "slice", "borrowed_ref", "raw_pointer",
                "qualified_path", "function_pointer", "dyn_trait", "impl_trait", "infer", "never",
                "type", "lifetime", "const", "angle_bracketed",
            }
            if len(pattern) == 1 and next(iter(pattern)) not in type_forms:
                return None
            results = [self.unify_type(crate, pattern[key], actual[key], variables, bindings)
                       for key in pattern]
            return None if None in results else all(results)
        return pattern == actual

    def unify_resolved_path(self, crate: str, first: dict[str, Any], second: dict[str, Any],
                            variables: set[str], bindings: dict[str, Any]) -> bool | None:
        first_target = self.resolve(crate, first.get("id")) if first.get("id") is not None else None
        second_target = self.resolve(crate, second.get("id")) if second.get("id") is not None else None
        first_path = self.definition_path(*first_target) if first_target is not None else tuple(
            str(first.get("path", "")).split("::"))
        second_path = self.definition_path(*second_target) if second_target is not None else tuple(
            str(second.get("path", "")).split("::"))
        if not first_path or not second_path:
            return None
        if first_path != second_path:
            return False
        first_args = first.get("args")
        second_args = second.get("args")
        return self.unify_type(crate, first_args, second_args, variables, bindings)

    def check_implementation_predicates(self, crate: str, impl: dict[str, Any],
                                        bindings: dict[str, Any]) -> bool | None:
        """Check concrete predicates needed to project an impl through an alias."""
        proof_key = self.implementation_proof_key(crate, impl, bindings)
        added_proof_key = proof_key is not None and proof_key not in self._trait_proof_stack
        if added_proof_key:
            self._trait_proof_stack.add(proof_key)
        try:
            return self.check_implementation_predicates_inner(crate, impl, bindings)
        finally:
            if added_proof_key:
                self._trait_proof_stack.remove(proof_key)

    def implementation_proof_key(self, crate: str, impl: dict[str, Any],
                                 bindings: dict[str, Any]) -> tuple[tuple[str, ...], str] | None:
        """Identify the concrete trait obligation represented by this impl."""
        trait = impl.get("trait")
        trait_id = trait.get("id") if isinstance(trait, dict) else None
        trait_path = self.reference_path(crate, trait_id) if trait_id is not None else None
        subject = self.substitute_type(impl.get("for"), bindings)
        if trait_path is None or self.contains_open_type_parameter(subject):
            return None
        formatted_subject = self.format_type(crate, subject)
        if formatted_subject is None:
            return None
        return trait_path, formatted_subject

    def has_only_symbolic_trait_predicates(self, crate: str, impl: dict[str, Any],
                                           bindings: dict[str, Any]) -> bool:
        """Recognize compiler-declared generic trait conditions, excluding lifetimes."""
        generics = impl.get("generics", {})
        saw_predicate = False

        def is_symbolic_trait_bound(subject: Any, bound: dict[str, Any]) -> bool:
            nonlocal saw_predicate
            trait_bound = bound.get("trait_bound")
            if trait_bound is None:
                return False
            trait = trait_bound.get("trait", {})
            trait_id = trait.get("id") if isinstance(trait, dict) else None
            if trait_id is None or self.reference_path(crate, trait_id) is None:
                return False
            concrete_subject = self.substitute_type(subject, bindings)
            if not self.contains_open_type_parameter(concrete_subject):
                return False
            saw_predicate = True
            return True

        for parameter in generics.get("params", []):
            kind = parameter.get("kind", {})
            if "lifetime" in kind:
                return False
            type_kind = kind.get("type", {})
            name = parameter.get("name")
            if type_kind.get("bounds") and not name:
                return False
            subject = {"generic": name} if name else None
            for bound in type_kind.get("bounds", []):
                if not is_symbolic_trait_bound(subject, bound):
                    return False
        for predicate in generics.get("where_predicates", []):
            bound_predicate = predicate.get("bound_predicate")
            if bound_predicate is None:
                return False
            subject = bound_predicate.get("type")
            bounds = bound_predicate.get("bounds", [])
            if not bounds:
                return False
            for bound in bounds:
                if not is_symbolic_trait_bound(subject, bound):
                    return False
        return saw_predicate

    def check_implementation_predicates_inner(self, crate: str, impl: dict[str, Any],
                                              bindings: dict[str, Any]) -> bool | None:
        generics = impl.get("generics", {})
        for parameter in generics.get("params", []):
            name = parameter.get("name")
            parameter_kind = parameter.get("kind", {})
            lifetime_kind = parameter_kind.get("lifetime")
            if name and lifetime_kind is not None:
                for outlives in lifetime_kind.get("outlives", []):
                    result = self.check_lifetime_relation(name, outlives, bindings)
                    if result is not True:
                        return result
            type_kind = parameter_kind.get("type", {})
            if not name or not type_kind.get("bounds"):
                continue
            subject = self.substitute_type({"generic": name}, bindings)
            for bound in type_kind["bounds"]:
                trait_bound = bound.get("trait_bound")
                if trait_bound is not None:
                    result = self.check_trait_bound(crate, subject, trait_bound, bindings)
                elif "outlives" in bound:
                    result = self.prove_static_type(crate, subject) if bound["outlives"] == "'static" else None
                else:
                    result = None
                if result is not True:
                    return result
        for predicate in generics.get("where_predicates", []):
            bound_predicate = predicate.get("bound_predicate")
            if bound_predicate is None:
                lifetime_predicate = predicate.get("lifetime_predicate")
                if lifetime_predicate is not None:
                    lifetime = lifetime_predicate.get("lifetime")
                    for outlives in lifetime_predicate.get("outlives", []):
                        result = self.check_lifetime_relation(lifetime, outlives, bindings)
                        if result is not True:
                            return result
                    continue
                return None
            subject = self.substitute_type(bound_predicate.get("type"), bindings)
            for bound in bound_predicate.get("bounds", []):
                trait_bound = bound.get("trait_bound")
                if trait_bound is not None:
                    result = self.check_trait_bound(crate, subject, trait_bound, bindings)
                elif "outlives" in bound:
                    result = self.prove_static_type(crate, subject) if bound["outlives"] == "'static" else None
                else:
                    return None
                if result is not True:
                    return result
        return True

    def conditional_trait_predicates(self, crate: str, impl: dict[str, Any],
                                    bindings: dict[str, Any]) -> list[dict[str, Any]] | None:
        """Serialize fully identified trait bounds that remain conditional on open type vars."""
        generics = impl.get("generics")
        if not isinstance(generics, dict):
            return None
        parameters = generics.get("params", [])
        if not isinstance(parameters, list) or any(
            not isinstance(parameter.get("kind"), dict)
            or "type" not in parameter["kind"]
            for parameter in parameters
        ):
            return None
        declared_names = self.generic_parameter_names(generics)
        if not declared_names:
            return None

        conditions: list[dict[str, Any]] = []
        saw_open_subject = False

        def add_trait_bound(subject: Any, bound: Any) -> bool:
            nonlocal saw_open_subject
            if not isinstance(bound, dict) or set(bound) != {"trait_bound"}:
                return False
            trait_bound = bound["trait_bound"]
            if not isinstance(trait_bound, dict) \
                    or trait_bound.get("modifier") not in {None, "none"} \
                    or trait_bound.get("generic_params", []):
                return False
            trait = trait_bound.get("trait")
            if not isinstance(trait, dict) or trait.get("id") is None:
                return False
            trait_path = self.reference_path(crate, trait["id"])
            if not trait_path:
                return False

            concrete_subject = self.substitute_type(subject, bindings)
            if not self.contains_open_type_parameter(concrete_subject):
                return False
            open_names = self.generic_type_names(concrete_subject)
            if not open_names or not open_names <= declared_names:
                return False
            formatted_subject = self.format_type(crate, concrete_subject)
            arguments = self.format_type_arguments(crate, trait.get("args"), bindings)
            if formatted_subject is None or arguments is None:
                return False
            conditions.append({
                "subject": formatted_subject,
                "traitPath": list(trait_path),
                "arguments": arguments,
            })
            saw_open_subject = True
            return True

        for parameter in parameters:
            name = parameter.get("name")
            type_kind = parameter.get("kind", {}).get("type", {})
            if not isinstance(type_kind, dict):
                return None
            for bound in type_kind.get("bounds", []):
                if not add_trait_bound({"generic": name}, bound):
                    return None

        predicates = generics.get("where_predicates", [])
        if not isinstance(predicates, list):
            return None
        for predicate in predicates:
            if not isinstance(predicate, dict):
                return None
            bound_predicate = predicate.get("bound_predicate")
            if not isinstance(bound_predicate, dict) \
                    or bound_predicate.get("generic_params") \
                    or not isinstance(bound_predicate.get("bounds"), list) \
                    or not bound_predicate["bounds"]:
                return None
            for bound in bound_predicate["bounds"]:
                if not add_trait_bound(bound_predicate.get("type"), bound):
                    return None

        if not conditions or not saw_open_subject:
            return None
        unique = {
            json.dumps(condition, sort_keys=True, separators=(",", ":")): condition
            for condition in conditions
        }
        return [unique[key] for key in sorted(unique)]

    @staticmethod
    def generic_type_names(expression: Any) -> set[str]:
        """Collect rustdoc generic type variables from a type expression."""
        if isinstance(expression, list):
            return set().union(*(_Inventory.generic_type_names(value) for value in expression))
        if not isinstance(expression, dict):
            return set()
        if set(expression) == {"generic"} and isinstance(expression["generic"], str):
            return {expression["generic"]}
        return set().union(*(_Inventory.generic_type_names(value) for value in expression.values()))

    def check_lifetime_relation(self, longer: str, shorter: str,
                                bindings: dict[str, Any]) -> bool | None:
        """Prove a lifetime outlives relation only from identity or `'static`."""
        longer_name = self.substitute_lifetime(longer, bindings)
        shorter_name = self.substitute_lifetime(shorter, bindings)
        if longer_name is _UNRESOLVED_LIFETIME or shorter_name is _UNRESOLVED_LIFETIME:
            return None
        if longer_name == shorter_name or longer_name == "'static":
            return True
        return None

    def prove_static_type(self, crate: str, expression: Any) -> bool | None:
        """Prove a closed rustdoc type has no non-static lifetime parameters."""
        if not isinstance(expression, dict) or len(expression) != 1:
            return None
        kind, value = next(iter(expression.items()))
        if kind == "primitive":
            return True
        if kind == "generic":
            return None
        if kind == "tuple":
            results = [self.prove_static_type(crate, element) for element in value]
            if False in results:
                return False
            return None if None in results else True
        if kind == "slice":
            return self.prove_static_type(crate, value)
        if kind in {"borrowed_ref", "raw_pointer"}:
            lifetime = value.get("lifetime") if kind == "borrowed_ref" else None
            if lifetime not in {None, "'static"}:
                return None
            if kind == "borrowed_ref" and lifetime is None:
                return None
            return self.prove_static_type(crate, value.get("type"))
        if kind != "resolved_path":
            return None
        target = self.resolve(crate, value.get("id")) if value.get("id") is not None else None
        if target is None:
            return None
        target_crate, target_id = target
        item = self.documents[target_crate]["index"].get(target_id)
        if item is None:
            return None
        item_kind = _kind(item)
        if item_kind not in {"struct", "enum", "union", "type_alias"}:
            return None
        inner = item["inner"][item_kind]
        parameters = self.generic_parameters(inner.get("generics", {}))
        arguments = value.get("args")
        if arguments is None:
            return True if not parameters else None
        angle = arguments.get("angle_bracketed") if isinstance(arguments, dict) else None
        if angle is None or angle.get("constraints"):
            return None
        supplied = angle.get("args", [])
        if len(supplied) != len(parameters):
            return None
        for parameter, argument in zip(parameters, supplied, strict=True):
            parameter_kind = parameter.get("kind", {})
            if "lifetime" in parameter_kind:
                lifetime = argument.get("lifetime")
                if lifetime != "'static":
                    return None
            elif "type" in parameter_kind:
                type_argument = argument.get("type")
                if type_argument is None or self.prove_static_type(crate, type_argument) is not True:
                    return None
            elif "const" not in parameter_kind:
                return None
        return True

    def check_trait_bound(self, crate: str, subject: Any, trait_bound: dict[str, Any],
                          bindings: dict[str, Any]) -> bool | None:
        if trait_bound.get("modifier") == "maybe":
            return True
        trait = trait_bound.get("trait", {})
        trait_id = trait.get("id")
        trait_path = self.reference_path(crate, trait_id) if trait_id is not None else None
        if trait_path is None or not trait_path:
            return None

        concrete_subject = self.substitute_type(subject, bindings)
        if self.contains_open_type_parameter(concrete_subject):
            return None
        formatted_subject = self.format_type(crate, concrete_subject)
        if formatted_subject is None:
            return None
        key = trait_path, formatted_subject
        if key in self._trait_proof_stack:
            return None

        self._trait_proof_stack.add(key)
        try:
            return self.find_trait_implementation(crate, trait_path, concrete_subject)
        finally:
            self._trait_proof_stack.remove(key)

    @staticmethod
    def contains_open_type_parameter(expression: Any) -> bool:
        """Reject proofs whose subject still contains an unbound type variable."""
        if isinstance(expression, list):
            return any(_Inventory.contains_open_type_parameter(entry) for entry in expression)
        if not isinstance(expression, dict):
            return False
        if "generic" in expression or "infer" in expression or "impl_trait" in expression:
            return True
        return any(_Inventory.contains_open_type_parameter(entry) for entry in expression.values())

    def find_trait_implementation(self, crate: str, trait_path: tuple[str, ...],
                                  subject: Any) -> bool | None:
        """Prove a trait bound only from a matching rustdoc implementation."""
        uncertain = False
        document = self.documents[crate]
        owner_kinds = {"struct", "enum", "union"}
        for owner in document["index"].values():
            owner_kind = _kind(owner)
            if owner.get("crate_id") != 0 or owner_kind not in owner_kinds:
                continue
            for implementation_id in owner["inner"][owner_kind].get("impls", []):
                implementation_item = document["index"].get(str(implementation_id))
                if implementation_item is None:
                    uncertain = True
                    continue
                implementation = implementation_item.get("inner", {}).get("impl")
                if implementation is None:
                    uncertain = True
                    continue
                trait = implementation.get("trait")
                if trait is None:
                    continue
                implementation_trait_id = trait.get("id")
                implementation_trait_path = (
                    self.reference_path(crate, implementation_trait_id)
                    if implementation_trait_id is not None else None
                )
                if implementation_trait_path is None:
                    uncertain = True
                    continue
                if implementation_trait_path != trait_path:
                    continue

                generic_names = self.generic_parameter_names(implementation.get("generics", {}))
                implementation_bindings: dict[str, Any] = {}
                blanket_impl = implementation.get("blanket_impl")
                blanket_name = blanket_impl.get("generic") if isinstance(blanket_impl, dict) else None
                if blanket_name is not None:
                    if blanket_name not in generic_names:
                        uncertain = True
                        continue
                    implementation_bindings[blanket_name] = subject
                    matches = True
                else:
                    matches = self.unify_type(
                        crate, implementation.get("for"), subject,
                        generic_names, implementation_bindings,
                    )
                if matches is None:
                    uncertain = True
                    continue
                if not matches:
                    continue

                predicates = self.check_implementation_predicates(
                    crate, implementation, implementation_bindings,
                )
                if predicates is True:
                    return True
                if predicates is None:
                    uncertain = True
        return None if uncertain else False

    def is_copy_type(self, crate: str, expression: Any, bindings: dict[str, Any],
                     visiting: set[tuple[str, str]]) -> bool | None:
        expression = self.substitute_type(expression, bindings)
        if not isinstance(expression, dict) or len(expression) != 1:
            return None
        kind, value = next(iter(expression.items()))
        if kind == "generic":
            return True
        if kind == "primitive":
            return value in {
                "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize",
                "u8", "u16", "u32", "u64", "u128", "usize",
            }
        if kind == "tuple":
            results = [self.is_copy_type(crate, child, bindings, visiting) for child in value]
            if False in results:
                return False
            return None if None in results else True
        if kind == "borrowed_ref":
            return not value.get("is_mutable", False)
        if kind == "raw_pointer":
            return True
        if kind != "resolved_path":
            return None
        target = self.resolve(crate, value.get("id")) if value.get("id") is not None else None
        if target is None:
            return None
        target_crate, target_id = target
        if target in visiting:
            return None
        item = self.documents[target_crate]["index"].get(target_id)
        if item is None or _kind(item) not in {"struct", "enum", "union"}:
            return None
        item_kind = _kind(item)
        impl_ids = item["inner"][item_kind].get("impls", [])
        actual_type = expression
        saw_uncertain_copy_impl = False
        for impl_id in impl_ids:
            impl_item = self.documents[target_crate]["index"].get(str(impl_id))
            if impl_item is None:
                saw_uncertain_copy_impl = True
                continue
            impl = impl_item.get("inner", {}).get("impl", {})
            trait = impl.get("trait") or {}
            impl_trait_path = self.reference_path(target_crate, trait.get("id")) if trait.get("id") is not None else None
            if impl_trait_path not in {("core", "marker", "Copy"), ("std", "marker", "Copy"), ("Copy",)}:
                continue
            parameters = self.generic_parameter_names(impl.get("generics", {}))
            impl_bindings: dict[str, Any] = {}
            matches = self.unify_type(target_crate, impl.get("for"), actual_type, parameters, impl_bindings)
            if matches is None:
                saw_uncertain_copy_impl = True
                continue
            if not matches:
                continue
            predicates = self.check_implementation_predicates(target_crate, impl, impl_bindings)
            if predicates is True:
                return True
            if predicates is None:
                saw_uncertain_copy_impl = True
        if saw_uncertain_copy_impl:
            return None
        return False

    def reference_path(self, crate: str, identifier: Any) -> tuple[str, ...] | None:
        if identifier is None:
            return None
        resolved = self.resolve(crate, identifier)
        if resolved is not None:
            return self.definition_path(*resolved)
        path = self.documents[crate].get("paths", {}).get(str(identifier))
        return None if path is None else tuple(path["path"])

    def canonicalize_identity_value(self, crate: str, value: Any) -> Any | None:
        """Replace rustdoc-local IDs with stable definition paths recursively."""
        if isinstance(value, list):
            items = [self.canonicalize_identity_value(crate, entry) for entry in value]
            return _UNRESOLVED_IDENTITY if any(entry is _UNRESOLVED_IDENTITY for entry in items) else items
        if not isinstance(value, dict):
            return value
        output: dict[str, Any] = {}
        canonical_path = None
        if "id" in value:
            canonical_path = self.reference_path(crate, value["id"])
            if canonical_path is None:
                return _UNRESOLVED_IDENTITY
        for key, entry in value.items():
            if key == "id":
                continue
            if key == "path" and canonical_path is not None:
                output[key] = list(canonical_path)
                continue
            canonical = self.canonicalize_identity_value(crate, entry)
            if canonical is _UNRESOLVED_IDENTITY:
                return _UNRESOLVED_IDENTITY
            output[key] = canonical
        if canonical_path is not None and "path" not in output:
            output["definitionPath"] = list(canonical_path)
        return output

    def implementation_members(self, crate: str, impl_ids: list[Any], package: str, path: tuple[str, ...],
                               ancestors: frozenset[tuple[str, str]], exports: tuple[dict[str, Any] | None, ...],
                               owner: dict[str, Any], owner_type: dict[str, Any] | None = None,
                               owner_display: str | None = None, filter_to_owner: bool = False) -> None:
        """Inventory inherent members and exact trait-implementation members."""
        if owner_type is None:
            owner_type = self.type_expression(crate, owner)
        if owner_display is None:
            owner_display = self.format_type(crate, owner_type)
        for impl_id in impl_ids:
            impl_item = self.documents[crate]["index"].get(str(impl_id))
            if impl_item is None:
                self.issue(package, "::".join(path), "missing-rustdoc-implementation", owner,
                           implementationId=str(impl_id))
                continue
            impl = impl_item["inner"]["impl"]
            implementation_type = impl.get("for")
            if implementation_type is None:
                self.issue(package, "::".join(path), "missing-rustdoc-implementation-target", impl_item,
                           implementationId=str(impl_id))
                continue
            implementation_parameters = self.generic_parameter_names(impl.get("generics", {}))
            bindings: dict[str, Any] = {}
            matches_owner = self.unify_type(crate, implementation_type, owner_type,
                                            implementation_parameters, bindings)
            if matches_owner is None:
                self.issue(package, "::".join(path), "unsupported-rustdoc-type-comparison", impl_item,
                           implementationId=str(impl_id), ownerPath=list(self.definition_path(crate, str(owner["id"]))))
                continue
            if filter_to_owner and not matches_owner:
                continue
            blanket = impl.get("blanket_impl")
            conditional_condition = None
            if filter_to_owner or isinstance(blanket, dict):
                predicate_bindings = dict(bindings)
                if isinstance(blanket, dict) and isinstance(blanket.get("generic"), str):
                    predicate_bindings[blanket["generic"]] = owner_type
                applies = self.check_implementation_predicates(crate, impl, predicate_bindings)
                if applies is False:
                    continue
                if applies is None and isinstance(blanket, dict):
                    conditional_condition = self.conditional_trait_predicates(
                        crate, impl, predicate_bindings,
                    )
                # A compiler-reported non-blanket impl remains a valid conditional API
                # when a generic alias preserves its exact owner type and parameters.
                # This carries the impl shape through the alias; it does not prove the
                # predicate for arbitrary substitutions (which remains unresolved).
                symbolic_owner_impl = (
                    filter_to_owner
                    and applies is None
                    and not isinstance(blanket, dict)
                    and self.contains_open_type_parameter(owner_type)
                    and self.format_type(crate, implementation_type)
                    == self.format_type(crate, owner_type)
                    and self.has_only_symbolic_trait_predicates(crate, impl, predicate_bindings)
                )
                if applies is None and conditional_condition is None and not symbolic_owner_impl:
                    self.issue(package, "::".join(path), "unresolved-rustdoc-implementation-predicate", impl_item,
                               implementationId=str(impl_id), ownerPath=list(self.definition_path(crate, str(owner["id"]))))
                    continue
            effective_owner_type = owner_type if filter_to_owner else self.substitute_type(implementation_type, bindings)
            formatted_implementation_owner = self.format_type(crate, effective_owner_type)
            if formatted_implementation_owner is None:
                self.issue(package, "::".join(path), "unsupported-rustdoc-implementation-target", impl_item,
                           implementationId=str(impl_id), ownerPath=list(self.definition_path(crate, str(owner["id"]))))
                continue
            qualified_owner = owner_display if filter_to_owner or matches_owner else formatted_implementation_owner
            if qualified_owner is None:
                self.issue(package, "::".join(path), "unsupported-rustdoc-owner-path", owner,
                           implementationId=str(impl_id))
                continue
            if impl.get("trait") is None:
                self.children(crate, impl.get("items", []), package, tuple(qualified_owner.split("::")),
                              ancestors, exports, associated=True)
                continue
            trait = impl["trait"]
            trait_id = str(trait.get("id", ""))
            trait_definition = self.resolve(crate, trait_id) if trait_id else None
            trait_path: tuple[str, ...] | None = None
            trait_item: dict[str, Any] | None = None
            if trait_definition is not None:
                trait_crate, resolved_trait_id = trait_definition
                trait_item = self.documents[trait_crate]["index"][resolved_trait_id]
                if _kind(trait_item) == "trait":
                    if trait_item.get("visibility") != "public":
                        continue
                    if trait_crate == crate and (trait_crate, resolved_trait_id) not in self.reachable_items:
                        continue
                    trait_path = self.definition_path(trait_crate, resolved_trait_id)
            if trait_path is None:
                raw_path = self.documents[crate]["paths"].get(trait_id)
                if raw_path is not None and raw_path.get("kind") == "trait":
                    trait_path = tuple(raw_path["path"])
            if trait_path is None:
                self.issue(package, f"<{'::'.join(path)} as {trait.get('path', '<unknown>')}>",
                           "unresolved-trait-implementation-path", impl_item,
                           ownerPath=list(self.definition_path(crate, str(owner["id"]))),
                           traitId=trait_id, traitPath=trait.get("path"))
                continue

            trait_bindings = dict(bindings)
            blanket = impl.get("blanket_impl")
            if isinstance(blanket, dict) and isinstance(blanket.get("generic"), str):
                trait_bindings[blanket["generic"]] = effective_owner_type
            trait_arguments = self.format_type_arguments(crate, trait.get("args"), trait_bindings)
            if trait_arguments is None:
                self.issue(package, f"<{qualified_owner} as {'::'.join(trait_path)}>",
                           "unsupported-rustdoc-trait-arguments", impl_item,
                           ownerPath=list(self.definition_path(crate, str(owner["id"]))),
                           traitPath=list(trait_path), traitId=trait_id)
                continue
            qualified_trait = "::".join(trait_path)
            if trait_arguments:
                qualified_trait += f"<{', '.join(trait_arguments)}>"
            qualified_impl = f"<{qualified_owner} as {qualified_trait}>"
            identity_components = self.trait_member_identity(crate, package, owner, trait_path,
                                                              effective_owner_type, qualified_trait, impl,
                                                              trait_bindings)
            if identity_components is None:
                continue
            impl_member_ids = list(impl.get("items", []))
            trait_member_by_name: dict[str, dict[str, Any]] = {}
            if trait_item is not None and trait_definition is not None:
                trait_crate, resolved_trait_id = trait_definition
                for member_id in trait_item["inner"]["trait"].get("items", []):
                    member = self.documents[trait_crate]["index"].get(str(member_id))
                    if member is not None and member.get("name") is not None:
                        trait_member_by_name[member["name"]] = member

            seen_members: set[tuple[str, str]] = set()
            for member_id in impl_member_ids:
                member = self.documents[crate]["index"].get(str(member_id))
                if member is None:
                    self.issue(package, f"{qualified_impl}>::<unknown:{member_id}>",
                               "missing-trait-implementation-item", impl_item,
                               implementationId=str(impl_id), itemId=str(member_id),
                               ownerPath=list(self.definition_path(crate, str(owner["id"]))),
                               traitPath=list(trait_path))
                    continue
                member_kind = _kind(member)
                if member_kind not in KINDS:
                    self.issue(package, f"{qualified_impl}>::{member.get('name', '<unknown>')}",
                               "unsupported-trait-implementation-item", member,
                               itemKind=member_kind, ownerPath=list(self.definition_path(crate, str(owner["id"]))),
                               traitPath=list(trait_path))
                    continue
                member_name = member.get("name")
                if member_name is None:
                    self.issue(package, f"{qualified_impl}>::<unnamed>",
                               "unnamed-trait-implementation-item", member,
                               ownerPath=list(self.definition_path(crate, str(owner["id"]))),
                               traitPath=list(trait_path))
                    continue
                if conditional_condition is None:
                    self.add_trait_impl_member(crate, member, package, qualified_impl,
                                               member_name, exports, owner, identity_components)
                else:
                    self.add_conditional_trait_impl_member(
                        crate, package, qualified_impl, member, member_name, impl_id, owner,
                        conditional_condition,
                    )
                seen_members.add((member_name, member_kind))

            for member_name in impl.get("provided_trait_methods", []):
                member = trait_member_by_name.get(member_name)
                if member is None:
                    # rustdoc explicitly lists this name as a provided trait method.
                    # Preserve that exact declaration without fabricating a signature.
                    member = {**impl_item, "id": f"{impl_id}:provided:{member_name}", "name": member_name,
                              "inner": {"function": {}}}
                member_kind = _kind(member)
                if (member_name, member_kind) not in seen_members:
                    if conditional_condition is None:
                        self.add_trait_impl_member(crate, member, package, qualified_impl,
                                                   member_name, exports, owner, identity_components)
                    else:
                        self.add_conditional_trait_impl_member(
                            crate, package, qualified_impl, member, member_name, impl_id, owner,
                            conditional_condition,
                        )
                    seen_members.add((member_name, member_kind))

            implemented_names = {
                (member["name"], _kind(member))
                for member_id in impl_member_ids
                if (member := self.documents[crate]["index"].get(str(member_id))) is not None
                and member.get("name") is not None
            }
            for member_name, member in trait_member_by_name.items():
                member_kind = _kind(member)
                member_inner = member["inner"][member_kind]
                has_default = (member_kind == "assoc_type" and member_inner.get("type") is not None) \
                    or (member_kind == "assoc_const" and member_inner.get("value") is not None)
                if has_default and (member_name, member_kind) not in implemented_names:
                    if conditional_condition is None:
                        self.add_trait_impl_member(crate, member, package, qualified_impl,
                                                   member_name, exports, owner, identity_components)
                    else:
                        self.add_conditional_trait_impl_member(
                            crate, package, qualified_impl, member, member_name, impl_id, owner,
                            conditional_condition,
                        )

    def format_type_arguments(self, crate: str, arguments: Any,
                              bindings: dict[str, Any]) -> list[str] | None:
        if arguments is None:
            return []
        if not isinstance(arguments, dict) or "angle_bracketed" not in arguments:
            return None
        angle = arguments["angle_bracketed"]
        if angle.get("constraints"):
            return None
        output = []
        for argument in angle.get("args", []):
            if "type" in argument:
                value = self.format_type(crate, self.substitute_type(argument["type"], bindings))
            elif "lifetime" in argument:
                value = str(argument["lifetime"])
            elif "const" in argument:
                value = argument["const"].get("value")
                value = None if value is None else str(value)
            else:
                value = None
            if value is None:
                return None
            output.append(value)
        return output

    def trait_member_identity(self, crate: str, package: str, owner: dict[str, Any],
                              trait_path: tuple[str, ...], owner_type: Any,
                              qualified_trait: str, impl: dict[str, Any],
                              bindings: dict[str, Any]) -> list[Any] | None:
        generics = self.substitute_type(impl.get("generics", {}), bindings)
        canonical_generics = self.canonicalize_identity_value(crate, generics)
        if canonical_generics is _UNRESOLVED_IDENTITY:
            self.issue(package, f"<{self.format_type(crate, owner_type)} as {qualified_trait}>",
                       "unresolved-rustdoc-predicate-identity", owner,
                       ownerPath=list(self.definition_path(crate, str(owner["id"]))),
                       traitPath=list(trait_path))
            return None
        return [
            "rust-trait-implementation", package,
            list(self.definition_path(crate, str(owner["id"]))), list(trait_path),
            self.format_type(crate, owner_type), qualified_trait,
            canonical_generics,
        ]

    def add_trait_impl_member(self, crate: str, member: dict[str, Any], package: str,
                              qualified_impl: str, member_name: str,
                              exports: tuple[dict[str, Any] | None, ...], owner: dict[str, Any],
                              identity_components: list[Any]) -> None:
        item_kind = _kind(member)
        export_name = f"{qualified_impl}::{member_name}"
        identity_components = [*identity_components, member_name, item_kind]
        self.add(crate, member, package, (export_name,), exports, associated=True,
                 identity_components=identity_components)

    def add_conditional_trait_impl_member(self, crate: str, package: str, qualified_impl: str,
                                          member: dict[str, Any], member_name: str,
                                          implementation_id: Any, owner: dict[str, Any],
                                          condition: list[dict[str, Any]]) -> None:
        """Record a known conditional trait member without advertising an API signature."""
        export_name = f"{qualified_impl}::{member_name}"
        record = {
            "packageId": package,
            "exportName": export_name,
            "kind": "associated-item",
            "implementationId": str(implementation_id),
            "ownerPath": list(self.definition_path(crate, str(owner["id"]))),
            "condition": condition,
        }
        key = package, export_name
        previous = self.conditional_records.get(key)
        if previous is not None and previous != record:
            self.issue(package, export_name, "conflicting-conditional-rustdoc-declaration", member,
                       implementationId=str(implementation_id))
            return
        self.conditional_records[key] = record

    @staticmethod
    def fields(kind: Any) -> list[Any]:
        if isinstance(kind, str):
            return []
        if "tuple" in kind:
            return kind["tuple"]
        return kind.get("plain", kind.get("struct", {})).get("fields", [])

    def run(self) -> dict[str, Any]:
        def visit_roots() -> None:
            for crate, doc in sorted(self.documents.items()):
                root_id = str(doc["root"])
                root = doc["index"][root_id]
                self.children(crate, root["inner"]["module"]["items"], "rust-" + crate.replace("_", "-"),
                              (crate,), frozenset({(crate, root_id)}), ())

        self.reachability_prepass = True
        visit_roots()
        self.reachability_prepass = False
        self.records.clear()
        self.conditional_records.clear()
        self.issues.clear()
        visit_roots()
        aliases: dict[str, list[str]] = {}
        for record in self.records.values():
            aliases.setdefault(record["identity"], []).append(record["exportName"])
        records = sorted(self.records.values(), key=_key)
        for record in records:
            record["aliases"] = sorted(set(aliases[record["identity"]]))
            sites = {json.dumps(site, sort_keys=True): site for site in record["exportLocations"]}
            record["exportLocations"] = [sites[key] for key in sorted(sites)]
        issues = {json.dumps(issue, sort_keys=True): issue for issue in self.issues}
        conditional_records = sorted(
            self.conditional_records.values(),
            key=lambda record: (record["packageId"], record["exportName"], record["kind"]),
        )
        return {
            "formatVersion": 1, "toolchain": TOOLCHAIN, "rustdocFormat": FORMAT_VERSION,
            "crates": sorted(self.documents), "complete": not issues,
            "declarations": records,
            "conditionalDeclarations": conditional_records,
            "issues": sorted(issues.values(), key=lambda i: (*_key(i), i["reason"], json.dumps(i, sort_keys=True))),
        }


def inventory(documents: dict[str, dict[str, Any]]) -> dict[str, Any]:
    """Enumerate actual public paths in the supported rustdoc declaration subset."""
    return _Inventory(documents).run()


def compare(observed: dict[str, Any], manifest: dict[str, Any]) -> dict[str, Any]:
    """Compare canonical package/name/kind records; blocked entries still count as inventory."""
    package_ids = {"rust-" + name.replace("_", "-") for name in observed["crates"]}
    expected: dict[tuple[str, str], dict[str, Any]] = {}
    for row in manifest["publicDeclarations"]:
        if row["packageId"] in package_ids:
            key = _comparison_key(row)
            if key in expected:
                raise ValueError(f"duplicate manifest declaration: {row['exportName']}")
            expected[key] = row
    exclusions = {_comparison_key(row) for row in manifest.get("approvedExclusions", [])}
    actual: dict[tuple[str, str], dict[str, Any]] = {}
    conditional_keys: set[tuple[str, str]] = set()
    conditional_rows = observed.get("conditionalDeclarations", [])
    conditional_keys = {_comparison_key(row) for row in conditional_rows}
    for row in [*observed["declarations"], *conditional_rows]:
        key = _comparison_key(row)
        previous = actual.get(key)
        if previous is not None:
            if previous.get("identity") != row.get("identity"):
                raise ValueError(
                    f"distinct Rust declarations collide after path normalization: "
                    f"{previous['exportName']} and {row['exportName']}"
                )
            raise ValueError(f"duplicate observed declaration after path normalization: {row['exportName']}")
        actual[key] = row
    matched, missing, excluded, wrong_kind = [], [], [], []
    conditional_state_mismatches, conditional_condition_mismatches = [], []
    for key, row in sorted(actual.items()):
        if key in expected:
            if row["kind"] == expected[key]["kind"]:
                expected_row = expected[key]
                if key in conditional_keys:
                    if expected_row.get("declarationState") != "blocked":
                        conditional_state_mismatches.append({
                            **row, "manifestState": expected_row.get("declarationState"),
                        })
                    if expected_row.get("condition") != row.get("condition"):
                        conditional_condition_mismatches.append({
                            **row, "manifestCondition": expected_row.get("condition"),
                        })
                elif "condition" in expected_row:
                    conditional_condition_mismatches.append({
                        **row, "manifestCondition": expected_row.get("condition"),
                    })
                matched.append({**row, "declarationState": expected_row["declarationState"]})
            else:
                wrong_kind.append({**row, "manifestKind": expected[key]["kind"]})
        elif key in exclusions:
            excluded.append(row)
        else:
            missing.append(row)
    not_observed = [expected[key] for key in sorted(expected.keys() - actual.keys())]
    return {
        **observed,
        "matchesManifest": not (
            missing or not_observed or wrong_kind
            or conditional_state_mismatches or conditional_condition_mismatches
        ),
        "matched": matched, "missingFromManifest": missing, "notObserved": not_observed,
        "kindMismatches": wrong_kind, "excluded": excluded,
        "conditionalStateMismatches": conditional_state_mismatches,
        "conditionalConditionMismatches": conditional_condition_mismatches,
        "counts": {"observed": len(actual), "manifest": len(expected), "matched": len(matched),
                   "missingFromManifest": len(missing), "notObserved": len(not_observed),
                   "kindMismatches": len(wrong_kind), "excluded": len(excluded),
                   "conditionalStateMismatches": len(conditional_state_mismatches),
                   "conditionalConditionMismatches": len(conditional_condition_mismatches),
                   "issues": len(observed["issues"])},
    }


def require_installed_toolchain(toolchains: str) -> None:
    """Fail before invoking a rustup proxy that could install a missing compiler."""
    if not any(line.split()[0].startswith(f"{TOOLCHAIN}-") for line in toolchains.splitlines() if line.strip()):
        raise ValueError(f"Rust {TOOLCHAIN} must be already installed; the auditor never downloads a toolchain")


def build_documents(repo_root: Path, target_dir: Path) -> dict[str, dict[str, Any]]:
    """Produce fresh JSON with an installed pinned compiler and no network fallback."""
    installed = subprocess.run(["rustup", "toolchain", "list"], check=True, capture_output=True, text=True)
    require_installed_toolchain(installed.stdout)
    version = subprocess.run(["rustc", f"+{TOOLCHAIN}", "--version"], check=True, capture_output=True, text=True)
    if not version.stdout.startswith(f"rustc {TOOLCHAIN} "):
        raise ValueError(f"the installed toolchain must be Rust {TOOLCHAIN}")
    documents = {}
    for package in PACKAGES:
        crate = package.replace("-", "_")
        environment = dict(os.environ)
        for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS",
                    "RUSTC", "RUSTDOC", "CARGO_BUILD_TARGET"):
            environment.pop(key, None)
        environment.update({"CARGO_TARGET_DIR": str(target_dir), "RUSTC_BOOTSTRAP": "1"})
        subprocess.run([
            "cargo", f"+{TOOLCHAIN}", "rustdoc", "--locked", "--offline", "--all-features", "--lib", "-p", package,
            "--", "-Z", "unstable-options", "--output-format", "json", "--document-private-items", "--document-hidden-items",
        ], cwd=repo_root, env=environment, check=True)
        documents[crate] = json.loads((target_dir / "doc" / f"{crate}.json").read_text(encoding="utf-8"))
    return documents


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path, help="write deterministic JSON; default: standard output")
    parser.add_argument("--check", action="store_true", help="exit 1 for inventory mismatches or incomplete audit scope")
    args = parser.parse_args(argv)
    root = args.repo_root.resolve()
    try:
        with tempfile.TemporaryDirectory(prefix="kmipkit-rust-export-audit-") as build_dir:
            documents = build_documents(root, Path(build_dir))
        manifest = json.loads((root / "specification/api/public-api.json").read_text(encoding="utf-8"))
        report = compare(inventory(documents), manifest)
        output = json.dumps(report, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
        if args.report is None:
            sys.stdout.write(output)
        else:
            args.report.write_text(output, encoding="utf-8", newline="\n")
            print(json.dumps(report["counts"], sort_keys=True))
        return int(args.check and not (report["complete"] and report["matchesManifest"]))
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Rust export audit failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
