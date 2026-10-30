package org.kmipkit.ttlv;

import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.Arrays;

import org.junit.jupiter.api.Test;
import org.kmipkit.ResourceLimitException;

final class TtlvCodecStatusMappingTest {
    @Test
    void statusOnlyResourceLimitMapsFromDecoderResult() throws ReflectiveOperationException {
        Method resultMapper = Arrays.stream(TtlvCodec.class.getDeclaredMethods())
                .filter(method -> method.getName().equals("decodeResult"))
                .findFirst()
                .orElse(null);
        assertNotNull(resultMapper, "JNI results need a shared typed-error mapper");

        InvocationTargetException failure = assertThrows(
                InvocationTargetException.class,
                () -> resultMapper.invoke(null, (Object) new long[] {0, 0, 0, 6}));
        assertInstanceOf(ResourceLimitException.class, failure.getCause());
    }
}
