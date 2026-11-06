"""In-memory KMS State Management and Collision Detection for KMIP 2.1."""

from __future__ import annotations

import datetime
import uuid
from typing import Any


class NameCollisionError(Exception):
    """Raised when an object with an identical name already exists in active/pre-active state."""
    pass


class ObjectNotFoundError(Exception):
    """Raised when the requested UniqueIdentifier does not exist."""
    pass


class IllegalLifecycleTransitionError(Exception):
    """Raised when an invalid KMIP lifecycle transition is attempted (e.g. destroy without revoke)."""
    pass


class KmsObject:
    """Represents a managed cryptographic object in the KMS state."""

    def __init__(
        self,
        unique_identifier: str,
        name: str,
        object_type: str = "SymmetricKey",
        state: str = "Active",
        algorithm: str | None = None,
        length: int | None = None,
        attributes: dict[str, Any] | None = None,
    ) -> None:
        self.unique_identifier = unique_identifier
        self.name = name
        self.object_type = object_type
        self.state = state
        self.algorithm = algorithm
        self.length = length
        self.attributes = attributes or {}
        self.created_at = datetime.datetime.now(datetime.timezone.utc).isoformat()
        self.revocation_reason: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "unique_identifier": self.unique_identifier,
            "name": self.name,
            "object_type": self.object_type,
            "state": self.state,
            "algorithm": self.algorithm,
            "length": self.length,
            "attributes": self.attributes,
            "created_at": self.created_at,
            "revocation_reason": self.revocation_reason,
        }


class KmsState:
    """In-memory registry of KMIP objects with collision and lifecycle enforcement."""

    def __init__(self) -> None:
        self.objects: dict[str, KmsObject] = {}
        self._name_index: dict[str, str] = {}  # name -> unique_identifier
        self._counter = 0

    def create_object(
        self,
        name: str,
        object_type: str = "SymmetricKey",
        algorithm: str | None = None,
        length: int | None = None,
        initial_state: str = "Active",
        attributes: dict[str, Any] | None = None,
    ) -> KmsObject:
        """Create and store a managed object. Enforces strict name uniqueness."""
        if name:
            existing_id = self._name_index.get(name)
            if existing_id:
                existing_obj = self.objects.get(existing_id)
                if existing_obj and existing_obj.state != "Destroyed":
                    raise NameCollisionError(
                        f"Name collision: object with name '{name}' already exists (ID: {existing_id}, State: {existing_obj.state})"
                    )

        self._counter += 1
        uid = f"kmipkit-mock-{self._counter:06d}-{uuid.uuid4().hex[:8]}"
        obj = KmsObject(
            unique_identifier=uid,
            name=name,
            object_type=object_type,
            state=initial_state,
            algorithm=algorithm,
            length=length,
            attributes=attributes,
        )
        self.objects[uid] = obj
        if name:
            self._name_index[name] = uid
        return obj

    def get_object(self, unique_identifier: str) -> KmsObject:
        """Retrieve an object by ID."""
        obj = self.objects.get(unique_identifier)
        if not obj or obj.state == "Destroyed":
            raise ObjectNotFoundError(f"Object with ID '{unique_identifier}' not found or destroyed")
        return obj

    def locate_objects(
        self,
        name_prefix: str | None = None,
        name_exact: str | None = None,
        object_type: str | None = None,
        state: str | None = None,
    ) -> list[str]:
        """Search objects matching criteria and return their unique identifiers."""
        matched = []
        for uid, obj in self.objects.items():
            if obj.state == "Destroyed":
                continue
            if name_exact and obj.name != name_exact:
                continue
            if name_prefix and not obj.name.startswith(name_prefix):
                continue
            if object_type and obj.object_type != object_type:
                continue
            if state and obj.state != state:
                continue
            matched.append(uid)
        return matched

    def activate_object(self, unique_identifier: str) -> KmsObject:
        """Transition an object from PreActive to Active."""
        obj = self.get_object(unique_identifier)
        if obj.state == "PreActive":
            obj.state = "Active"
        return obj

    def revoke_object(self, unique_identifier: str, reason: str = "CessationOfOperation") -> KmsObject:
        """Transition an object to Revoked."""
        obj = self.get_object(unique_identifier)
        if obj.state in ("Active", "PreActive"):
            obj.state = "Revoked"
            obj.revocation_reason = reason
        return obj

    def destroy_object(self, unique_identifier: str, allow_active: bool = False) -> KmsObject:
        """Transition an object to Destroyed and release its name from the index."""
        obj = self.objects.get(unique_identifier)
        if not obj:
            raise ObjectNotFoundError(f"Object with ID '{unique_identifier}' not found")
        if obj.state == "Active" and not allow_active:
            raise IllegalLifecycleTransitionError(
                f"Object '{unique_identifier}' is in Active state. Must be Revoked before Destroy."
            )
        obj.state = "Destroyed"
        if obj.name in self._name_index and self._name_index[obj.name] == unique_identifier:
            del self._name_index[obj.name]
        return obj

    def count_active_objects(self) -> int:
        """Return the count of active or preactive objects in memory."""
        return sum(1 for o in self.objects.values() if o.state in ("Active", "PreActive"))

    def reset(self) -> None:
        """Clear all in-memory state."""
        self.objects.clear()
        self._name_index.clear()
        self._counter = 0
