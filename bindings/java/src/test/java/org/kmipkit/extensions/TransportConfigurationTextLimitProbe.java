package org.kmipkit.extensions;

import java.lang.ref.Reference;
import java.util.ArrayList;
import java.util.List;
import org.kmipkit.ResourceLimitException;

/** Child process used to verify oversized text is rejected under a constrained heap. */
public final class TransportConfigurationTextLimitProbe {
    private TransportConfigurationTextLimitProbe() {}

    /** Runs the real TLS-name setter with an input larger than the configured limit. */
    public static void main(String[] arguments) {
        String oversized;
        List<byte[]> heapPressure = new ArrayList<>();
        try {
            oversized = "x".repeat(Integer.parseInt(arguments[0]));
            Runtime runtime = Runtime.getRuntime();
            long minimumHeadroom = 17 * 1024 * 1024L;
            while (runtime.maxMemory() - runtime.totalMemory() + runtime.freeMemory() > minimumHeadroom) {
                heapPressure.add(new byte[1024 * 1024]);
            }
        } catch (OutOfMemoryError error) {
            System.out.println("probe-setup-allocation-failed");
            System.exit(24);
            return;
        }

        try {
            TransportConfiguration.builder().tlsServerName(oversized);
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
}
