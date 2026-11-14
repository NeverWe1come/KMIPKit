package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Arrays;
import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.AttributeSet;

/** Verifies the public namespace exposes every assigned KMIP 3.0 fluent route. */
final class Kmip30FluentApiShapeTest {
    @Test
    void createCredentialInfersItsTypeFromTheClosedObjectAlternative() throws Exception {
        var method = Arrays.stream(FluentRequest.class.getMethods())
                .filter(candidate -> candidate.getName().equals("createCredential"))
                .findFirst()
                .orElseThrow();
        assertEquals(2, method.getParameterCount());
        assertEquals(AttributeSet.class, method.getParameterTypes()[0]);
        assertEquals(CreateCredentialObject.class, method.getParameterTypes()[1]);
    }

    @Test
    void requestNamespaceProvidesAllKmip30Builders() {
        String[] names = {
            "createCredential", "createGroup", "createUser", "deactivate",
            "decapsulate", "encapsulate", "obliterate"
        };
        for (String name : names) {
            assertTrue(Arrays.stream(FluentRequest.class.getMethods())
                    .anyMatch(method -> method.getName().equals(name)),
                    () -> "missing fluent request method: " + name);
        }
    }
}
