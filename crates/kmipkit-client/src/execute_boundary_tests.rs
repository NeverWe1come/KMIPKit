//! Red-only tests for the private execute-boundary source checker.
//!
//! Fixture files are inert Rust source snippets loaded as text. They are not
//! compiled, executed, or read from OASIS sources. T011 replaces these
//! deliberately incomplete test-only checker candidates with the approved
//! pinned Rust-AST audit; T017 wires that production audit into CI.
//! The low-level caller-owned transport exception belongs to
//! `kmipkit-transport`; client source may call `exchange` only from
//! `Client::execute`.
//! The audit is syntactic: it walks the production module graph under `src`,
//! accepts only `cfg(test)`/`cfg(not(test))`, and rejects unknown attributes,
//! imported protected names, custom macros, and every macro outside its small
//! standard-library whitelist. It does not expand macros or resolve types; the
//! whitelist's token trees are checked for protected boundary identifiers.
//!
//! Traceability: `KMIPKIT-0007-FR-001`, `-FR-003`, `-FR-011`, `-FR-012`,
//! `-FR-015`, `-FR-017`; `KMIPKIT-0007-SC-004`, `-SC-007`, `-SC-008`;
//! ADR-0012, ADR-0014, OD-005.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};
use syn::{Attribute, Expr, ExprCall, ExprMethodCall, Item, ItemMod, Meta, Type, UseTree};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceCoverage {
    CandidateInspected,
    Uninspected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedDecision {
    Accept,
    Reject,
}

struct Fixture {
    id: &'static str,
    path: &'static str,
    source: &'static str,
    probe: &'static str,
    coverage: SourceCoverage,
    expected: ExpectedDecision,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "valid_execute",
        path: "tests/fixtures/execute_boundary/valid_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/valid_execute.rs"),
        probe: "OperationEncodingPermit::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "canonical_vec_macro",
        path: "tests/fixtures/execute_boundary/canonical_vec_macro.rs",
        source: include_str!("../tests/fixtures/execute_boundary/canonical_vec_macro.rs"),
        probe: "vec![0u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "generic_item_input",
        path: "tests/fixtures/execute_boundary/generic_item_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/generic_item_input.rs"),
        probe: "kmipkit_ttlv::Item",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_body_input",
        path: "tests/fixtures/execute_boundary/raw_body_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/raw_body_input.rs"),
        probe: "body: &[u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_structure_input",
        path: "tests/fixtures/execute_boundary/public_structure_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/public_structure_input.rs"),
        probe: "kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "owned_vec_bytes_input",
        path: "tests/fixtures/execute_boundary/owned_vec_bytes_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/owned_vec_bytes_input.rs"),
        probe: "body: Vec<u8>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "boxed_byte_slice_input",
        path: "tests/fixtures/execute_boundary/boxed_byte_slice_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/boxed_byte_slice_input.rs"),
        probe: "Box<[u8]>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "fixed_array_bytes_input",
        path: "tests/fixtures/execute_boundary/fixed_array_bytes_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/fixed_array_bytes_input.rs"),
        probe: "body: [u8; 32]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "exact_unique_batch_id_setter",
        path: "tests/fixtures/execute_boundary/exact_unique_batch_id_setter.rs",
        source: include_str!("../tests/fixtures/execute_boundary/exact_unique_batch_id_setter.rs"),
        probe: "with_unique_batch_item_id",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "other_owned_bytes_setter",
        path: "tests/fixtures/execute_boundary/other_owned_bytes_setter.rs",
        source: include_str!("../tests/fixtures/execute_boundary/other_owned_bytes_setter.rs"),
        probe: "with_bytes",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_enum_input",
        path: "tests/fixtures/execute_boundary/public_enum_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/public_enum_input.rs"),
        probe: "pub enum RequestInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_type_alias",
        path: "tests/fixtures/execute_boundary/public_type_alias.rs",
        source: include_str!("../tests/fixtures/execute_boundary/public_type_alias.rs"),
        probe: "pub type RequestInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "item_reexport_alias",
        path: "tests/fixtures/execute_boundary/item_reexport_alias.rs",
        source: include_str!("../tests/fixtures/execute_boundary/item_reexport_alias.rs"),
        probe: "Item as RequestInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_trait_transport_input",
        path: "tests/fixtures/execute_boundary/public_trait_transport_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/public_trait_transport_input.rs"),
        probe: "trait TransportFactory",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_writer",
        path: "tests/fixtures/execute_boundary/public_writer.rs",
        source: include_str!("../tests/fixtures/execute_boundary/public_writer.rs"),
        probe: "pub fn encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_exchange_outside_execute",
        path: "tests/fixtures/execute_boundary/raw_exchange_outside_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/raw_exchange_outside_execute.rs"),
        probe: "transport.exchange(request, max_response_bytes)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "client_raw_body_execute",
        path: "tests/fixtures/execute_boundary/client_raw_body_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/client_raw_body_execute.rs"),
        probe: "self.transport.exchange(caller_body)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_client_transport_injection",
        path: "tests/fixtures/execute_boundary/public_client_transport_injection.rs",
        source: include_str!(
            "../tests/fixtures/execute_boundary/public_client_transport_injection.rs"
        ),
        probe: "pub fn with_transport<T: Transport>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "facade_transport_reexport",
        path: "tests/fixtures/execute_boundary/facade_transport_reexport.rs",
        source: include_str!("../tests/fixtures/execute_boundary/facade_transport_reexport.rs"),
        probe: "pub use kmipkit_transport::Transport",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "missing_permit",
        path: "tests/fixtures/execute_boundary/missing_permit.rs",
        source: include_str!("../tests/fixtures/execute_boundary/missing_permit.rs"),
        probe: "self.writer.encode(request)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "duplicate_permit",
        path: "tests/fixtures/execute_boundary/duplicate_permit.rs",
        source: include_str!("../tests/fixtures/execute_boundary/duplicate_permit.rs"),
        probe: "let second = OperationEncodingPermit::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_outside_execute",
        path: "tests/fixtures/execute_boundary/permit_outside_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/permit_outside_execute.rs"),
        probe: "fn prepare_permit()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "duplicate_writer",
        path: "tests/fixtures/execute_boundary/duplicate_writer.rs",
        source: include_str!("../tests/fixtures/execute_boundary/duplicate_writer.rs"),
        probe: "self.writer.encode(request, permit);\n        self.writer.encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_outside_execute",
        path: "tests/fixtures/execute_boundary/writer_outside_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/writer_outside_execute.rs"),
        probe: "fn submit",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_alias",
        path: "tests/fixtures/execute_boundary/writer_alias.rs",
        source: include_str!("../tests/fixtures/execute_boundary/writer_alias.rs"),
        probe: "as renamed_encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_reexport",
        path: "tests/fixtures/execute_boundary/writer_reexport.rs",
        source: include_str!("../tests/fixtures/execute_boundary/writer_reexport.rs"),
        probe: "pub use crate::private_wire_writer::encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "macro_token_tree",
        path: "tests/fixtures/execute_boundary/macro_token_tree.rs",
        source: include_str!("../tests/fixtures/execute_boundary/macro_token_tree.rs"),
        probe: "macro_rules! hidden_writer_call",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "qualified_custom_vec_macro",
        path: "tests/fixtures/execute_boundary/qualified_custom_vec_macro.rs",
        source: include_str!("../tests/fixtures/execute_boundary/qualified_custom_vec_macro.rs"),
        probe: "untrusted::vec![0u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "glob_writer_import_with_direct_permit",
        path: "tests/fixtures/execute_boundary/glob_writer_import_with_direct_permit.rs",
        source: include_str!("../tests/fixtures/execute_boundary/glob_writer_import_with_direct_permit.rs"),
        probe: "use crate::private_wire_writer::*",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_mint_and_helper_struct_literals",
        path: "tests/fixtures/execute_boundary/permit_mint_and_helper_struct_literals.rs",
        source: include_str!("../tests/fixtures/execute_boundary/permit_mint_and_helper_struct_literals.rs"),
        probe: "fn helper() -> Self",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "include_bypass",
        path: "tests/fixtures/execute_boundary/include_bypass.rs",
        source: include_str!("../tests/fixtures/execute_boundary/include_bypass.rs"),
        probe: "include!(\"included_writer.rs\")",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "included_writer_source",
        path: "tests/fixtures/execute_boundary/included_writer.rs",
        source: include_str!("../tests/fixtures/execute_boundary/included_writer.rs"),
        probe: "private_wire_writer::encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "generated_source",
        path: "tests/fixtures/execute_boundary/generated_source.rs",
        source: include_str!("../tests/fixtures/execute_boundary/generated_source.rs"),
        probe: "OUT_DIR",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "conditional_compilation",
        path: "tests/fixtures/execute_boundary/conditional_compilation.rs",
        source: include_str!("../tests/fixtures/execute_boundary/conditional_compilation.rs"),
        probe: "#[cfg(any(",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "log_request_time_stamp",
        path: "tests/fixtures/execute_boundary/log_request_time_stamp.rs",
        source: include_str!("../tests/fixtures/execute_boundary/log_request_time_stamp.rs"),
        probe: "request_time_stamp = ?request_time_stamp",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "log_asynchronous_correlation_value",
        path: "tests/fixtures/execute_boundary/log_asynchronous_correlation_value.rs",
        source: include_str!(
            "../tests/fixtures/execute_boundary/log_asynchronous_correlation_value.rs"
        ),
        probe: "asynchronous_correlation_value = ?value",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "uninspected_construct",
        path: "tests/fixtures/execute_boundary/uninspected_construct.rs",
        source: include_str!("../tests/fixtures/execute_boundary/uninspected_construct.rs"),
        probe: "opaque_compiler_construct!",
        coverage: SourceCoverage::Uninspected,
        expected: ExpectedDecision::Reject,
    },
];

