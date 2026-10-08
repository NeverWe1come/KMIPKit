use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprMethodCall, ImplItem, Item, ItemImpl, Stmt};

#[test]
fn registry_provenance_precedes_request_build_codec_and_adapter_handoff() {
    let source = include_str!("../../src/execute.rs");
    let syntax = syn::parse_file(source).expect("client execution source parses");
    let client_impl = syntax
        .items
        .iter()
        .find_map(|item| match item {
            Item::Impl(item_impl) if is_client_impl(item_impl) => Some(item_impl),
            _ => None,
        })
        .expect("Client implementation exists");
    let execute = method(client_impl, "execute_with_options");
    let provenance = call_position(
        execute.block.stmts.as_slice(),
        "validate_request_extension_ownership",
    );
    let request_build = call_position(execute.block.stmts.as_slice(), "build_request_message");
    let exchange = call_position(execute.block.stmts.as_slice(), "exchange_operation");
    assert!(provenance < request_build);
    assert!(request_build < exchange);

    let exchange_operation = method(client_impl, "exchange_operation");
    let encode = call_position(
        exchange_operation.block.stmts.as_slice(),
        "encode_for_execute",
    );
    let adapter = call_position(exchange_operation.block.stmts.as_slice(), "exchange");
    assert!(encode < adapter);
}

fn is_client_impl(item_impl: &ItemImpl) -> bool {
    matches!(item_impl.self_ty.as_ref(),
            syn::Type::Path(type_path) if type_path.path.is_ident("Client"))
}

fn method<'a>(item_impl: &'a ItemImpl, name: &str) -> &'a syn::ImplItemFn {
    item_impl
        .items
        .iter()
        .find_map(|item| match item {
            ImplItem::Fn(function) if function.sig.ident == name => Some(function),
            _ => None,
        })
        .expect("expected Client method exists")
}

fn call_position(statements: &[Stmt], name: &str) -> usize {
    statements
        .iter()
        .position(|statement| {
            let mut calls = CallNames::default();
            calls.visit_stmt(statement);
            calls.0.iter().any(|called| called == name)
        })
        .expect("expected call appears in method body")
}

#[derive(Default)]
struct CallNames(Vec<String>);

impl<'ast> Visit<'ast> for CallNames {
    fn visit_expr_call(&mut self, expression: &'ast ExprCall) {
        if let Expr::Path(path) = expression.func.as_ref()
            && let Some(segment) = path.path.segments.last()
        {
            self.0.push(segment.ident.to_string());
        }
        visit::visit_expr_call(self, expression);
    }

    fn visit_expr_method_call(&mut self, expression: &'ast ExprMethodCall) {
        self.0.push(expression.method.to_string());
        visit::visit_expr_method_call(self, expression);
    }
}
