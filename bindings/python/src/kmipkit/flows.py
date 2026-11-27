"""Phase 4 advanced fluent flows for Python: batching, workflows, streaming,
pending operations, Locate pagination, and closure-based S2C handler builder.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable

from . import client as _client_mod
from . import extensions, fluent
from .high_level import (
    CipherSpec,
    HashAlgorithm,
    KeySpec,
    KeysClient,
    MacSpec,
    SignatureSpec,
)


class FluentBatchBuilder:
    """Fluent multi-operation KMIP request batch with ordering and ID Placeholder support."""

    __slots__ = ("_client", "_batch", "_options")

    def __init__(self, kmip_client: Any) -> None:
        self._client = kmip_client
        self._batch = _client_mod.Batch.create()
        self._options: Any = None

    def add(self, item: extensions.ClientBatchItem) -> FluentBatchBuilder:
        self._batch.add_item(item)
        return self

    def batch_order(self, enabled: bool = True) -> FluentBatchBuilder:
        self._batch.set_ordered(enabled)
        return self

    def ordered(self) -> FluentBatchBuilder:
        return self.batch_order(True)

    def authentication(self, auth: Any) -> FluentBatchBuilder:
        self._batch.set_authentication(auth)
        return self

    def asynchronous_indicator(self, value: int) -> FluentBatchBuilder:
        self._batch.set_asynchronous_indicator(value)
        return self

    def with_options(self, options: Any) -> FluentBatchBuilder:
        self._options = options
        return self

    def build(self) -> _client_mod.Batch:
        return self._batch

    def execute(self) -> Any:
        limits = self._client.codec_limits
        if self._options is not None:
            return self._client.execute_with_options(self._batch, limits, self._options)
        return self._client.execute(self._batch, limits)

    def close(self) -> None:
        self._batch.close()

    def __enter__(self) -> FluentBatchBuilder:
        return self

    def __exit__(self, *_: object) -> None:
        self.close()


class WorkflowsClient:
    """High-level multi-step workflows combining common KMIP sequences safely."""

    __slots__ = ("_client",)

    def __init__(self, kmip_client: Any) -> None:
        self._client = kmip_client

    def create_active_symmetric_key(
        self, spec: KeySpec, usage_mask: int, name: str | None = None
    ) -> str:
        builder = fluent.Request(self._client).create_symmetric_key().with_key_spec(spec).usage_mask(usage_mask)
        if name is not None:
            builder.name(name)
        resp = builder.execute()
        item = resp.item_at(0)
        uid = item.unique_identifier
        if uid is None or uid.text is None:
            raise ValueError("Create response omitted text UniqueIdentifier")
        fluent.Request(self._client).activate().unique_identifier(uid).execute()
        return uid.text

    def revoke_and_destroy(self, unique_identifier: str, reason: extensions.RevocationReason) -> None:
        uid = extensions.UniqueIdentifier.text_string(unique_identifier)
        fluent.Request(self._client).revoke(reason).unique_identifier(uid).execute()
        fluent.Request(self._client).destroy(uid).execute()


class _StreamBase:
    __slots__ = ("_client", "_started", "_correlation")

    def __init__(self, kmip_client: Any) -> None:
        self._client = kmip_client
        self._started = False
        self._correlation: bytes | None = None

    def _apply_indicators(self, builder: Any, *, final: bool = False) -> None:
        if final:
            builder.final_indicator(True)
        if not self._started:
            builder.initial_indicator(True)
            self._started = True
        elif self._correlation is not None:
            builder.correlation_value(self._correlation)


class HashStream(_StreamBase):
    __slots__ = ("_alg",)

    def __init__(self, kmip_client: Any, algorithm: HashAlgorithm) -> None:
        super().__init__(kmip_client)
        self._alg = algorithm

    def update(self, chunk: bytes) -> None:
        b = fluent.Request(self._client).hash().with_hash_algorithm(self._alg).data(chunk)
        self._apply_indicators(b)
        resp = b.execute()
        corr = resp.item_at(0).correlation_value
        if corr is not None:
            self._correlation = corr

    def finish(self) -> bytes:
        b = fluent.Request(self._client).hash().with_hash_algorithm(self._alg)
        self._apply_indicators(b, final=True)
        resp = b.execute()
        data = resp.item_at(0).data
        if data is None:
            raise ValueError("Hash final response omitted Data")
        return data


class EncryptStream(_StreamBase):
    __slots__ = ("_uid", "_spec")

    def __init__(self, kmip_client: Any, unique_identifier: str, spec: CipherSpec) -> None:
        super().__init__(kmip_client)
        self._uid = extensions.UniqueIdentifier.text_string(unique_identifier)
        self._spec = spec

    def update(self, chunk: bytes) -> bytes:
        b = fluent.Request(self._client).encrypt().unique_identifier(self._uid).with_cipher_spec(self._spec).data(chunk)
        self._apply_indicators(b)
        resp = b.execute()
        item = resp.item_at(0)
        if item.correlation_value is not None:
            self._correlation = item.correlation_value
        return item.data or b""

    def finish(self) -> bytes:
        b = fluent.Request(self._client).encrypt().unique_identifier(self._uid).with_cipher_spec(self._spec)
        self._apply_indicators(b, final=True)
        resp = b.execute()
        return resp.item_at(0).data or b""


class StreamsClient:
    """Entry point for explicit stateful multipart cryptographic streams."""

    __slots__ = ("_client",)

    def __init__(self, kmip_client: Any) -> None:
        self._client = kmip_client

    def hash(self, algorithm: HashAlgorithm) -> HashStream:
        return HashStream(self._client, algorithm)

    def encrypt(self, unique_identifier: str, spec: CipherSpec) -> EncryptStream:
        return EncryptStream(self._client, unique_identifier, spec)


@dataclass(frozen=True, slots=True)
class PendingOperation:
    """Handle for an asynchronous KMIP operation carrying its correlation value."""

    correlation_value: bytes

    def poll(self, kmip_client: Any) -> Any:
        return fluent.Request(kmip_client).poll(self.correlation_value).execute()

    def process(self, kmip_client: Any) -> Any:
        return fluent.Request(kmip_client).process(self.correlation_value).execute()

    def cancel(self, kmip_client: Any) -> Any:
        return fluent.Request(kmip_client).cancel(self.correlation_value).execute()


class LocatePaginator:
    """Page-by-page paginator for KMIP Locate using Maximum Items and Offset Items."""

    __slots__ = ("_client", "_attributes", "_page_size", "_offset", "_exhausted")

    def __init__(self, kmip_client: Any, attributes: Any, page_size: int = 100) -> None:
        self._client = kmip_client
        self._attributes = attributes
        self._page_size = max(1, page_size)
        self._offset = 0
        self._exhausted = False

    def next_page(self) -> list[extensions.UniqueIdentifier] | None:
        if self._exhausted:
            return None
        resp = (
            fluent.Request(self._client)
            .locate(self._attributes)
            .maximum_items(self._page_size)
            .offset_items(self._offset)
            .execute()
        )
        ids = list(resp.item_at(0).located_unique_identifiers)
        self._offset += len(ids)
        if len(ids) < self._page_size:
            self._exhausted = True
        return ids or None


@dataclass(slots=True)
class ServerHandlerBuilder:
    """Closure-based builder for Server-to-Client listener handlers."""

    _on_discover_versions: Callable[[Any], Any] | None = field(default=None)
    _on_notify: Callable[[Any], Any] | None = field(default=None)
    _on_put: Callable[[Any], Any] | None = field(default=None)
    _on_query: Callable[[Any], Any] | None = field(default=None)

    def on_discover_versions(self, fn: Callable[[Any], Any]) -> ServerHandlerBuilder:
        self._on_discover_versions = fn
        return self

    def on_notify(self, fn: Callable[[Any], Any]) -> ServerHandlerBuilder:
        self._on_notify = fn
        return self

    def on_put(self, fn: Callable[[Any], Any]) -> ServerHandlerBuilder:
        self._on_put = fn
        return self

    def on_query(self, fn: Callable[[Any], Any]) -> ServerHandlerBuilder:
        self._on_query = fn
        return self

    def build(self) -> dict[str, Callable[[Any], Any] | None]:
        return {
            "discover_versions": self._on_discover_versions,
            "notify": self._on_notify,
            "put": self._on_put,
            "query": self._on_query,
        }


def install_phase4_flows(client_cls: type) -> None:
    """Attach batch(), workflows(), streams() and locate_iter() cleanly with full annotations."""

    def batch(self: Any) -> FluentBatchBuilder:
        return FluentBatchBuilder(self)

    def workflows(self: Any) -> WorkflowsClient:
        return WorkflowsClient(self)

    def streams(self: Any) -> StreamsClient:
        return StreamsClient(self)

    def locate_iter(self: Any, attributes: Any, page_size: int = 100) -> LocatePaginator:
        return LocatePaginator(self._request._client, attributes, page_size)

    client_cls.batch = batch
    client_cls.workflows = workflows
    client_cls.streams = streams
    KeysClient.locate_iter = locate_iter


__all__ = [
    "EncryptStream",
    "FluentBatchBuilder",
    "HashStream",
    "LocatePaginator",
    "PendingOperation",
    "ServerHandlerBuilder",
    "StreamsClient",
    "WorkflowsClient",
    "install_phase4_flows",
]
