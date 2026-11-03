"""Pin exact T102 KMIPKIT-0042 Interop and Log error traits."""

from __future__ import annotations
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification/api/public-api.json"
REPORT = ROOT / "specs/043-public-api-bindings/reports/rust-export-report-2026-11-01.json"
PROTOCOL, ROOT_PACKAGE, CLIENT = "rust-kmipkit-protocol", "rust-kmipkit", "rust-kmipkit-client"
OWNER_APPROVAL = {
    "kind": "feature-specification", "path": "specs/028-diagnostics-interop/spec.md",
    "status": "Approved", "evidence": "**Status**: Approved for implementation — 2026-10-11",
}
OWNER_REQUIREMENTS = ("KMIPKIT-0042-FR-001", "KMIPKIT-0042-FR-004")
T102_REQUIREMENTS = ("KMIPKIT-0043-FR-010", "KMIPKIT-0043-FR-021")
COMMON_DIRECT = {
    "alloc::borrow::ToOwned>::Owned", "alloc::borrow::ToOwned>::clone_into", "alloc::borrow::ToOwned>::to_owned",
    "alloc::string::ToString>::to_string", "core::any::Any>::type_id", "core::clone::Clone>::clone",
    "core::clone::Clone>::clone_from", "core::clone::CloneToUninit>::clone_to_uninit",
    "core::cmp::Eq>::assert_receiver_is_total_eq", "core::cmp::PartialEq>::eq", "core::cmp::PartialEq>::ne",
    "core::error::Error>::cause", "core::error::Error>::description", "core::error::Error>::provide",
    "core::error::Error>::source", "core::error::Error>::type_id", "core::fmt::Debug>::fmt", "core::fmt::Display>::fmt",
}
CONDITIONAL = {
    "core::convert::Into<U>>::into", "core::convert::TryFrom<U>>::Error", "core::convert::TryFrom<U>>::try_from",
    "core::convert::TryInto<U>>::Error", "core::convert::TryInto<U>>::try_into",
}
OPERATIONS = {
    "InteropError": {"module":"interop", "element":"KMIPKIT-ELEM-OP-C2S-INTEROP", "reference":"OASIS KMIP v2.1 Section 6.1.26, Table 243", "caption":"Table 243: Interop Errors", "request":"InteropRequest", "response":"InteropResponse", "method":"interop", "title":"Interop"},
    "LogError": {"module":"log", "element":"KMIPKIT-ELEM-OP-C2S-LOG", "reference":"OASIS KMIP v2.1 Section 6.1.29, Table 252", "caption":"Table 252: Log Errors", "request":"LogRequest", "response":"LogResponse", "method":"log", "title":"Log"},
}
ERROR_SUFFIXES = {
    "":"blocked", "::InvalidOperationResult":"blocked", "::InvalidOperationResult::0":"blocked",
    "::MalformedResponseItem":"blocked", "::MalformedResponsePayload":"blocked", "::MissingResponsePayload":"blocked",
    "::MissingResultStatus":"blocked", "::UnexpectedOperation":"blocked",
}
RELATED_STATES = {
    "InteropRequest":{"":"verified", "::interop_function":"blocked", "::into_ttlv_payload":"blocked", "::new":"verified", "::with_extension":"blocked", "::with_interop_identifier":"blocked"},
    "InteropResponse":{"":"verified", "::result":"verified", "::try_from_response_item":"blocked", "::with_ttlv":"verified"},
    "LogRequest":{"":"verified", "::into_ttlv_payload":"blocked", "::new":"verified", "::with_extension":"blocked", "::with_log_message":"blocked"},
    "LogResponse":{"":"verified", "::result":"verified", "::try_from_response_item":"blocked", "::with_ttlv":"verified"},
}

def trait_paths(name: str, suffixes: set[str]) -> set[str]:
    return {f"<kmipkit_protocol::{name} as {suffix}" for suffix in suffixes}

def direct_suffixes(name: str) -> set[str]:
    model=f"kmipkit_protocol::{OPERATIONS[name]['module']}::{name}"
    return COMMON_DIRECT | {f"core::borrow::Borrow<{model}>>::borrow", f"core::borrow::BorrowMut<{model}>>::borrow_mut", f"core::convert::From<{model}>>::from"}

def operation_rows() -> dict[tuple[str,str],str]:
    rows={}
    for op in OPERATIONS.values():
        method,title=op['method'],op['title']
        for package,prefix in ((ROOT_PACKAGE,'kmipkit'),(CLIENT,'kmipkit_client')):
            names=(f'{prefix}::Client::{method}',f'{prefix}::ClientBatchOutcome::{title}',f'{prefix}::ClientBatchOutcome::{title}::0',f'{prefix}::ClientOperation::{title}',f'{prefix}::ClientRequest::{title}',f'{prefix}::ClientRequest::{title}::0',f'{prefix}::ClientRequest::{method}')
            rows.update({(package,name):'verified' for name in names})
        rows[(CLIENT,f"kmipkit_client::ClientResponseView<'a>::{method}")]='blocked'
    return rows

class T102InteropLogErrorTraitExclusionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest=json.loads(MANIFEST.read_text(encoding='utf-8'))
        cls.report=json.loads(REPORT.read_text(encoding='utf-8'))

    def test_exact_error_trait_paths_are_approved_exclusions(self):
        self.assertTrue(self.report['complete']); self.assertEqual(self.report['toolchain'],'1.94.0')
        self.assertTrue(self.report['matchesManifest']); self.assertEqual(self.report['issues'],[])
        pairs=set()
        for name,op in OPERATIONS.items():
            direct=trait_paths(name,direct_suffixes(name)); conditional=trait_paths(name,CONDITIONAL); expected=direct|conditional
            prefix=f'<kmipkit_protocol::{name} as '
            for key,exact in (('declarations',direct),('conditionalDeclarations',conditional),('excluded',expected)):
                actual={r['exportName'] for r in self.report[key] if r.get('packageId')==PROTOCOL and r.get('exportName','').startswith(prefix)}
                self.assertEqual(actual,exact)
            self.assertEqual((len(direct),len(conditional)),(21,5))
            declarations={r['exportName'] for r in self.manifest['publicDeclarations'] if r.get('packageId')==PROTOCOL and r.get('exportName','').startswith(prefix)}
            exclusions={r['exportName']:r for r in self.manifest['approvedExclusions'] if r.get('packageId')==PROTOCOL and r.get('exportName','').startswith(prefix)}
            self.assertEqual(declarations,set()); self.assertEqual(set(exclusions),expected)
            for path,row in exclusions.items():
                with self.subTest(name=name,path=path):
                    self.assertEqual(row['approvalSource'],OWNER_APPROVAL)
                    for req in (*OWNER_REQUIREMENTS,op['element'],*T102_REQUIREMENTS): self.assertIn(req,row['evidence'])
                    self.assertIn(op['reference'],row['evidence']); self.assertIn(op['caption'],row['evidence'])
                    self.assertIn('inventory-only',row['evidence']); self.assertIn('makes no conformance claim',row['evidence'])
                    self.assertIn('exact Rust standard-trait implementation',row['reason']); self.assertIn('not a separate KMIP client capability',row['reason'])
            pairs.update((PROTOCOL,path) for path in expected)
        self.assertEqual(len(pairs),52)

    def test_error_rows_and_absent_root_aliases_are_pinned(self):
        for name in OPERATIONS:
            prefix=f'kmipkit_protocol::{name}'
            expected={f'{prefix}{suffix}':state for suffix,state in ERROR_SUFFIXES.items()}
            actual={r['exportName']:r['declarationState'] for r in self.manifest['publicDeclarations'] if r.get('packageId')==PROTOCOL and (r.get('exportName')==prefix or r.get('exportName','').startswith(prefix+'::')) and not r.get('exportName','').startswith('<')}
            self.assertEqual(actual,expected)
            root=f'kmipkit::{name}'
            self.assertFalse(any(r.get('packageId')==ROOT_PACKAGE and (r.get('exportName')==root or r.get('exportName','').startswith(root+'::') or r.get('exportName','').startswith(f'<kmipkit::{name} as ')) for r in self.manifest['publicDeclarations']))

    def test_request_response_and_client_operation_rows_keep_states(self):
        for op in OPERATIONS.values():
            for name in (op['request'],op['response']):
                prefix=f'kmipkit_protocol::{name}'; expected={f'{prefix}{s}':v for s,v in RELATED_STATES[name].items()}
                actual={r['exportName']:r['declarationState'] for r in self.manifest['publicDeclarations'] if r.get('packageId')==PROTOCOL and (r.get('exportName')==prefix or r.get('exportName','').startswith(prefix+'::'))}
                self.assertEqual(actual,expected)
                root=f'kmipkit::{name}'
                self.assertFalse(any(r.get('packageId')==ROOT_PACKAGE and (r.get('exportName')==root or r.get('exportName','').startswith(root+'::') or r.get('exportName','').startswith(f'<kmipkit::{name} as ')) for r in self.manifest['publicDeclarations']))
        expected=operation_rows()
        actual={(r.get('packageId'),r.get('exportName')):r['declarationState'] for r in self.manifest['publicDeclarations'] if (r.get('packageId'),r.get('exportName')) in expected}
        self.assertEqual(actual,expected)

    def test_manifest_and_t102_totals_decrease_by_52(self):
        count=sum(r.get('candidateSpecId')=='KMIPKIT-0043' and r['declarationState']=='blocked' for r in self.manifest['publicDeclarations'])
        self.assertEqual(count,2193); self.assertEqual(len(self.manifest['publicDeclarations']),7391); self.assertEqual(len(self.manifest['approvedExclusions']),7333)

if __name__=='__main__': unittest.main()
