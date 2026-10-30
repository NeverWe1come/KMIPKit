package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import org.junit.jupiter.api.Test;

final class ProfileValidationMappingTest {
    @Test
    void selectionAndContextRetainCallerOwnedValues() {
        var selection = new ProfileValidationSelection(List.of("baseline_client"));
        var state = new ProfileObjectState("object-7").withOriginalCreationDatePresent(false);
        var context = new ProfileValidationContext().withObjectState(0, state);

        assertEquals(List.of("baseline_client"), selection.profileIds());
        assertEquals(List.of(new ProfileValidationContext.Entry(0, state)), context.objectStates());
    }

    @Test
    void clientExposesExplicitProfileExchangeMethods() throws Exception {
        assertEquals(4, java.util.Arrays.stream(Client.class.getMethods())
                .filter(method -> method.getName().startsWith("executeWithProfiles"))
                .count());
    }
}
