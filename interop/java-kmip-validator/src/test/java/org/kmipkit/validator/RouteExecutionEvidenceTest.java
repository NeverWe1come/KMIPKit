package org.kmipkit.validator;

import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvDecodeErrorKind;
import org.kmipkit.ttlv.TtlvDecodeException;

import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class RouteExecutionEvidenceTest {
    private static final byte[] UNALLOCATED_TAG_INTEGER = {
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00
    };

    @Test
    void installedDecoderRejectsTheUnallocatedTagForBothProtocolVersions() throws Exception {
        var provenance = ArtifactProvenance.load(Path.of(System.getProperty("kmipkit.artifactManifest")),
                "9a16cc90a12205101d22a38501a5e74c665df4dc");
        Path loadedJar = Path.of(TtlvCodec.class.getProtectionDomain().getCodeSource().getLocation().toURI())
                .toAbsolutePath().normalize();
        assertEquals(provenance.artifactPath(), loadedJar,
                "The decoder invocation must resolve to the independently installed package JAR.");

        for (String protocolVersion : new String[] {"2.0", "2.1"}) {
            TtlvDecodeException rejection = assertThrows(TtlvDecodeException.class,
                    () -> TtlvCodec.decode(UNALLOCATED_TAG_INTEGER), protocolVersion);
            assertEquals(TtlvDecodeErrorKind.UNALLOCATED_TAG, rejection.kind(), protocolVersion);
        }
    }
}
