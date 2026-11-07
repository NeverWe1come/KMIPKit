package org.kmipkit.validator;

import java.util.function.Supplier;

public class RecipeRunner {
    public static <T> T executeWithGuaranteedTeardown(
            Supplier<Boolean> setup,
            Supplier<T> target,
            Supplier<Boolean> teardown
    ) {
        if (setup != null && !Boolean.TRUE.equals(setup.get())) {
            throw new IllegalStateException("Setup phase failed");
        }

        try {
            return (target != null) ? target.get() : null;
        } finally {
            if (teardown != null) {
                teardown.get();
            }
        }
    }
}