const EXPECTED_FIXTURE_IDS: &[&str] = &[
    "valid_execute",
    "canonical_vec_macro",
    "generic_item_input",
    "raw_body_input",
    "public_structure_input",
    "owned_vec_bytes_input",
    "boxed_byte_slice_input",
    "fixed_array_bytes_input",
    "exact_unique_batch_id_setter",
    "other_owned_bytes_setter",
    "public_enum_input",
    "public_type_alias",
    "item_reexport_alias",
    "public_trait_transport_input",
    "public_writer",
    "raw_exchange_outside_execute",
    "client_raw_body_execute",
    "public_client_transport_injection",
    "facade_transport_reexport",
    "missing_permit",
    "duplicate_permit",
    "permit_outside_execute",
    "duplicate_writer",
    "writer_outside_execute",
    "writer_alias",
    "writer_reexport",
    "macro_token_tree",
    "qualified_custom_vec_macro",
    "glob_writer_import_with_direct_permit",
    "permit_mint_and_helper_struct_literals",
    "include_bypass",
    "included_writer_source",
    "generated_source",
    "conditional_compilation",
    "log_request_time_stamp",
    "log_asynchronous_correlation_value",
    "uninspected_construct",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateRejection {
    BoundaryViolation,
    UninspectedSource,
}

#[derive(Default)]
struct BoundaryAudit {
    rejection_reasons: Vec<String>,
    permit_calls: usize,
    writer_calls: usize,
    misplaced_permit_calls: usize,
    misplaced_writer_calls: usize,
    in_client_impl: bool,
    current_function: Option<String>,
    allowed_call_target: bool,
    logged_sensitive_value: bool,
}

impl BoundaryAudit {
    fn reject(&mut self, reason: impl Into<String>) {
        self.rejection_reasons.push(reason.into());
    }

    fn is_rejected(&self) -> bool {
        !self.rejection_reasons.is_empty()
    }

    fn check_attributes(&mut self, attributes: &[Attribute]) {
        for attribute in attributes {
            let path = attribute.path();
            let name = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if path.is_ident("cfg") {
                if !supported_cfg(attribute) {
                    self.reject(format!("unsupported cfg attribute: {name}"));
                }
            } else if path.is_ident("cfg_attr") {
                self.reject("cfg_attr is unsupported");
            } else if path.is_ident("derive") {
                if !supported_derive(attribute) {
                    self.reject(format!("unsupported derive: {name}"));
                }
            } else if path.is_ident("path")
                || path.is_ident("doc")
                || path.is_ident("allow")
                || path.is_ident("warn")
                || path.is_ident("deny")
                || path.is_ident("forbid")
                || path.is_ident("must_use")
                || path.is_ident("non_exhaustive")
                || path.is_ident("inline")
            {
                // These built-in attributes do not inject Rust source or macro code.
            } else {
                // Unknown attributes may invoke unreviewed procedural macros.
                self.reject(format!("unknown attribute: {name}"));
            }
        }
    }

    fn check_public_signature(&mut self, signature: &syn::Signature) {
        let mut finder = PublicTypeFinder::default();
        finder.visit_generics(&signature.generics);
        for argument in &signature.inputs {
            if let syn::FnArg::Typed(argument) = argument {
                finder.visit_type(&argument.ty);
            }
        }
        if finder.has_untyped_item || finder.has_raw_body || finder.has_transport_bound {
            self.reject(format!(
                "public signature exposes a forbidden input (Item={}, raw_body={}, Transport={})",
                finder.has_untyped_item, finder.has_raw_body, finder.has_transport_bound
            ));
        }
    }

    fn finish_fixture(&self) -> Result<(), CandidateRejection> {
        if self.is_rejected()
            || self.misplaced_permit_calls != 0
            || self.misplaced_writer_calls != 0
            || self.permit_calls != self.writer_calls
            || self.permit_calls > 1
        {
            Err(CandidateRejection::BoundaryViolation)
        } else {
            Ok(())
        }
    }
}

impl<'ast> Visit<'ast> for BoundaryAudit {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.check_attributes(std::slice::from_ref(attribute));
        visit::visit_attribute(self, attribute);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        // External module discovery is performed separately from AST policy.
        self.check_attributes(&module.attrs);
        if module.content.is_some() && !is_test_cfg(&module.attrs) {
            visit::visit_item_mod(self, module);
        }
    }

    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        // Declarative/procedural item macros have opaque expansion behavior.
        if is_sensitive_log_macro(&item.mac.path)
            && contains_logged_sensitive_value(&item.mac.tokens.to_string())
        {
            self.logged_sensitive_value = true;
        }
        self.reject("item macro has opaque expansion");
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.check_attributes(&function.attrs);
        if is_test_cfg(&function.attrs) {
            return;
        }
        if matches!(function.vis, syn::Visibility::Public(_))
            && function.sig.ident.to_string().starts_with("encode")
        {
            self.reject("public writer function");
        }
        if matches!(function.vis, syn::Visibility::Public(_)) {
            self.check_public_signature(&function.sig);
        }
        let previous = self
            .current_function
            .replace(function.sig.ident.to_string());
        visit::visit_item_fn(self, function);
        self.current_function = previous;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.check_attributes(&function.attrs);
        if is_test_cfg(&function.attrs) {
            return;
        }
        if matches!(function.vis, syn::Visibility::Public(_)) {
            self.check_public_signature(&function.sig);
        }
        let previous = self
            .current_function
            .replace(function.sig.ident.to_string());
        visit::visit_impl_item_fn(self, function);
        self.current_function = previous;
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        self.check_attributes(&implementation.attrs);
        let previous = self.in_client_impl;
        self.in_client_impl = type_path_ends_with_client(&implementation.self_ty);
        visit::visit_item_impl(self, implementation);
        self.in_client_impl = previous;
    }

    fn visit_item_struct(&mut self, structure: &'ast syn::ItemStruct) {
        self.check_attributes(&structure.attrs);
        if is_test_cfg(&structure.attrs) {
            return;
        }
        if matches!(structure.vis, syn::Visibility::Public(_)) {
            for field in &structure.fields {
                if matches!(field.vis, syn::Visibility::Public(_)) {
                    let mut finder = PublicTypeFinder::default();
                    finder.visit_type(&field.ty);
                    if finder.has_untyped_item || finder.has_raw_body || finder.has_transport_bound
                    {
                        self.reject("public struct field exposes forbidden input");
                    }
                }
            }
        }
        visit::visit_item_struct(self, structure);
    }

    fn visit_item_enum(&mut self, enumeration: &'ast syn::ItemEnum) {
        self.check_attributes(&enumeration.attrs);
        if is_test_cfg(&enumeration.attrs) {
            return;
        }
        if matches!(enumeration.vis, syn::Visibility::Public(_)) {
            for field in enumeration
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter())
            {
                let mut finder = PublicTypeFinder::default();
                finder.visit_type(&field.ty);
                if finder.has_untyped_item || finder.has_raw_body || finder.has_transport_bound {
                    self.reject("public enum variant exposes forbidden input");
                }
            }
        }
        visit::visit_item_enum(self, enumeration);
    }

    fn visit_item_type(&mut self, alias: &'ast syn::ItemType) {
        self.check_attributes(&alias.attrs);
        if !is_test_cfg(&alias.attrs) {
            // Type aliases are not resolved by this syntactic audit and could
            // otherwise conceal a raw Item or byte-slice boundary.
            self.reject("type alias is not inspected by the syntactic boundary audit");
        }
        visit::visit_item_type(self, alias);
    }

    fn visit_item_union(&mut self, union: &'ast syn::ItemUnion) {
        self.check_attributes(&union.attrs);
        if !is_test_cfg(&union.attrs) {
            let mut finder = PublicTypeFinder::default();
            for field in &union.fields.named {
                finder.visit_type(&field.ty);
            }
            if finder.has_untyped_item || finder.has_raw_body || finder.has_transport_bound {
                self.reject("union exposes forbidden input");
            }
        }
        visit::visit_item_union(self, union);
    }

    fn visit_item_trait(&mut self, trait_item: &'ast syn::ItemTrait) {
        self.check_attributes(&trait_item.attrs);
        if is_test_cfg(&trait_item.attrs) {
            return;
        }
        if matches!(trait_item.vis, syn::Visibility::Public(_)) {
            let mut finder = PublicTypeFinder::default();
            for bound in &trait_item.supertraits {
                finder.visit_type_param_bound(bound);
            }
            if finder.has_untyped_item || finder.has_raw_body || finder.has_transport_bound {
                self.reject("public trait exposes a forbidden bound");
            }
            for member in &trait_item.items {
                match member {
                    syn::TraitItem::Fn(function) if !is_test_cfg(&function.attrs) => {
                        self.check_attributes(&function.attrs);
                        self.check_public_signature(&function.sig);
                    }
                    syn::TraitItem::Type(associated) => {
                        let mut finder = PublicTypeFinder::default();
                        finder.visit_generics(&associated.generics);
                        for bound in &associated.bounds {
                            finder.visit_type_param_bound(bound);
                        }
                        if let Some((_, default)) = &associated.default {
                            finder.visit_type(default);
                        }
                        if finder.has_untyped_item
                            || finder.has_raw_body
                            || finder.has_transport_bound
                        {
                            self.reject("public trait associated type exposes forbidden input");
                        }
                    }
                    syn::TraitItem::Const(constant) => {
                        let mut finder = PublicTypeFinder::default();
                        finder.visit_type(&constant.ty);
                        if finder.has_untyped_item
                            || finder.has_raw_body
                            || finder.has_transport_bound
                        {
                            self.reject("public trait associated constant exposes forbidden type");
                        }
                    }
                    _ => {}
                }
            }
        }
        visit::visit_item_trait(self, trait_item);
    }

    fn visit_item_use(&mut self, item_use: &'ast syn::ItemUse) {
        self.check_attributes(&item_use.attrs);
        if is_test_cfg(&item_use.attrs) {
            return;
        }
        if matches!(item_use.vis, syn::Visibility::Public(_))
            && (use_tree_has_name(&item_use.tree, "Transport")
                || use_tree_has_name(&item_use.tree, "Item")
                || use_tree_has_name(&item_use.tree, "encode")
                || use_tree_has_name(&item_use.tree, "encode_for_execute"))
        {
            self.reject("public transport or writer reexport");
        }
        if use_tree_has_name(&item_use.tree, "renamed_encode")
            || use_tree_has_rename(&item_use.tree)
            || (use_tree_has_name(&item_use.tree, "private_wire_writer")
                && use_tree_has_name(&item_use.tree, "encode"))
            || [
                "OperationEncodingPermit",
                "mint",
                "encode",
                "encode_for_execute",
                "encode_raw",
                "write",
                "matches",
                "vec",
                "Clone",
                "Copy",
                "Debug",
                "Default",
                "Eq",
                "PartialEq",
                "Hash",
            ]
            .iter()
            .any(|protected| use_tree_has_name(&item_use.tree, protected))
        {
            self.reject("writer alias or protected import");
        }
        visit::visit_item_use(self, item_use);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let segments = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if is_permit_mint(&segments) {
                self.permit_calls += 1;
                if !self.in_client_impl || self.current_function.as_deref() != Some("execute") {
                    self.misplaced_permit_calls += 1;
                }
            }
            if is_writer_call(&segments) {
                self.writer_calls += 1;
                if !self.in_client_impl || self.current_function.as_deref() != Some("execute") {
                    self.misplaced_writer_calls += 1;
                }
            }
        }
        let previous = self.allowed_call_target;
        self.allowed_call_target = true;
        self.visit_expr(&call.func);
        self.allowed_call_target = previous;
        for argument in &call.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if matches!(
            method.as_str(),
            "encode" | "encode_for_execute" | "encode_raw"
        ) && is_private_writer_receiver(&call.receiver)
        {
            self.writer_calls += 1;
            if !self.in_client_impl || self.current_function.as_deref() != Some("execute") {
                self.misplaced_writer_calls += 1;
            }
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        let segments = expression
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if (is_permit_mint(&segments) || is_writer_call(&segments)) && !self.allowed_call_target {
            self.reject(format!(
                "writer or permit function used as an alias: {segments:?}"
            ));
        }
        visit::visit_expr_path(self, expression);
    }

    fn visit_macro(&mut self, macro_call: &'ast syn::Macro) {
        let last = macro_call
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string());
        let tokens = macro_call.tokens.to_string();
        if is_sensitive_log_macro(&macro_call.path) && contains_logged_sensitive_value(&tokens) {
            self.logged_sensitive_value = true;
            self.reject("sensitive request metadata appears in a logger macro");
        }
        if !matches!(last.as_deref(), Some("write" | "matches" | "vec")) {
            let path = macro_call
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            self.reject(format!("unsupported macro: {path}"));
        }
        if contains_protected_macro_tokens(&tokens) {
            self.reject("protected boundary tokens appear in opaque macro input");
        }
    }
}

