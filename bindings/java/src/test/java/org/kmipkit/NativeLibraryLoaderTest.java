package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.AclEntryPermission;
import java.nio.file.attribute.AclEntryType;
import java.nio.file.attribute.AclFileAttributeView;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.PosixFilePermission;
import java.security.MessageDigest;
import java.util.EnumSet;
import java.util.HexFormat;
import java.util.Map;
import java.util.Set;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

final class NativeLibraryLoaderTest {
    private static final String WINDOWS_TARGET = "windows-x86_64-msvc";
    private static final String WINDOWS_LIBRARY = "kmipkit_jni.dll";

    @TempDir
    Path temporaryDirectory;

    @Test
    void selectsOnlyPublishedNativeTargets() {
        assertEquals(WINDOWS_TARGET, NativeLibraryLoader.targetIdFor("Windows 11", "amd64"));
        assertEquals("linux-glibc-x86_64", NativeLibraryLoader.targetIdFor("Linux", "x86_64"));
        assertEquals("linux-glibc-aarch64", NativeLibraryLoader.targetIdFor("Linux", "aarch64"));
        assertEquals("macos-x86_64", NativeLibraryLoader.targetIdFor("Mac OS X", "x86_64"));
        assertEquals("macos-aarch64", NativeLibraryLoader.targetIdFor("Darwin", "arm64"));
        assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.targetIdFor("Linux", "riscv64"));
        assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.targetIdFor("FreeBSD", "x86_64"));
    }

    @Test
    void resolvesTargetLibraryAndDigestFromBundledResources() throws Exception {
        byte[] libraryBytes = {0x4b, 0x4d, 0x49, 0x50};
        Map<String, byte[]> resources = Map.of(
                "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY, libraryBytes,
                "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY + ".sha256",
                (sha256(libraryBytes) + "\n").getBytes(java.nio.charset.StandardCharsets.US_ASCII));

        Path loadedPath = NativeLibraryLoader.extractBundled(
                classLoader(resources), "Windows 11", "amd64", temporaryDirectory,
                NativeLibraryLoader::createPrivateDirectory);

        assertArrayEquals(libraryBytes, Files.readAllBytes(loadedPath));
        assertEquals(temporaryDirectory.toAbsolutePath().normalize(),
                loadedPath.getParent().getParent().toAbsolutePath().normalize());
        BasicFileAttributes attributes = Files.readAttributes(loadedPath, BasicFileAttributes.class);
        assertTrue(attributes.isRegularFile());
    }

    @Test
    void missingResourceFailsWithRedactedError() {
        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractBundled(
                        classLoader(Map.of()), "Windows 11", "amd64", temporaryDirectory,
                        NativeLibraryLoader::createPrivateDirectory));

        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertEquals(null, failure.getCause());
    }

    @Test
    void corruptBundledLibraryIsRejectedBeforeItCanBeLoaded() throws Exception {
        byte[] libraryBytes = {1, 2, 3, 4};
        Map<String, byte[]> resources = Map.of(
                "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY, libraryBytes,
                "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY + ".sha256",
                ("0".repeat(64) + "\n").getBytes(java.nio.charset.StandardCharsets.US_ASCII));

        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractBundled(
                        classLoader(resources), "Windows 11", "amd64", temporaryDirectory,
                        NativeLibraryLoader::createPrivateDirectory));

        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertEquals(null, failure.getCause());
        try (var files = Files.list(temporaryDirectory)) {
            assertEquals(0, files.count());
        }
    }

    @Test
    void existingDestinationIsNeverReplaced() throws Exception {
        byte[] expectedContent = {10, 11};
        byte[] libraryBytes = {20, 21, 22};
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        Path destination = privateDirectory.resolve(WINDOWS_LIBRARY);
        Files.write(destination, expectedContent);

        assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        new ByteArrayInputStream(libraryBytes), sha256(libraryBytes),
                        privateDirectory, WINDOWS_LIBRARY));

        assertArrayEquals(expectedContent, Files.readAllBytes(destination));
    }

    @Test
    void extractionUsesFreshPrivateDirectoryAndOwnerOnlyFiles() throws Exception {
        byte[] libraryBytes = {9, 8, 7};
        Map<String, byte[]> resources = resources(libraryBytes, sha256(libraryBytes));
        Path first = NativeLibraryLoader.extractBundled(
                classLoader(resources), "Windows 11", "amd64", temporaryDirectory,
                NativeLibraryLoader::createPrivateDirectory);
        Path second = NativeLibraryLoader.extractBundled(
                classLoader(resources), "Windows 11", "amd64", temporaryDirectory,
                NativeLibraryLoader::createPrivateDirectory);

        assertNotEquals(first, second);
        if (Files.getFileStore(first).supportsFileAttributeView("posix")) {
            Set<PosixFilePermission> directoryPermissions = Files.getPosixFilePermissions(first.getParent());
            Set<PosixFilePermission> filePermissions = Files.getPosixFilePermissions(first);
            assertEquals(Set.of(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE,
                    PosixFilePermission.OWNER_EXECUTE), directoryPermissions);
            assertEquals(Set.of(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE),
                    filePermissions);
        } else if (Files.getFileStore(first).supportsFileAttributeView("acl")) {
            assertPrivateAcl(first.getParent());
            assertPrivateAcl(first);
        }
    }

    @Test
    void permissionPolicyFailureFailsClosedWithoutExposingDetails() {
        byte[] libraryBytes = {1, 3, 3, 7};
        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractBundled(
                        classLoader(resources(libraryBytes, sha256(libraryBytes))),
                        "Windows 11", "amd64", temporaryDirectory,
                        ignored -> { throw new IOException("private ACL setup details"); }));

        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertEquals(null, failure.getCause());
        assertTrue(Files.isDirectory(temporaryDirectory));
    }

    @Test
    void rejectsUnsupportedNativeAbiVersionsWithRedactedError() {
        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.validateNativeAbiVersion(2));

        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertEquals(null, failure.getCause());
    }

    @Test
    void rejectsPresentButBlankNativeLibraryOverrideInsteadOfFallingBack() {
        UnsatisfiedLinkError failure = assertThrows(
                UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.resolveExplicitLibrary("  "));
        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertEquals(null, failure.getCause());
    }

    private static ClassLoader classLoader(Map<String, byte[]> resources) {
        return new ClassLoader(null) {
            @Override
            public InputStream getResourceAsStream(String name) {
                byte[] contents = resources.get(name);
                return contents == null ? null : new ByteArrayInputStream(contents);
            }
        };
    }

    private static Map<String, byte[]> resources(byte[] libraryBytes, String hash) {
        return Map.of(
                "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY, libraryBytes,
                "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY + ".sha256",
                (hash + "\n").getBytes(java.nio.charset.StandardCharsets.US_ASCII));
    }

    private static String sha256(byte[] bytes) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
    }

    private static void assertPrivateAcl(Path path) throws IOException {
        AclFileAttributeView view = Files.getFileAttributeView(path, AclFileAttributeView.class);
        assertNotNull(view);
        var owner = Files.getOwner(path);
        var entries = view.getAcl();
        assertEquals(1, entries.size());
        assertEquals(AclEntryType.ALLOW, entries.get(0).type());
        assertEquals(owner, entries.get(0).principal());
        assertEquals(EnumSet.allOf(AclEntryPermission.class), entries.get(0).permissions());
        assertTrue(entries.get(0).flags().isEmpty());
    }
}
