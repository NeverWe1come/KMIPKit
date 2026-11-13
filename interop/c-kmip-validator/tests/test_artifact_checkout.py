from pathlib import Path
import json
import re
import subprocess
import unittest


ROOT = Path(__file__).resolve().parents[3]
WORKFLOW = ROOT / ".github/workflows/interop-c.yml"


def shared_route_source_pin():
    manifest = json.loads(
        (ROOT / "interop/shared/coverage/route-capabilities.json").read_text(
            encoding="utf-8"
        )
    )
    return manifest["library_commit"]


def installed_artifact_pin():
    support = (ROOT / "interop/c-kmip-validator/tools/artifact_support.py").read_text(
        encoding="utf-8"
    )
    match = re.search(r'^PIN = "([0-9a-f]{40})"$', support, re.MULTILINE)
    if match is None:
        raise AssertionError("artifact support must define one full source commit pin")
    return match.group(1)


class ArtifactCheckoutContractTests(unittest.TestCase):
    def test_every_catalog_enum_and_bitmask_has_a_typed_probe(self):
        catalog = json.loads(
            (ROOT / "interop/shared/catalog/kmipkit-1.0-test-catalog.json").read_text(
                encoding="utf-8"
            )
        )
        live_driver = (ROOT / "interop/c-kmip-validator/src/live_driver.c").read_text(
            encoding="utf-8"
        )
        enum_types = {
            case["requires"]["enum_type"]
            for case in catalog["test_cases"]
            if "enum_type" in case["requires"]
        }
        bitmask_types = {
            case["requires"]["bitmask_type"]
            for case in catalog["test_cases"]
            if "bitmask_type" in case["requires"]
        }
        elements = json.loads(
            (ROOT / "specification/catalog/kmip-2.1.json").read_text(encoding="utf-8")
        )["elements"]
        tag_values = {
            element["name"]: int(element["wire_value"], 16)
            for element in elements
            if element["kind"] == "tag"
        }
        enum_probes = {}
        for name, hexadecimal_tag, zero_tag, helper in re.findall(
            r'\{"([^\"]+)",\s*(?:UINT32_C\(0x([0-9A-Fa-f]+)\)|(0)),\s*(KMIPKIT_ENUMERATION_\w+_name|NULL)\}',
            live_driver,
        ):
            enum_probes[name] = (int(hexadecimal_tag, 16) if hexadecimal_tag else int(zero_tag), helper)
        bitmask_probes = {
            name: (int(tag, 16), known_mask)
            for name, tag, known_mask in re.findall(
                r'\{"([^\"]+)",\s*UINT32_C\(0x([0-9A-Fa-f]+)\),\s*(KMIPKIT_BITMASK_\w+_KNOWN_MASK)',
                live_driver,
            )
        }
        enum_fallbacks = {}
        for name, hexadecimal_value, decimal_value in re.findall(
            r'\{"([^\"]+)",\s*UINT32_C\((?:0x([0-9A-Fa-f]+)|([0-9]+))\)\},?',
            live_driver.split("KMIP_ENUM_FALLBACK_PROBES", 1)[-1].split("};", 1)[0],
        ):
            enum_fallbacks[name] = int(hexadecimal_value, 16) if hexadecimal_value else int(decimal_value)

        self.assertSetEqual(enum_types, set(enum_probes))
        for name, (wire_tag, helper) in enum_probes.items():
            if name == "Item Type":
                self.assertEqual((wire_tag, helper), (0, "NULL"))
                continue
            normalized = re.sub(r"[^A-Za-z0-9]+", "_", name).strip("_").upper()
            self.assertEqual(wire_tag, tag_values[name], name)
            self.assertEqual(helper, f"KMIPKIT_ENUMERATION_{normalized}_name", name)

        self.assertSetEqual(bitmask_types, set(bitmask_probes))
        for name, (wire_tag, known_mask) in bitmask_probes.items():
            normalized = re.sub(r"[^A-Za-z0-9]+", "_", name).strip("_").upper()
            self.assertEqual(wire_tag, tag_values[name], name)
            self.assertEqual(known_mask, f"KMIPKIT_BITMASK_{normalized}_KNOWN_MASK", name)

        self.assertDictEqual(
            enum_fallbacks,
            {
                "Opaque Data Type": 0x80000000,
                "PKCS#11 Function": 1,
                "PKCS#11 Return Code": 0,
                "Tag": 0x420001,
            },
        )

    def test_ci_checkout_includes_the_pinned_release_commit_history(self):
        workflow = WORKFLOW.read_text(encoding="utf-8")
        marker = "      - name: Checkout repository\n"
        start = workflow.index(marker) + len(marker)
        next_step = workflow.find("      - name:", start)
        checkout_step = workflow[start:next_step]

        self.assertIn("fetch-depth: 0", checkout_step)

    def test_c_readme_states_generic_ttlv_encoder_and_dispatch_limits(self):
        readme = (ROOT / "interop/c-kmip-validator/README.md").read_text(encoding="utf-8")

        self.assertIn("no generic TTLV encoder", readme)
        self.assertIn("no generic TTLV request dispatch", readme)

    def test_pinned_c_artifact_is_consistent_and_available(self):
        support = (ROOT / "interop/c-kmip-validator/tools/artifact_support.py").read_text(
            encoding="utf-8"
        )
        installer = (ROOT / "interop/c-kmip-validator/tools/install_artifact.py").read_text(
            encoding="utf-8"
        )
        verifier = (ROOT / "interop/c-kmip-validator/tools/verify_artifact.py").read_text(
            encoding="utf-8"
        )
        pin = installed_artifact_pin()
        for script in (installer, verifier):
            self.assertIn("from artifact_support import", script)
            self.assertNotRegex(script, r"^PIN = ", msg="source pin must have one source of truth")
            self.assertNotRegex(script, r"^def sha256\(", msg="artifact hashing must be shared")
        self.assertEqual(support.count("def sha256("), 1)
        self.assertEqual(support.count("def verify_installed_artifact("), 1)

        cmake = (ROOT / "interop/c-kmip-validator/CMakeLists.txt").read_text(encoding="utf-8")
        workflow = WORKFLOW.read_text(encoding="utf-8")
        readme = (ROOT / "interop/c-kmip-validator/README.md").read_text(encoding="utf-8")
        spec = (ROOT / "specs/085-interop-c-live/spec.md").read_text(encoding="utf-8")
        self.assertIn(f'set(KMIPKIT_LIBRARY_COMMIT "{pin}")', cmake)
        self.assertIn(f"/install/{pin}", workflow)
        self.assertIn(pin, readme)
        self.assertIn("**Base**: `release/1.0.0` at", spec)
        self.assertEqual(pin, shared_route_source_pin())
        self.assertEqual(pin, "9a16cc90a12205101d22a38501a5e74c665df4dc")

        subprocess.run(
            ["git", "cat-file", "-e", f"{pin}^{{tree}}"], cwd=ROOT, check=True
        )
        subprocess.run(
            ["git", "merge-base", "--is-ancestor", pin, "origin/release/1.0.0"],
            cwd=ROOT,
            check=True,
        )

    def test_pinned_c_artifact_matches_the_shared_route_manifest(self):
        self.assertEqual(installed_artifact_pin(), shared_route_source_pin())

    def test_every_c_artifact_identity_uses_the_shared_source_pin(self):
        pin = shared_route_source_pin()
        cmake = (ROOT / "interop/c-kmip-validator/CMakeLists.txt").read_text(
            encoding="utf-8"
        )
        workflow = WORKFLOW.read_text(encoding="utf-8")

        self.assertEqual(installed_artifact_pin(), pin)
        self.assertIn(f'set(KMIPKIT_LIBRARY_COMMIT "{pin}")', cmake)
        self.assertIn(f"/install/{pin}", workflow)


if __name__ == "__main__":
    unittest.main()
