#!/usr/bin/env python3
"""Project the live GitHub branch-protection response into strict evidence."""

from __future__ import annotations

import json
import sys
from collections.abc import Mapping
from typing import Any


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON object key")
        result[key] = value
    return result


def _enabled_setting(response: Mapping[str, Any], name: str, expected: bool) -> dict[str, bool]:
    setting = response.get(name)
    if not isinstance(setting, Mapping) or type(setting.get("enabled")) is not bool:
        raise ValueError("branch-protection policy is missing or malformed")
    if setting["enabled"] is not expected:
        raise ValueError("branch-protection policy is insufficient")
    return {"enabled": expected}


def normalize_branch_protection(response: Any) -> dict[str, Any]:
    """Validate required live settings and omit unrelated API response fields."""
    if not isinstance(response, Mapping):
        raise ValueError("branch-protection response is malformed")

    reviews = response.get("required_pull_request_reviews")
    if not isinstance(reviews, Mapping):
        raise ValueError("required review policy is missing or malformed")
    approvals = reviews.get("required_approving_review_count")
    dismiss_stale = reviews.get("dismiss_stale_reviews")
    if type(approvals) is not int or approvals < 1 or dismiss_stale is not True:
        raise ValueError("required review policy is insufficient")

    allowances = reviews.get("bypass_pull_request_allowances")
    if not isinstance(allowances, Mapping) or set(allowances) != {"users", "teams", "apps"}:
        raise ValueError("pull-request bypass policy is missing or malformed")
    if any(type(allowances[name]) is not list for name in ("users", "teams", "apps")):
        raise ValueError("pull-request bypass policy is missing or malformed")
    if any(allowances[name] for name in ("users", "teams", "apps")):
        raise ValueError("pull-request bypass policy is insufficient")

    normalized = {
        # A successful 200 response from GET /branches/{branch}/protection is
        # the protection-presence signal; that endpoint has no `protected` key.
        "protected": True,
        "required_pull_request_reviews": {
            "required_approving_review_count": approvals,
            "dismiss_stale_reviews": True,
            "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
        },
        "enforce_admins": _enabled_setting(response, "enforce_admins", True),
        "allow_force_pushes": _enabled_setting(response, "allow_force_pushes", False),
        "allow_deletions": _enabled_setting(response, "allow_deletions", False),
    }
    return normalized


def main() -> int:
    """Read one API response and emit a single safe GitHub output parameter."""
    try:
        response = json.loads(sys.stdin.read(), object_pairs_hook=_unique_object)
        normalized = normalize_branch_protection(response)
    except (OSError, TypeError, ValueError, json.JSONDecodeError):
        print("branch-protection response is invalid or insufficient", file=sys.stderr)
        return 1

    print("policy_json=" + json.dumps(normalized, separators=(",", ":"), sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
