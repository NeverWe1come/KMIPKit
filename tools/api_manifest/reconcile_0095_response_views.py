"""Reconcile the approved KMIPKIT-0095 explicit response inspection bindings."""
from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path

REQUIREMENTS = [f"KMIPKIT-0095-FR-{index:03d}" for index in range(1, 6)]
APPROVAL = {
    "kind": "feature-specification", "path": "specs/095-fluent-response-views/spec.md",
    "status": "Approved", "evidence": "**Status**: Approved for implementation.",
}
EVIDENCE = [
    "crates/kmipkit-ffi/tests/support/client_abi_tests.rs",
    "bindings/c/tests/client_abi_consumer.c",
    "bindings/java/src/test/java/org/kmipkit/extensions/ResponseViewsTest.java",
    "bindings/java/src/test/java/org/kmipkit/ClientFacadeTlsIntegrationTest.java",
    "bindings/python/tests/test_response_views.py",
    "bindings/python/tests/test_response_raw_navigation.py",
]


def reconcile(manifest: dict) -> dict:
    result = deepcopy(manifest)
    template = next(row for row in result["functions"] if row["id"] == "client_batch_item_response_raw_ttlv")
    declarations = result["publicDeclarations"]
    for method, java_method in [("ttlv_hex", "ttlvHex"), ("json", "json"), ("xml", "xml")]:
        symbol = "kmipkit_client_batch_item_response_" + method
        bridge = "clientBatchItemResponse" + java_method[0].upper() + java_method[1:]
        record = deepcopy(template)
        record["id"] = "client_batch_item_response_" + method
        record["requirementIds"] = REQUIREMENTS
        record["rust"].update(name=symbol, parameters=[
            {"name": "item", "type": "*mut kmipkit_client_batch_item_response_t"},
            {"name": "limits", "type": "*mut kmipkit_codec_limits_t"},
            {"name": "out_rendered", "type": "*mut *mut kmipkit_secret_bytes_t"},
        ])
        record["c"].update(symbol=symbol, documentation=[
            "Renders the complete retained ResponseMessage, including header and ordered batch, under caller codec limits.",
            "Returns independently owned zeroizing UTF-8 bytes. Use kmipkit_secret_bytes_with_bytes for scoped access and kmipkit_secret_bytes_release to release them.",
            "Output is null on failure. Errors contain no body. This inspection path never sends data.",
        ], parameters=[
            {"name": "item", "kind": "handle", "type": "kmipkit_client_batch_item_response_t *", "ownership": "borrowed"},
            {"name": "limits", "kind": "handle", "type": "kmipkit_codec_limits_t *", "ownership": "borrowed"},
            {"name": "out_rendered", "kind": "output", "type": "kmipkit_secret_bytes_t **", "ownership": "library-owned", "nullable": False},
        ])
        record["java"].update(method=java_method, parameters=[{"name": "limits", "type": "CodecLimits"}],
            jniSymbol="Java_org_kmipkit_NativeExtensionRegistry_" + bridge, returnType="SecretBytes")
        record["python"].update(function="kmipkit.client.BatchItemResult." + method,
            parameters=[{"name": "limits", "type": "CodecLimits"}], cffiSymbol=symbol, returnType="credentials.SecretBytes")
        record["ordering"] = "preserves-response-field-wire-order"
        result["functions"] = [row for row in result["functions"] if row["id"] != record["id"]] + [record]
        mappings = {
            "rust": {"packageId": "rust-kmipkit-client", "name": "kmipkit_client::ClientBatchItemResponse::" + method},
            "c": {"packageId": "c-kmipkit", "name": symbol},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.extensions.ClientBatchItemResult." + java_method},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.client.BatchItemResult." + method},
        }
        for language, kind in [("rust", "associated-item"), ("c", "c-symbol"), ("java", "java-member"), ("python", "python-export")]:
            identities = [(mappings[language]["packageId"], mappings[language]["name"])]
            if language == "rust":
                identities.append(("rust-kmipkit", "kmipkit::ClientBatchItemResponse::" + method))
            for package, name in identities:
                existing = next((row for row in declarations if row["packageId"] == package and row["exportName"] == name), None)
                row = {"declarationId": existing["declarationId"] if existing else "kmipkit-0095-" + method + "-" + language,
                    "packageId": package, "kind": kind, "exportName": name, "capabilityId": "client_execution",
                    "requirementIds": REQUIREMENTS, "declarationState": "verified", "parityScope": "four-language",
                    "approvalSource": APPROVAL, "candidateSpecId": "KMIPKIT-0095", "languageMappings": mappings, "parityEvidenceIds": EVIDENCE}
                if language == "rust":
                    row["languageMappings"] = deepcopy(mappings)
                    row["languageMappings"]["rust"] = {"packageId": package, "name": name}
                if existing is not None:
                    existing.clear(); existing.update(row)
                else:
                    declarations.append(row)
        exclusion = {"packageId": "java-kmipkit", "exportName": "org.kmipkit.NativeExtensionRegistry." + bridge,
            "reason": "JNI plumbing for the typed response renderer; no independent product capability.",
            "approvalSource": APPROVAL, "evidence": "Reviewed against KMIPKIT-0043-FR-021 and KMIPKIT-0095-FR-003. Applications use ClientBatchItemResult." + java_method + "(CodecLimits), returning an AutoCloseable native zeroizing owner."}
        result["approvedExclusions"] = [row for row in result["approvedExclusions"] if not (row["packageId"] == exclusion["packageId"] and row["exportName"] == exclusion["exportName"])] + [exclusion]
    for row in declarations:
        if "::ResponseEncodingError" not in row["exportName"]:
            continue
        resource = row["exportName"].endswith(("::LimitExceeded", "::AllocationFailed"))
        invalid = row["exportName"].endswith("::InvalidResponseTree")
        row.update(declarationState="verified", candidateSpecId="KMIPKIT-0095", approvalSource=APPROVAL, parityEvidenceIds=EVIDENCE, languageMappings={
            "rust": {"packageId": row["packageId"], "name": row["exportName"]},
            "c": {"packageId": "c-kmipkit", "name": "KMIPKIT_ERROR_RESOURCE_LIMIT" if resource else "KMIPKIT_ERROR_INVALID_INPUT" if invalid else "kmipkit_client_batch_item_response_json"},
            "java": {"packageId": "java-kmipkit", "name": "org.kmipkit.ResourceLimitException" if resource else "org.kmipkit.InvalidInputException" if invalid else "org.kmipkit.extensions.ClientBatchItemResult.json"},
            "python": {"packageId": "python-kmipkit", "name": "kmipkit.errors.ResourceLimitError" if resource else "kmipkit.errors.InvalidInputError" if invalid else "kmipkit.client.BatchItemResult.json"},
        })
        row.pop("blockedReason", None);row.pop("featureSpecId", None)
    capability = next(row for row in result["capabilities"] if row["capabilityId"] == "client_execution")
    ready = [row for row in declarations if row["capabilityId"] == "client_execution" and row["declarationState"] in {"declared", "verified"}]
    for language in ("rust", "c", "java", "python"):
        capability["mappings"][language] = sorted({row["languageMappings"][language]["name"] for row in ready})
    return result


if __name__ == "__main__":
    path = Path("specification/api/public-api.json")
    path.write_text(json.dumps(reconcile(json.loads(path.read_text())), indent=2, ensure_ascii=False) + "\n")
