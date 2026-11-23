"""Interpret payload presence rules in the pinned KMIP specifications."""

import re


def payload_presence_contract(requiredness: str, description: str) -> tuple[list[str], list[dict]]:
    """Translate pinned REQUIRED cells and conditional SHALL prose into presence branches."""
    normalized = re.sub(r"\s+", " ", requiredness).strip().casefold()
    branches: list[str] = ["present"]
    presence: list[dict] = []

    def add(condition: str, value: str) -> None:
        branch = {"condition": condition, "presence": value}
        if branch not in presence:
            presence.append(branch)

    if normalized.startswith("yes for single-part"):
        if "unless digested data is supplied" in normalized:
            add("single-part and Digested Data is absent", "required")
            add("single-part and Digested Data is supplied", "optional")
            add("multi-part", "optional")
            branches.extend(["condition_true_and_false", "required_when_condition_true", "optional_when_condition_false"])
        else:
            add("single-part", "required")
            add("multi-part", "optional")
            branches.extend(["condition_true_and_false", "required_when_condition_true", "optional_when_condition_false"])
    elif normalized.startswith("yes, unless a failure"):
        add("no failure", "required")
        add("failure", "optional")
        branches.extend(["condition_true_and_false", "required_when_condition_true", "optional_when_condition_false"])
    elif normalized.startswith("if and only if the key object is wrapped"):
        add("key object is wrapped", "required")
        add("key object is not wrapped", "prohibited")
        branches.extend(["condition_true_and_false", "required_when_condition_true", "prohibited_when_condition_false"])
    elif normalized.startswith("yes"):
        branches.append("missing_rejected")
    elif normalized.startswith("no"):
        branches.append("omitted")
    else:
        raise ValueError(f"Unrecognized pinned operation payload requiredness: {requiredness!r}")

    if "repeat" in normalized or "repeat" in description.casefold():
        branches.append("occurrence_count_boundaries")

    condition_patterns = (
        (r"\bshall be returned if\s+([^.;]+)", "required"),
        (r"\b(?:required|shall be present|must be present)\s+(?:if|when)\s+([^.;]+)", "required"),
        (r"\bshould be specified\s+(?:if|when)\s+([^.;]+)", "recommended"),
        (r"\bshall not be (?:present|specified)\s+(?:if|when|for)\s+([^.;]+)", "prohibited"),
    )
    for pattern, when_true in condition_patterns:
        for match in re.finditer(pattern, description, re.IGNORECASE):
            condition = re.sub(r"\s+", " ", match.group(1)).strip().rstrip(",")
            if condition.casefold().endswith(" then"):
                condition = condition[:-5].rstrip()
            add(condition, when_true)
            false_condition = (
                condition[:-len("is requested")] + "is not requested"
                if condition.casefold().endswith("is requested")
                else f"{condition} is false"
            )
            add(false_condition, "optional")
            branches.extend(["condition_true_and_false", "required_when_condition_true" if when_true == "required" else f"{when_true}_when_condition_true"])
            if when_true == "prohibited":
                branches.append("prohibited_when_condition_true")
            else:
                branches.append("optional_when_condition_false")

    return list(dict.fromkeys(branches)), presence
