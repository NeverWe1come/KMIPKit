package org.kmipkit.server;

import java.time.Duration;
import org.kmipkit.InvalidInputException;

final class ServerTimeout {
    private ServerTimeout() {}

    static long[] nativeValues(Duration timeout) {
        if (timeout == null) return new long[] {0, 0};
        if (timeout.isNegative()) throw new InvalidInputException("timeout cannot be negative");
        return new long[] {timeout.getSeconds(), timeout.getNano()};
    }
}
