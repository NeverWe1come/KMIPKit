package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.ClientOperationException;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.protocol.KmipValues;

final class CreateKeyPairBuilderTest {
    @Test
    void fluentRequestExposesCreateKeyPairBuilder() throws Exception {
        assertEquals(CreateKeyPairBuilder.class,
                FluentRequest.class.getMethod("createKeyPair").getReturnType());
        assertNotNull(CreateKeyPairBuilder.class.getMethod("privateUsageMask", int.class));
        assertNotNull(CreateKeyPairBuilder.class.getMethod("publicUsageMask", int.class));
    }

    @Test
    void builderRejectsMissingCryptoChoicesAndBuildsTypedBatchItem() {
        try (CreateKeyPairBuilder builder = new CreateKeyPairBuilder(null)) {
            assertThrows(ClientOperationException.class, builder::build);
            builder.algorithm(KmipValues.CryptographicAlgorithm.RSA)
                    .length(2048)
                    .privateUsageMask((int) KmipValues.CryptographicUsageMask.SIGN)
                    .publicUsageMask((int) KmipValues.CryptographicUsageMask.VERIFY);
            builder.validate();
            try (ClientBatchItem item = builder.build()) {
                assertEquals(KmipValues.Operation.CREATE_KEY_PAIR, item.operation());
            }
        }
    }
}
