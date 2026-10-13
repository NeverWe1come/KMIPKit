package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;

/** Forward-compatible common outcome discriminator from the C client ABI. */
public record BatchOutcomeKind(long raw) {
    public static final long COMPLETED_VALUE = 1;
    public static final long PENDING_VALUE = 2;

    public BatchOutcomeKind {
        if (raw < 0 || raw > 0xffff_ffffL) {
            throw new InvalidInputException("outcome discriminator is outside uint32");
        }
    }

    /** Returns whether this is the completed outcome defined by ABI v1. */
    public boolean isCompleted() {
        return raw == COMPLETED_VALUE;
    }

    /** Returns whether this is the Pending outcome defined by ABI v1. */
    public boolean isPending() {
        return raw == PENDING_VALUE;
    }
}
