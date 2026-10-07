// Generated from specification/api/public-api.json and tests/fixtures/extensions/cases.json. API contract sha256: db9618f0b89ec0c9c6842c3be1749aea88202d426890f19ca5e747dd5ab575f1. Do not edit.
package org.kmipkit;

import java.util.List;

final class SharedExtensionFixtures {
    record Item(int tag, int type, String text, long number, List<Item> children) {}
    record ChildRule(int tag, Schema schema) {}
    record Schema(int type, String text, boolean hasRange, long minimum, long maximum, List<ChildRule> children) {}
    record Definition(String id, String vendor, String name, String version, List<Integer> path, int discriminatorType, String discriminatorText, long discriminatorNumber, Schema schema) {}
    record Case(String id, String vendor, boolean critical, List<Item> payload, String outcome, List<String> matchedIds, boolean typed) {}
    record Attachment(String fixtureId, boolean criticalityIndicator) {}
    record OutboundRequest(String fixtureId, String outcome, List<Attachment> attachments) {}
    static final List<Definition> DEFINITIONS = List.of(
        new Definition("ambiguous.beta", "example.vendor", "ambiguous-beta", "1", List.of(0x540010, 0x540012), 7, "route-beta", 0L, new Schema(1, "", false, 0L, 0L, List.of(new ChildRule(0x540010, new Schema(1, "", false, 0L, 0L, List.of(new ChildRule(0x540011, new Schema(7, "route-alpha", false, 0L, 0L, List.of())), new ChildRule(0x540012, new Schema(7, "route-beta", false, 0L, 0L, List.of())), new ChildRule(0x540013, new Schema(7, "shared", false, 0L, 0L, List.of())))))))),
        new Definition("known.alpha", "example.vendor", "alpha", "1", List.of(0x420001), 7, "alpha-v1", 0L, new Schema(1, "", false, 0L, 0L, List.of(new ChildRule(0x420001, new Schema(7, "alpha-v1", false, 0L, 0L, List.of())), new ChildRule(0x420004, new Schema(3, "", true, 0L, 99L, List.of()))))),
        new Definition("ambiguous.alpha", "example.vendor", "ambiguous-alpha", "1", List.of(0x540010, 0x540011), 7, "route-alpha", 0L, new Schema(1, "", false, 0L, 0L, List.of(new ChildRule(0x540010, new Schema(1, "", false, 0L, 0L, List.of(new ChildRule(0x540011, new Schema(7, "route-alpha", false, 0L, 0L, List.of())), new ChildRule(0x540012, new Schema(7, "route-beta", false, 0L, 0L, List.of())), new ChildRule(0x540013, new Schema(7, "shared", false, 0L, 0L, List.of()))))))))
    );
    static final List<OutboundRequest> OUTBOUND_REQUESTS = List.of(
        new OutboundRequest("synthetic-secret-bearing", "outbound.validated", List.of(new Attachment("synthetic-secret-bearing", false), new Attachment("valid-recognized", true)))
    );
    static final List<Case> CASES = List.of(
        new Case("valid-recognized", "example.vendor", false, List.of(new Item(0x420001, 7, "alpha-v1", 0L, List.of()), new Item(0x420002, 5, "", 4294967295L, List.of()), new Item(0x420004, 3, "", 42L, List.of()), new Item(0x420006, 7, "preserve-this-child-order", 0L, List.of()), new Item(0x540001, 7, "preserve-vendor-range-tag", 0L, List.of())), "recognized", List.of("known.alpha"), true),
        new Case("invalid-schema", "example.vendor", false, List.of(new Item(0x420001, 7, "alpha-v1", 0L, List.of()), new Item(0x420004, 3, "", 100L, List.of())), "unrecognized.schema_invalid", List.of("known.alpha"), false),
        new Case("multiply-matching", "example.vendor", false, List.of(new Item(0x540010, 1, "", 0L, List.of(new Item(0x540011, 7, "route-alpha", 0L, List.of()), new Item(0x540012, 7, "route-beta", 0L, List.of()), new Item(0x540013, 7, "shared", 0L, List.of()))), new Item(0x540014, 5, "", 314159L, List.of()), new Item(0x540015, 7, "preserve-this-child-order", 0L, List.of())), "unrecognized.ambiguous", List.of("ambiguous.alpha", "ambiguous.beta"), false),
        new Case("unknown-preserved", "example.vendor", false, List.of(new Item(0x540021, 5, "", 8675309L, List.of()), new Item(0x540022, 7, "unknown-extension-content", 0L, List.of())), "unrecognized.no_match", List.of(), false),
        new Case("synthetic-secret-bearing", "example.vendor", false, List.of(new Item(0x420001, 7, "alpha-v1", 0L, List.of()), new Item(0x420004, 3, "", 42L, List.of()), new Item(0x420005, 7, "SYNTHETIC-ONLY-EXTENSION-SECRET-7C4A", 0L, List.of())), "recognized", List.of("known.alpha"), true)
    );
    private SharedExtensionFixtures() {}
}