#[derive(Default)]
struct PublicTypeFinder {
    has_untyped_item: bool,
    has_raw_body: bool,
    has_transport_bound: bool,
}

impl<'ast> Visit<'ast> for PublicTypeFinder {
    fn visit_type_param_bound(&mut self, bound: &'ast syn::TypeParamBound) {
        if matches!(bound, syn::TypeParamBound::Trait(trait_bound) if trait_bound.path.segments.iter().any(|segment| segment.ident == "Transport"))
        {
            self.has_transport_bound = true;
        }
        visit::visit_type_param_bound(self, bound);
    }

    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Item")
        {
            self.has_untyped_item = true;
        }
        if path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Transport")
        {
            self.has_transport_bound = true;
        }
        visit::visit_type_path(self, path);
    }

    fn visit_type_reference(&mut self, reference: &'ast syn::TypeReference) {
        if let Type::Slice(slice) = reference.elem.as_ref()
            && matches!(slice.elem.as_ref(), Type::Path(path) if path.path.is_ident("u8"))
        {
            self.has_raw_body = true;
        }
        visit::visit_type_reference(self, reference);
    }
}

fn supported_derive(attribute: &Attribute) -> bool {
    let Meta::List(list) = &attribute.meta else {
        return false;
    };
    let derives = syn::parse::Parser::parse2(
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        list.tokens.clone(),
    );
    derives.is_ok_and(|derives| {
        !derives.is_empty()
            && derives.iter().all(|derive| {
                derive.segments.len() == 1
                    && matches!(
                        derive.segments[0].ident.to_string().as_str(),
                        "Clone" | "Copy" | "Debug" | "Default" | "Eq" | "PartialEq" | "Hash"
                    )
            })
    })
}

