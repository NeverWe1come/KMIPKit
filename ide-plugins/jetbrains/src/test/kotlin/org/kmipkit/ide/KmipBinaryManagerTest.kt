package org.kmipkit.ide

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import org.kmipkit.ide.binary.KmipBinaryManager
import java.io.File
import java.security.MessageDigest

class KmipBinaryManagerTest {
    @Test
    fun verifySha256DetectsMatchAndMismatch() {
        val tmp = File.createTempFile("kmip-lsp-test", ".bin")
        try {
            val bytes = "kmipkit-core-binary".toByteArray()
            tmp.writeBytes(bytes)
            val expected = MessageDigest.getInstance("SHA-256")
                .digest(bytes)
                .joinToString("") { "%02x".format(it) }

            assertTrue(KmipBinaryManager.verifySha256(tmp, expected))
            assertFalse(KmipBinaryManager.verifySha256(tmp, "0".repeat(64)))
        } finally {
            tmp.delete()
        }
    }
}
