from kmipkit import extensions, ttlv
from kmipkit.protocol_values import Operation


def main() -> None:
    with extensions.client_batch_item_ping() as ping:
        assert ping.operation == Operation.PING
    with ttlv.ttlv_structure_create() as payload:
        with ttlv.ttlv_structure_view(payload) as view:
            assert ttlv.ttlv_structure_view_item_count(view) == 0


if __name__ == "__main__":
    main()