fn type_path_ends_with_client(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Client"))
}

fn is_permit_mint(segments: &[String]) -> bool {
    matches!(
        segments,
        [permit, mint] if permit == "OperationEncodingPermit" && mint == "mint"
    )
}

fn is_writer_call(segments: &[String]) -> bool {
    segments.len() >= 2
        && segments[segments.len() - 2] == "private_wire_writer"
        && matches!(
            segments.last().map(String::as_str),
            Some("encode" | "encode_for_execute" | "encode_raw")
        )
}

fn is_private_writer_receiver(receiver: &Expr) -> bool {
    matches!(
        receiver,
        Expr::Field(field)
            if matches!(field.base.as_ref(), Expr::Path(path) if path.path.is_ident("self"))
                && matches!(&field.member, syn::Member::Named(name) if name == "writer")
    )
}

fn use_tree_has_name(tree: &UseTree, expected: &str) -> bool {
    match tree {
        UseTree::Path(path) => path.ident == expected || use_tree_has_name(&path.tree, expected),
        UseTree::Name(name) => name.ident == expected,
        UseTree::Rename(rename) => rename.ident == expected || rename.rename == expected,
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|tree| use_tree_has_name(tree, expected)),
        UseTree::Glob(_) => false,
    }
}

fn use_tree_has_rename(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => use_tree_has_rename(&path.tree),
        UseTree::Rename(_) => true,
        UseTree::Group(group) => group.items.iter().any(use_tree_has_rename),
        UseTree::Name(_) | UseTree::Glob(_) => false,
    }
}

