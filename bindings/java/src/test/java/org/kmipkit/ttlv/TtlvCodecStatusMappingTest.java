package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import java.lang.reflect.Method;
import java.util.Arrays;

import org.junit.jupiter.api.Test;
import org.kmipkit.ResourceLimitException;

final class TtlvCodecStatusMappingTest {
    @Test
    void statusOnlyResourceLimitMapsToResourceLimitException() throws ReflectiveOperationException {
        Method statusFailure = Arrays.stream(TtlvCodec.class.getDeclaredMethods())
                .filter(method -> method.getName().equals("statusFailure"))
                .findFirst()
                .orElse(null);
        assertNotNull(statusFailure, "status-only decoder failures need a typed mapper");

        RuntimeException mapped = (RuntimeException) statusFailure.invoke(null, 6);
        assertInstanceOf(ResourceLimitException.class, mapped);
    }
}
