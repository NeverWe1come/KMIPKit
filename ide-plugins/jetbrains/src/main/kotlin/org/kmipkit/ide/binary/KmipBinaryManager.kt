package org.kmipkit.ide.binary

import java.io.File
import java.security.MessageDigest
import java.util.Locale

object KmipBinaryManager {
    fun currentPlatformTarget(): String {
        val os = System.getProperty("os.name").lowercase(Locale.ROOT)
        val arch = System.getProperty("os.arch").lowercase(Locale.ROOT)

        val normOs = when {
            os.contains("win") -> "win32"
            os.contains("mac") || os.contains("darwin") -> "darwin"
            else -> "linux"
        }
        val normArch = when (arch) {
            "amd64", "x86_64" -> "x64"
            "aarch64", "arm64" -> "arm64"
            else -> arch
        }
        return "$normOs-$normArch"
    }

    fun computeSha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buf = ByteArray(8192)
            var read = input.read(buf)
            while (read != -1) {
                digest.update(buf, 0, read)
                read = input.read(buf)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }

    fun verifySha256(file: File, expectedHex: String): Boolean {
        return computeSha256(file).equals(expectedHex.trim(), ignoreCase = true)
    }

    fun resolveBinary(configuredPath: String, projectBasePath: String?): File {
        val exeName = if (System.getProperty("os.name").lowercase(Locale.ROOT).contains("win")) {
            "kmip-lsp.exe"
        } else {
            "kmip-lsp"
        }

        if (configuredPath.isNotBlank()) {
            val manual = File(configuredPath.trim())
            if (manual.isFile) return manual
            throw IllegalStateException("Configured kmip-lsp binary does not exist: ${manual.absolutePath}")
        }

        if (!projectBasePath.isNullOrBlank()) {
            for (profile in listOf("release", "debug")) {
                val devBin = File(projectBasePath, "ide-plugins/core/target/$profile/$exeName")
                if (devBin.isFile) return devBin
            }
        }

        val cacheDir = File(System.getProperty("user.home"), ".cache/kmipkit/bin/${currentPlatformTarget()}")
        val cached = File(cacheDir, exeName)
        if (cached.isFile) return cached

        throw IllegalStateException(
            "Could not locate kmip-lsp for ${currentPlatformTarget()}. Configure path in Settings -> Tools -> KMIPKit."
        )
    }
}