fn is_sensitive_log_macro(path: &syn::Path) -> bool {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    segments.iter().any(|segment| {
        matches!(
            segment.as_str(),
            "trace" | "debug" | "info" | "warn" | "error"
        )
    }) && segments.iter().any(|segment| {
        matches!(
            segment.as_str(),
            "log" | "tracing" | "trace" | "debug" | "info" | "warn" | "error"
        )
    })
}

fn contains_logged_sensitive_value(tokens: &str) -> bool {
    let normalized = tokens.to_ascii_lowercase();
    [
        "time_stamp",
        "timestamp",
        "asynchronous_correlation_value",
        "correlation_value",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn contains_protected_macro_tokens(tokens: &str) -> bool {
    let normalized = tokens.to_ascii_lowercase();
    [
        "operationencodingpermit",
        "private_wire_writer",
        "encode",
        "mint",
        "asynchronous_correlation_value",
        "time_stamp",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn audit_source(source: &str) -> Result<BoundaryAudit, CandidateRejection> {
    let parsed = syn::parse_file(source).map_err(|_| CandidateRejection::UninspectedSource)?;
    let mut audit = BoundaryAudit::default();
    audit.visit_file(&parsed);
    audit.finish_fixture().map(|()| audit)
}

fn candidate_check_fixture(fixture: &Fixture) -> Result<(), CandidateRejection> {
    if fixture.coverage == SourceCoverage::Uninspected {
        return Err(CandidateRejection::UninspectedSource);
    }
    audit_source(fixture.source).map(|_| ())
}

fn candidate_check_inventory(inventory: &[&Fixture]) -> Result<(), CandidateRejection> {
    for fixture in inventory {
        if fixture.coverage == SourceCoverage::Uninspected {
            return Err(CandidateRejection::UninspectedSource);
        }
        candidate_check_fixture(fixture)?;
    }
    Ok(())
}

pub(super) fn audit_callsite_for_test(source: &str) -> bool {
    let Ok(parsed) = syn::parse_file(source) else {
        return true;
    };
    let mut audit = BoundaryAudit::default();
    audit.visit_file(&parsed);
    audit.logged_sensitive_value
}

#[derive(Default)]
struct ProductionSources {
    files: Vec<(PathBuf, String)>,
    production_paths: HashSet<PathBuf>,
    test_only_paths: HashSet<PathBuf>,
    rejection_reasons: Vec<String>,
    rejected: bool,
}

fn production_sources() -> ProductionSources {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(src) = manifest.join("src").canonicalize().ok() else {
        return ProductionSources {
            rejected: true,
            ..ProductionSources::default()
        };
    };
    let mut inventory = ProductionSources::default();
    let root = src.join("lib.rs");
    discover_module_file(&root, &src, &mut inventory);

    let mut all_rust_files = Vec::new();
    if collect_rust_files(&src, &mut all_rust_files).is_err() {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push("unable to enumerate src Rust files".to_owned());
    }
    for path in all_rust_files {
        let Ok(path) = path.canonicalize() else {
            inventory.rejected = true;
            continue;
        };
        if !inventory.production_paths.contains(&path) && !inventory.test_only_paths.contains(&path)
        {
            inventory.rejected = true;
            inventory
                .rejection_reasons
                .push(format!("unclassified source: {path:?}"));
        }
    }
    inventory
}

fn collect_rust_files(directory: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
    Ok(())
}

fn discover_module_file(path: &Path, src: &Path, inventory: &mut ProductionSources) {
    let Ok(canonical) = path.canonicalize() else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("missing module file: {path:?}"));
        return;
    };
    if !canonical.starts_with(src) {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("module escapes src: {canonical:?}"));
        return;
    }
    if !inventory.production_paths.insert(canonical.clone()) {
        // Rust may not compile one source file into multiple modules here.
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("duplicate module source: {canonical:?}"));
        return;
    }
    let Ok(source) = fs::read_to_string(&canonical) else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("unreadable module source: {canonical:?}"));
        return;
    };
    let Ok(parsed) = syn::parse_file(&source) else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("unparseable module source: {canonical:?}"));
        return;
    };
    let child_base = module_base(&canonical);
    let mut audit = BoundaryAudit::default();
    audit.visit_file(&parsed);
    inventory.rejected |= audit.is_rejected();
    if audit.is_rejected() {
        inventory.rejection_reasons.push(format!(
            "unsupported or unsafe AST construct in {canonical:?}: {:?}",
            audit.rejection_reasons
        ));
    }
    inventory.files.push((canonical.clone(), source));
    discover_items(
        &parsed.items,
        &child_base,
        canonical.parent().unwrap_or(src),
        src,
        inventory,
    );
}

