"""Keep shared matrix identity intact in C evidence and pending diagnostics."""

CELL_FIELDS = (
    'case_id', 'route_id', 'target_id', 'public_api_symbol', 'execution_route_id',
    'execution_api_symbol', 'action', 'protocol_version', 'encoding', 'transport',
    'workflow_step_index', 'workflow_operation', 'workflow_item_index',
)


def cell_fields(cell: dict) -> dict:
    return {field: cell[field] for field in CELL_FIELDS if field in cell}


def pending_cell(cell: dict, reason: str) -> dict:
    return {**cell_fields(cell), 'reason': reason}
