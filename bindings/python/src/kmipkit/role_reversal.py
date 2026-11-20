"""Role Reversal convenience module re-exporting key types."""

from __future__ import annotations

from .client import SetEndpointRoleResponse
from .protocol_values import EndpointRole
from .server_to_client import (
    RoleReversedSession,
    S2cEndpointRoleResponse,
)

__all__ = [
    "EndpointRole",
    "RoleReversedSession",
    "S2cEndpointRoleResponse",
    "SetEndpointRoleResponse",
]