fn discover_items(
    items: &[Item],
    base: &Path,
    attribute_base: &Path,
    src: &Path,
    inventory: &mut ProductionSources,
) {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };
        if module.content.is_some() {
            if is_test_cfg(&module.attrs) {
                continue;
            }
            let inline_base = base.join(module.ident.to_string());
            discover_items(
                module
                    .content
                    .as_ref()
                    .map(|(_, items)| items.as_slice())
                    .unwrap_or_default(),
                &inline_base,
                &inline_base,
                src,
                inventory,
            );
            continue;
        }
        let Some(path) = module_file_path(module, base, attribute_base) else {
            inventory.rejected = true;
            inventory
                .rejection_reasons
                .push(format!("unresolved module: {}", module.ident));
            continue;
        };
        if is_test_cfg(&module.attrs) {
            if let Ok(path) = path.canonicalize() {
                inventory.test_only_paths.insert(path);
            } else {
                inventory.rejected = true;
                inventory
                    .rejection_reasons
                    .push(format!("missing test-only module: {path:?}"));
            }
        } else {
            discover_module_file(&path, src, inventory);
        }
    }
}

fn module_file_path(module: &ItemMod, base: &Path, attribute_base: &Path) -> Option<PathBuf> {
    for attribute in &module.attrs {
        if attribute.path().is_ident("path") {
            let Meta::NameValue(value) = &attribute.meta else {
                return None;
            };
            let Expr::Lit(value) = &value.value else {
                return None;
            };
            let syn::Lit::Str(value) = &value.lit else {
                return None;
            };
            return Some(attribute_base.join(value.value()));
        }
    }
    let flat = base.join(format!("{}.rs", module.ident));
    if flat.exists() {
        return Some(flat);
    }
    let nested = base.join(module.ident.to_string()).join("mod.rs");
    nested.exists().then_some(nested)
}

