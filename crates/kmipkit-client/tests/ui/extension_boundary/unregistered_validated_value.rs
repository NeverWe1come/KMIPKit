use kmipkit_client::extension_registry::client_request_message_extension;
use kmipkit_protocol::extension::ValidatedExtensionValue;

fn attach_standalone_value(value: ValidatedExtensionValue) {
    let _ = client_request_message_extension(value, true);
}

fn main() {}
