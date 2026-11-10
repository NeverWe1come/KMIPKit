use std::env;
use std::path::{Path, PathBuf};

struct Consumer {
    source: &'static str,
    library: &'static str,
    entry_point: Option<&'static str>,
    include_generated_header: bool,
    warnings_as_errors: bool,
}

const CONSUMERS: [Consumer; 13] = [
    Consumer {
        source: "bindings/c/tests/extension_registry.c",
        library: "kmipkit_extension_registry_c_consumer",
        entry_point: Some("kmipkit_extension_registry_c_consumer_main"),
        include_generated_header: true,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/client_abi_consumer.c",
        library: "kmipkit_client_abi_c_consumer",
        entry_point: Some("kmipkit_client_abi_c_consumer_main"),
        include_generated_header: false,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/managed_object_parity_consumer.c",
        library: "kmipkit_managed_object_parity_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: true,
    },
    Consumer {
        source: "bindings/c/tests/message_model_abi_consumer.c",
        library: "kmipkit_message_model_abi_c_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/credential_type_abi_consumer.c",
        library: "kmipkit_credential_type_abi_c_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/nonce_abi_consumer.c",
        library: "kmipkit_nonce_abi_c_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/credential_abi_consumer.c",
        library: "kmipkit_credential_abi_c_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/credential_secret_abi_consumer.c",
        library: "kmipkit_credential_secret_abi_c_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: false,
    },
    Consumer {
        source: "bindings/c/tests/typed_attribute_parity_consumer.c",
        library: "kmipkit_typed_attribute_parity_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: true,
    },
    Consumer {
        source: "bindings/c/tests/authentication_consumer.c",
        library: "kmipkit_parameters_authentication_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: true,
    },
    Consumer {
        source: "bindings/c/tests/request_validation_consumer.c",
        library: "kmipkit_request_validation_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: true,
    },
    Consumer {
        source: "bindings/c/tests/operation_parity_consumer.c",
        library: "kmipkit_operation_parity_0053_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: true,
    },
    Consumer {
        source: "bindings/c/tests/public_client_api.c",
        library: "kmipkit_public_client_api_consumer",
        entry_point: None,
        include_generated_header: false,
        warnings_as_errors: false,
    },
];

fn compile_consumer(workspace_root: &Path, public_header: &Path, consumer: &Consumer) {
    let source = workspace_root.join(consumer.source);
    println!("cargo:rerun-if-changed={}", source.display());

    let mut build = cc::Build::new();
    build
        .file(source)
        .include(public_header)
        .flag_if_supported("/std:c11")
        .flag_if_supported("-std=c11");
    if consumer.include_generated_header {
        build.include(workspace_root.join("tests/fixtures/api/generated"));
    }
    if let Some(entry_point) = consumer.entry_point {
        build.define("main", Some(entry_point));
    }
    if consumer.warnings_as_errors {
        build.warnings_into_errors(true);
    }
    build.compile(consumer.library);
}

fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_COVERAGE_C_CONSUMER");
    if env::var_os("CARGO_FEATURE_COVERAGE_C_CONSUMER").is_none() {
        return;
    }

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("Cargo provides the package manifest directory"),
    );
    let workspace_root = manifest_dir.join("../..");
    let public_header = workspace_root.join("bindings/c/include");
    let internal_header = workspace_root.join("tests/fixtures/api/generated");
    let target_deps = PathBuf::from(
        env::var_os("OUT_DIR").expect("Cargo provides the build-script output directory"),
    )
    .join("../../../deps");

    println!("cargo:rerun-if-changed={}", public_header.display());
    println!("cargo:rerun-if-changed={}", internal_header.display());
    println!("cargo:rustc-link-search=native={}", target_deps.display());

    for consumer in &CONSUMERS {
        compile_consumer(&workspace_root, &public_header, consumer);
    }
}