fn module_base(file: &Path) -> PathBuf {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    if matches!(
        file.file_name().and_then(|name| name.to_str()),
        Some("lib.rs" | "main.rs" | "mod.rs")
    ) {
        parent.to_path_buf()
    } else {
        parent.join(file.file_stem().unwrap_or_default())
    }
}

fn is_test_cfg(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && matches!(&attribute.meta, Meta::List(list) if list.tokens.to_string() == "test")
    })
}

fn supported_cfg(attribute: &Attribute) -> bool {
    matches!(
        &attribute.meta,
        Meta::List(list) if matches!(list.tokens.to_string().as_str(), "test" | "not (test)")
    )
}

#[test]
fn production_source_inventory_is_complete_and_execute_owns_the_only_writer_permit_pair() {
    let inventory = production_sources();
    assert!(
        !inventory.rejected,
        "production AST inventory failed closed: reasons={:?}, production={:?}, test_only={:?}",
        inventory.rejection_reasons, inventory.production_paths, inventory.test_only_paths
    );
    let paths = inventory
        .files
        .iter()
        .filter_map(|(path, _)| path.file_name().and_then(|name| name.to_str()))
        .collect::<HashSet<_>>();
    for expected in ["lib.rs", "error.rs", "execute.rs", "wire_encoder.rs"] {
        assert!(
            paths.contains(expected),
            "production source missing from audit: {expected}"
        );
    }

    let mut audit = BoundaryAudit::default();
    for (_, source) in &inventory.files {
        let parsed = syn::parse_file(source).expect("the discovered production source parsed");
        audit.visit_file(&parsed);
    }
    assert!(
        !audit.is_rejected(),
        "production Rust source contains an uninspected boundary"
    );
    assert_eq!(
        audit.permit_calls, 1,
        "expected one permit mint in production"
    );
    assert_eq!(
        audit.writer_calls, 1,
        "expected one private writer call in production"
    );
    assert_eq!(audit.misplaced_permit_calls, 0);
    assert_eq!(audit.misplaced_writer_calls, 0);
}

fn candidate_rejects_boundary_fixture(fixture: &Fixture) -> bool {
    candidate_check_fixture(fixture) == Err(CandidateRejection::BoundaryViolation)
}

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .expect("fixture id is part of the explicit source inventory")
}

