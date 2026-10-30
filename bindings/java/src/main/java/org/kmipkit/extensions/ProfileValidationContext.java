package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

/** Caller-owned remote state supplied only for one profile-validated exchange. */
public record ProfileValidationContext(List<Entry> objectStates) {
    public ProfileValidationContext {
        if (objectStates == null) throw new InvalidInputException("profile context is invalid");
        if (objectStates.size() > 100_000) {
            throw new ResourceLimitException("profile context exceeds the configured limit");
        }
        objectStates = List.copyOf(objectStates);
    }

    public ProfileValidationContext() {
        this(List.of());
    }

    /** Returns a copy with caller state bound to one zero-based batch item. */
    public ProfileValidationContext withObjectState(long batchItemIndex, ProfileObjectState state) {
        if (batchItemIndex < 0 || state == null) {
            throw new InvalidInputException("profile context is invalid");
        }
        if (objectStates.size() >= 100_000) {
            throw new ResourceLimitException("profile context exceeds the configured limit");
        }
        List<Entry> values = new ArrayList<>(objectStates);
        values.add(new Entry(batchItemIndex, state));
        return new ProfileValidationContext(values);
    }

    @Override
    public String toString() {
        return "ProfileValidationContext[objectStateCount=" + objectStates.size() + "]";
    }

    /** One caller-attested object state entry. */
    public record Entry(long batchItemIndex, ProfileObjectState state) {
        public Entry {
            if (batchItemIndex < 0 || state == null) {
                throw new InvalidInputException("profile context entry is invalid");
            }
        }

        @Override
        public String toString() {
            return "Entry[batchItemIndex=" + batchItemIndex + ", state=[REDACTED]]";
        }
    }
}
