package org.kmipkit.extensions;

import java.lang.ref.Reference;
import java.util.ArrayList;
import java.util.List;
import org.kmipkit.ResourceLimitException;

/** Child process used to verify Revoke text limits before encoding allocations. */
public final class RevokeTextLimitProbe {
    private RevokeTextLimitProbe() {}

    /** Runs one Revoke text conversion with a bounded heap and a value beyond 16 MiB. */
    public static void main(String[] arguments) {
        String inputKind = arguments[0];
        String oversized;
        List<byte[]> heapPressure = new ArrayList<>();
        try {
            oversized = "x".repeat(Integer.parseInt(arguments[1]));
            Runtime runtime = Runtime.getRuntime();
            long minimumHeadroom = 17 * 1024 * 1024L;
            while (remainingHeapCapacity(runtime) > minimumHeadroom) {
                heapPressure.add(new byte[1024 * 1024]);
            }
        } catch (OutOfMemoryError error) {
            System.out.println("probe-setup-allocation-failed");
            System.exit(24);
            return;
        }

        try {
            switch (inputKind) {
                case "identifier" -> UniqueIdentifier.textString(oversized).nativeText();
                case "message" -> RevocationReason.create(1, oversized);
                default -> throw new IllegalArgumentException("unknown probe input kind");
            }
            Reference.reachabilityFence(heapPressure);
            System.out.println("accepted");
            System.exit(25);
        } catch (ResourceLimitException exception) {
            Reference.reachabilityFence(heapPressure);
            System.out.println("resource-limit");
        } catch (OutOfMemoryError error) {
            Reference.reachabilityFence(heapPressure);
            System.out.println("encoding-allocation-failed");
            System.exit(23);
        }
    }

    private static long remainingHeapCapacity(Runtime runtime) {
        long usedMemory = runtime.totalMemory() - runtime.freeMemory();
        return runtime.maxMemory() - usedMemory;
    }
}
