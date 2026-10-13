use kmipkit_client::UsageAllocationContext;

fn clone_context(context: &UsageAllocationContext) -> UsageAllocationContext {
    Clone::clone(context)
}

fn main() {}
