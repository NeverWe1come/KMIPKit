package org.kmipkit.validator;

import org.kmipkit.extensions.ProtocolVersion;

import java.util.List;

/** Semantic checks shared by typed response verification and deterministic response tests. */
final class JavaResponseChecks {
    private JavaResponseChecks() { }

    static boolean advertisesRequestedVersion(List<ProtocolVersion> advertised, ProtocolVersion requested) {
        return advertised != null && requested != null && advertised.contains(requested);
    }
}
