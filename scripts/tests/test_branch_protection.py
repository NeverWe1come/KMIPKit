"""Tests for the release-only branch-protection API adapter."""

from __future__ import annotations

import io
import json
import unittest
from unittest import mock

from scripts import branch_protection


class BranchProtectionTests(unittest.TestCase):
    def protection_response(self) -> dict[str, object]:
        return {
            "url": "https://api.github.com/repos/NeverWe1come/KMIPKit/branches/release/1.0.0/protection",
            "required_pull_request_reviews": {
                "url": "https://api.github.com/required_pull_request_reviews",
                "required_approving_review_count": 2,
                "dismiss_stale_reviews": True,
                "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
            },
            "enforce_admins": {"url": "https://api.github.com/enforce_admins", "enabled": True},
            "allow_force_pushes": {"enabled": False},
            "allow_deletions": {"enabled": False},
        }

    def test_normalizer_projects_only_required_policy_fields(self) -> None:
        normalized = branch_protection.normalize_branch_protection(self.protection_response())

        self.assertEqual(
            {
                "protected": True,
                "required_pull_request_reviews": {
                    "required_approving_review_count": 2,
                    "dismiss_stale_reviews": True,
                    "bypass_pull_request_allowances": {"users": [], "teams": [], "apps": []},
                },
                "enforce_admins": {"enabled": True},
                "allow_force_pushes": {"enabled": False},
                "allow_deletions": {"enabled": False},
            },
            normalized,
        )

    def test_normalizer_rejects_missing_or_malformed_policy_fields(self) -> None:
        response = self.protection_response()
        response["required_pull_request_reviews"]["required_approving_review_count"] = True
        with self.assertRaises(ValueError):
            branch_protection.normalize_branch_protection(response)

    def test_normalizer_accepts_single_maintainer_policy_and_api_omitted_bypass(self) -> None:
        response = self.protection_response()
        reviews = response["required_pull_request_reviews"]
        reviews["required_approving_review_count"] = 0
        reviews["require_code_owner_reviews"] = False
        del reviews["bypass_pull_request_allowances"]

        normalized = branch_protection.normalize_branch_protection(response)

        self.assertEqual(0, normalized["required_pull_request_reviews"]["required_approving_review_count"])
        self.assertFalse(normalized["required_pull_request_reviews"]["require_code_owner_reviews"])
        self.assertEqual(
            {"users": [], "teams": [], "apps": []},
            normalized["required_pull_request_reviews"]["bypass_pull_request_allowances"],
        )

    def test_normalizer_rejects_review_policy_that_blocks_solo_maintainer(self) -> None:
        for field, value in (
            ("required_approving_review_count", 1),
            ("require_code_owner_reviews", True),
        ):
            response = self.protection_response()
            response["required_pull_request_reviews"][field] = value
            with self.subTest(field=field):
                with self.assertRaisesRegex(ValueError, "review policy is insufficient"):
                    branch_protection.normalize_branch_protection(response)

    def test_normalizer_rejects_each_nonempty_bypass_allowance_without_echoing_entries(self) -> None:
        for field in ("users", "teams", "apps"):
            response = self.protection_response()
            response["required_pull_request_reviews"]["bypass_pull_request_allowances"][field] = [
                "SecretPolicyPrincipalMarker"
            ]
            with self.subTest(field=field):
                with self.assertRaisesRegex(ValueError, "policy is insufficient") as raised:
                    branch_protection.normalize_branch_protection(response)
                self.assertNotIn("SecretPolicyPrincipalMarker", str(raised.exception))

    def test_cli_emits_one_compact_normalized_output(self) -> None:
        input_stream = io.StringIO(json.dumps(self.protection_response()))
        output_stream = io.StringIO()
        with mock.patch("sys.stdin", input_stream), mock.patch("sys.stdout", output_stream):
            exit_code = branch_protection.main()

        self.assertEqual(0, exit_code)
        lines = output_stream.getvalue().splitlines()
        self.assertEqual(1, len(lines))
        key, value = lines[0].split("=", 1)
        self.assertEqual("policy_json", key)
        self.assertEqual(
            branch_protection.normalize_branch_protection(self.protection_response()),
            json.loads(value),
        )

    def test_cli_rejects_malformed_json_without_echoing_the_response(self) -> None:
        input_stream = io.StringIO('{"secret_marker":"do-not-print"')
        error_stream = io.StringIO()
        with mock.patch("sys.stdin", input_stream), mock.patch("sys.stderr", error_stream):
            exit_code = branch_protection.main()

        self.assertEqual(1, exit_code)
        self.assertNotIn("do-not-print", error_stream.getvalue())


if __name__ == "__main__":
    unittest.main()