fn accepted_ids_for_rejected_fixtures(ids: &[&str]) -> Vec<&'static str> {
    ids.iter()
        .filter_map(|id| {
            let fixture = fixture(id);
            (!candidate_rejects_boundary_fixture(fixture)).then_some(fixture.id)
        })
        .collect()
}

#[test]
fn generic_item_and_raw_body_inputs_are_rejected() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "generic_item_input",
        "raw_body_input",
        "public_structure_input",
        "owned_vec_bytes_input",
        "boxed_byte_slice_input",
        "fixed_array_bytes_input",
        "other_owned_bytes_setter",
        "public_enum_input",
        "public_type_alias",
        "item_reexport_alias",
        "public_trait_transport_input",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted forbidden fixtures: {accepted:?}"
    );
}

#[test]
fn low_level_exception_does_not_bypass_the_typed_client_boundary() {
    assert_eq!(
        candidate_check_fixture(fixture("raw_exchange_outside_execute")),
        Err(CandidateRejection::BoundaryViolation),
        "client source may exchange only from Client::execute"
    );
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "public_writer",
        "client_raw_body_execute",
        "public_client_transport_injection",
        "facade_transport_reexport",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted forbidden fixtures: {accepted:?}"
    );
}

#[test]
fn exactly_one_permit_and_writer_call_are_inside_execute() {
    assert_eq!(
        candidate_check_fixture(fixture("valid_execute")),
        Ok(()),
        "one permit mint and one writer call inside Client::execute is allowed"
    );
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "missing_permit",
        "duplicate_permit",
        "permit_outside_execute",
        "duplicate_writer",
        "writer_outside_execute",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted boundary violations: {accepted:?}"
    );
}

#[test]
fn canonical_macros_and_unique_batch_identifier_setter_remain_allowed() {
    for id in [
        "valid_execute",
        "canonical_vec_macro",
        "exact_unique_batch_id_setter",
    ] {
        assert_eq!(
            candidate_check_fixture(fixture(id)),
            Ok(()),
            "approved positive-control fixture should be accepted: {id}"
        );
    }
}

#[test]
fn permit_struct_literals_are_confined_to_the_mint_constructor() {
    let fixture = fixture("permit_mint_and_helper_struct_literals");
    assert_eq!(
        candidate_check_fixture(fixture),
        Err(CandidateRejection::BoundaryViolation),
        "Self literals in permit helpers must not bypass mint ownership"
    );
}

#[test]
fn complete_execute_candidate_requires_one_transport_exchange() {
    assert!(
        matches!(
            audit_source(fixture("canonical_vec_macro").source),
            Err(CandidateRejection::BoundaryViolation)
        ),
        "a complete client source inventory must contain one canonical exchange"
    );
}

#[test]
fn aliases_macros_includes_generated_and_cfg_sources_cannot_bypass_the_audit() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "writer_alias",
        "writer_reexport",
        "macro_token_tree",
        "qualified_custom_vec_macro",
        "glob_writer_import_with_direct_permit",
        "include_bypass",
        "included_writer_source",
        "generated_source",
        "conditional_compilation",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted bypass fixtures: {accepted:?}"
    );
}

#[test]
fn production_logs_cannot_format_request_time_stamp_or_correlation_value() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "log_request_time_stamp",
        "log_asynchronous_correlation_value",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted logging fixtures: {accepted:?}"
    );
}

#[test]
fn source_inventory_fails_closed_for_uninspected_constructs() {
    let uninspected = fixture("uninspected_construct");
    assert_eq!(uninspected.coverage, SourceCoverage::Uninspected);
    assert_eq!(uninspected.expected, ExpectedDecision::Reject);

    assert_eq!(
        candidate_check_inventory(&[uninspected]),
        Err(CandidateRejection::UninspectedSource),
        "the audit must fail closed when source syntax is not inspected"
    );
}

#[test]
fn fixture_inventory_is_explicit_nonempty_and_confined_to_client_tests() {
    let actual_ids = FIXTURES
        .iter()
        .map(|fixture| fixture.id)
        .collect::<Vec<_>>();
    assert_eq!(actual_ids, EXPECTED_FIXTURE_IDS);

    for fixture in FIXTURES {
        assert!(
            !fixture.source.trim().is_empty(),
            "empty fixture: {}",
            fixture.id
        );
        assert!(
            fixture.source.contains(fixture.probe),
            "fixture {} is missing its syntax probe",
            fixture.id
        );
        assert!(
            fixture.path.starts_with("tests/fixtures/execute_boundary/"),
            "fixture {} escapes the crate-local inventory",
            fixture.id
        );
        assert!(!fixture.path.contains("specification/oasis"));
        match fixture.expected {
            ExpectedDecision::Accept => assert!(matches!(
                fixture.id,
                "valid_execute" | "canonical_vec_macro" | "exact_unique_batch_id_setter"
            )),
            ExpectedDecision::Reject => assert!(!matches!(
                fixture.id,
                "valid_execute" | "canonical_vec_macro" | "exact_unique_batch_id_setter"
            )),
        }
    }
}
