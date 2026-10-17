package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayInputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.lang.management.ManagementFactory;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.FileSystem;
import java.nio.file.FileSystems;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.AclEntry;
import java.nio.file.attribute.AclEntryPermission;
import java.nio.file.attribute.AclEntryType;
import java.nio.file.attribute.AclFileAttributeView;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.PosixFilePermission;
import java.security.MessageDigest;
import java.util.EnumSet;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.Function;
import javax.tools.JavaCompiler;
import javax.tools.ToolProvider;

import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.kmipkit.extensions.ClientBatchItem;

final class NativeLibraryLoaderTest {
    private static final String WINDOWS_TARGET = "windows-x86_64-msvc";
    private static final String WINDOWS_LIBRARY = "kmipkit_jni.dll";
    private static final int MAX_OPERATION_INPUT_BYTES = 16 * 1024 * 1024;

    @TempDir
    Path temporaryDirectory;

    @Test
    void exactLimitOperationInputsAreAcceptedWithoutMutatingCallerArrays() {
        byte[] input = new byte[MAX_OPERATION_INPUT_BYTES];
        java.util.Arrays.fill(input, (byte) 0x5a);
        byte[] original = input.clone();
        List<Function<byte[], ClientBatchItem>> factories = List.of(
                value -> ClientBatchItem.certify(null, null, value, null, null),
                value -> ClientBatchItem.macVerify(null, null, null, value, null, null, null),
                value -> ClientBatchItem.sign(null, null, null, value, null, null, null),
                value -> ClientBatchItem.signatureVerify(null, null, null, value, null, null, null, null),
                value -> ClientBatchItem.signatureVerify(null, null, null, null, value, null, null, null));

        for (Function<byte[], ClientBatchItem> factory : factories) {
            try (ClientBatchItem item = factory.apply(input)) {
                assertNotNull(item);
            }
            assertArrayEquals(original, input);
        }
    }

    @Test
    void oversizedOperationInputsAreRejectedBeforeJni() throws Exception {
        Path testClasses = classLocation(NativeLibraryLoadProbe.class);
        Path mainClasses = classLocation(NativeExtensionRegistry.class);
        Path nullableClasses = classLocation(org.jspecify.annotations.Nullable.class);
        Path java = Path.of(System.getProperty("java.home"), "bin",
                System.getProperty("os.name", "").startsWith("Windows") ? "java.exe" : "java");
        String classPath = String.join(File.pathSeparator,
                testClasses.toString(), mainClasses.toString(), nullableClasses.toString());
        Path unavailableLibrary = temporaryDirectory.resolve("native-library-must-not-load.dll");
        List<String> command = new java.util.ArrayList<>(List.of(
                java.toString(), "-Dkmipkit.nativeLibrary=" + unavailableLibrary,
                "--class-path", classPath, NativeLibraryLoadProbe.class.getName(), "operation-input-limit"));
        String jacocoAgent = childJacocoAgentArgument(ManagementFactory.getRuntimeMXBean().getInputArguments());
        if (jacocoAgent != null) {
            command.add(1, jacocoAgent);
        }

        Process process = new ProcessBuilder(command).redirectErrorStream(true).start();
        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);

        assertEquals(0, process.waitFor(), output);
        assertTrue(output.contains("OPERATION_INPUT_LIMIT_REJECTED_BEFORE_NATIVE_LOAD"), output);
    }

    @Test
    void selectsOnlyPublishedNativeTargets() {
        assertEquals(WINDOWS_TARGET, NativeLibraryLoader.targetIdFor("Windows 11", "amd64"));
        assertEquals("linux-glibc-x86_64", NativeLibraryLoader.targetIdFor("Linux", "x86_64"));
        assertEquals("linux-glibc-aarch64", NativeLibraryLoader.targetIdFor("Linux", "aarch64"));
        assertEquals("macos-x86_64", NativeLibraryLoader.targetIdFor("Mac OS X", "x86_64"));
        assertEquals("macos-aarch64", NativeLibraryLoader.targetIdFor("Darwin", "arm64"));
        UnsatisfiedLinkError unsupportedArchitecture = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.targetIdFor("Linux", "riscv64"));
        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, unsupportedArchitecture.getMessage());
        assertNull(unsupportedArchitecture.getCause());

        UnsatisfiedLinkError unsupportedOperatingSystem = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.targetIdFor("FreeBSD", "x86_64"));
        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, unsupportedOperatingSystem.getMessage());
        assertNull(unsupportedOperatingSystem.getCause());
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
    void resolvesUnixLibraryNameFromBundledResources() throws Exception {
        byte[] libraryBytes = {0x4b, 0x4d, 0x49, 0x50};
        String[][] targets = {
            {"Linux", "amd64", "linux-glibc-x86_64", "libkmipkit_jni.so"},
            {"Linux", "aarch64", "linux-glibc-aarch64", "libkmipkit_jni.so"},
            {"Mac OS X", "x86_64", "macos-x86_64", "libkmipkit_jni.dylib"},
            {"Darwin", "arm64", "macos-aarch64", "libkmipkit_jni.dylib"},
        };

        for (String[] target : targets) {
            String resource = "META-INF/native/" + target[2] + "/" + target[3];
            Map<String, byte[]> resources = Map.of(
                    resource, libraryBytes,
                    resource + ".sha256",
                    (sha256(libraryBytes) + "\n").getBytes(java.nio.charset.StandardCharsets.US_ASCII));
            Path loadedPath = NativeLibraryLoader.extractBundled(
                    classLoader(resources), target[0], target[1], temporaryDirectory,
                    NativeLibraryLoader::createPrivateDirectory);

            assertEquals(target[3], loadedPath.getFileName().toString());
            assertArrayEquals(libraryBytes, Files.readAllBytes(loadedPath));
        }
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
    void malformedDigestMetadataFailsBeforeCreatingPrivateDirectory() throws Exception {
        byte[] libraryBytes = {4, 2};
        String resourceName = "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY;
        String[] malformedDigests = {
            "f".repeat(63),
            "f".repeat(64) + "x",
            "f".repeat(64) + "xyz",
            "f".repeat(64) + "\r",
            "F".repeat(64),
            "g".repeat(64),
        };

        for (String malformedDigest : malformedDigests) {
            UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractBundled(
                            classLoader(Map.of(
                                    resourceName, libraryBytes,
                                    resourceName + ".sha256",
                                    malformedDigest.getBytes(java.nio.charset.StandardCharsets.US_ASCII))),
                            "Windows 11", "amd64", temporaryDirectory,
                            NativeLibraryLoader::createPrivateDirectory));
            assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
            assertNull(failure.getCause());
        }

        try (var files = Files.list(temporaryDirectory)) {
            assertEquals(0, files.count());
        }
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
    void rejectsDirectoryPermissionsThatAreNotPrivate() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] libraryBytes = {6, 2, 6};
        var fileStore = Files.getFileStore(privateDirectory);
        Set<PosixFilePermission> originalPosixPermissions = null;
        AclFileAttributeView aclView = null;
        java.util.List<AclEntry> originalAcl = null;
        try {
            if (fileStore.supportsFileAttributeView("posix")) {
                originalPosixPermissions = Files.getPosixFilePermissions(privateDirectory);
                Set<PosixFilePermission> broaderPermissions = new java.util.HashSet<>(originalPosixPermissions);
                broaderPermissions.add(PosixFilePermission.OTHERS_READ);
                Files.setPosixFilePermissions(privateDirectory, broaderPermissions);
            } else if (fileStore.supportsFileAttributeView("acl")) {
                aclView = Files.getFileAttributeView(privateDirectory, AclFileAttributeView.class);
                originalAcl = java.util.List.copyOf(aclView.getAcl());
                AclEntry extraEntry = AclEntry.newBuilder()
                        .setType(AclEntryType.DENY)
                        .setPrincipal(Files.getOwner(privateDirectory))
                        .setPermissions(EnumSet.of(AclEntryPermission.READ_DATA))
                        .build();
                java.util.List<AclEntry> broaderAcl = new java.util.ArrayList<>(originalAcl);
                broaderAcl.add(extraEntry);
                aclView.setAcl(broaderAcl);
            } else {
                return;
            }

            UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractVerified(
                            new ByteArrayInputStream(libraryBytes), sha256(libraryBytes),
                            privateDirectory, WINDOWS_LIBRARY));
            assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
            assertNull(failure.getCause());
        } finally {
            if (originalPosixPermissions != null) {
                Files.setPosixFilePermissions(privateDirectory, originalPosixPermissions);
            }
            if (aclView != null && originalAcl != null) {
                aclView.setAcl(originalAcl);
            }
        }
    }

    @Test
    void createPrivateDirectoryRejectsTemporaryRootThatIsNotADirectory() throws Exception {
        Path file = temporaryDirectory.resolve("not-a-temporary-directory");
        Files.write(file, new byte[] {1});

        assertThrows(IOException.class, () -> NativeLibraryLoader.createPrivateDirectory(file));
        assertTrue(Files.isRegularFile(file));
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

        assertEquals(NativeLibraryLoader.ABI_FAILURE_MESSAGE, failure.getMessage());
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

    @Test
    void explicitNativeLibraryOverrideMustBeAbsolute() {
        Path absolute = temporaryDirectory.resolve("native.dll").toAbsolutePath().normalize();
        assertEquals(absolute, NativeLibraryLoader.resolveExplicitLibrary(absolute.toString()));

        for (String relative : new String[] {"native.dll", "subdir/native.dll"}) {
            UnsatisfiedLinkError failure = assertThrows(
                    UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.resolveExplicitLibrary(relative));
            assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
            assertNull(failure.getCause());
        }
    }

    @Test
    void preservesAbiMismatchWhenLoadingAnExplicitNativeLibrary() throws Exception {
        Path fixture = Path.of("..", "..", "tools", "native_package_smoke", "wrong_abi.rs")
                .toAbsolutePath().normalize();
        assertTrue(Files.isRegularFile(fixture), "The wrong-ABI native fixture must exist");
        Path library = temporaryDirectory.resolve(wrongAbiLibraryName()).toAbsolutePath();
        Process compiler = new ProcessBuilder("rustc", "--edition=2024", "--crate-name",
                "kmipkit_wrong_abi", "--crate-type", "cdylib", fixture.toString(), "-o",
                library.toString()).redirectErrorStream(true).start();
        String compilerOutput = new String(compiler.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        assertEquals(0, compiler.waitFor(), compilerOutput);

        Path testClasses = classLocation(NativeLibraryLoadProbe.class);
        Path mainClasses = classLocation(NativeExtensionRegistry.class);
        Path java = Path.of(System.getProperty("java.home"), "bin",
                System.getProperty("os.name", "").startsWith("Windows") ? "java.exe" : "java");
        String classPath = String.join(File.pathSeparator, testClasses.toString(), mainClasses.toString());
        List<String> command = new java.util.ArrayList<>(List.of(
                java.toString(), "-Dkmipkit.nativeLibrary=" + library,
                "--class-path", classPath, NativeLibraryLoadProbe.class.getName(), "abi-mismatch"));
        String jacocoAgent = childJacocoAgentArgument(ManagementFactory.getRuntimeMXBean().getInputArguments());
        if (jacocoAgent != null) {
            command.add(1, jacocoAgent);
        }

        Process process = new ProcessBuilder(command).redirectErrorStream(true).start();
        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);

        assertNotEquals(0, process.waitFor(), output);
        assertTrue(output.contains(NativeLibraryLoader.ABI_FAILURE_MESSAGE), output);
        assertFalse(output.contains(NativeLibraryLoader.LOAD_FAILURE_MESSAGE), output);
    }

    @Test
    void loadRedactsExplicitLibraryLoadingFailure() {
        String previous = System.getProperty("kmipkit.nativeLibrary");
        String invalidLibrary = temporaryDirectory.resolve("missing-native-library.dll").toString();
        try {
            System.setProperty("kmipkit.nativeLibrary", invalidLibrary);
            UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class, NativeLibraryLoader::load);
            assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
            assertNull(failure.getCause());
            assertFalse(failure.getMessage().contains(invalidLibrary));
        } finally {
            restoreProperty("kmipkit.nativeLibrary", previous);
        }
    }

    @Test
    void loadRedactsInvalidTemporaryPathFailures() {
        String previousLibrary = System.getProperty("kmipkit.nativeLibrary");
        String previousTemporaryDirectory = System.getProperty("java.io.tmpdir");
        try {
            System.clearProperty("kmipkit.nativeLibrary");
            System.setProperty("java.io.tmpdir", "invalid\u0000temporary-directory");
            UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class, NativeLibraryLoader::load);
            assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
            assertNull(failure.getCause());
        } finally {
            restoreProperty("kmipkit.nativeLibrary", previousLibrary);
            restoreProperty("java.io.tmpdir", previousTemporaryDirectory);
        }
    }

    @Test
    void loadsVerifiedBundledLibraryInAnIsolatedJvmWithoutAnExplicitOverride() throws Exception {
        Path nativeLibrary = Path.of(System.getProperty("kmipkit.nativeLibrary", ""));
        assertTrue(Files.isRegularFile(nativeLibrary), "Maven must build the JNI library before this test");
        byte[] nativeBytes = Files.readAllBytes(nativeLibrary);
        String targetId = NativeLibraryLoader.targetIdFor(
                System.getProperty("os.name", ""), System.getProperty("os.arch", ""));
        Path resourceRoot = Files.createDirectory(temporaryDirectory.resolve("bundled-classpath"));
        Path resourceLibrary = resourceRoot.resolve("META-INF/native")
                .resolve(targetId).resolve(nativeLibrary.getFileName());
        Files.createDirectories(resourceLibrary.getParent());
        Files.write(resourceLibrary, nativeBytes);
        Files.writeString(resourceLibrary.resolveSibling(resourceLibrary.getFileName() + ".sha256"),
                sha256(nativeBytes) + "\n", StandardCharsets.US_ASCII);

        Path childTemporaryRoot = Files.createDirectory(temporaryDirectory.resolve("child-tmp"));
        Path testClasses = classLocation(NativeLibraryLoadProbe.class);
        Path mainClasses = classLocation(NativeExtensionRegistry.class);
        Path java = Path.of(System.getProperty("java.home"), "bin",
                System.getProperty("os.name", "").startsWith("Windows") ? "java.exe" : "java");
        String classPath = String.join(File.pathSeparator,
                resourceRoot.toString(), testClasses.toString(), mainClasses.toString());
        List<String> command = new java.util.ArrayList<>(List.of(
                java.toString(), "-Djava.io.tmpdir=" + childTemporaryRoot,
                "--class-path", classPath, NativeLibraryLoadProbe.class.getName()));
        String jacocoAgent = childJacocoAgentArgument(ManagementFactory.getRuntimeMXBean().getInputArguments());
        if (jacocoAgent != null) {
            command.add(1, jacocoAgent);
        }
        Process process = new ProcessBuilder(command)
                .redirectErrorStream(true)
                .start();
        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        int exitCode = process.waitFor();

        assertEquals(0, exitCode, output);
        assertTrue(output.contains("ABI=1"), output);
        try (var extractedDirectories = Files.list(childTemporaryRoot)) {
            List<Path> directories = extractedDirectories.toList();
            assertEquals(1, directories.size());
            assertTrue(directories.get(0).getFileName().toString().startsWith("kmipkit-native-"));
            assertTrue(Files.isRegularFile(directories.get(0).resolve(nativeLibrary.getFileName())));
        }
    }

    @Test
    void childJacocoAgentAppendsToTheActiveExecutionDataFile() {
        assertEquals(
                "-javaagent:/tools/jacoco-agent.jar=destfile=/tmp/kmipkit.exec,append=true",
                NativeLibraryLoaderTest.childJacocoAgentArgument(List.of(
                        "-Xmx256m",
                        "-javaagent:/tools/jacoco-agent.jar=destfile=/tmp/kmipkit.exec")));
    }

    @Test
    void nativeVersionLookupSurfacesMissingRecordClassFromJni() throws Exception {
        Path resourceRoot = bundledResourcesForChild();
        Path childTemporaryRoot = Files.createDirectory(temporaryDirectory.resolve("missing-version-tmp"));
        Path childMainClasses = copyMainClassesWithoutVersionRecord(
                temporaryDirectory.resolve("missing-version-classes"));

        ProcessResult result = runVersionProbe(resourceRoot, childTemporaryRoot, null, childMainClasses);

        assertNotEquals(0, result.exitCode());
        assertTrue(result.output().contains("NoClassDefFoundError"), result.output());
    }

    @Test
    void nativeVersionLookupSurfacesMissingRecordConstructorFromJni() throws Exception {
        Path resourceRoot = bundledResourcesForChild();
        Path childTemporaryRoot = Files.createDirectory(temporaryDirectory.resolve("missing-constructor-tmp"));
        Path childMainClasses = classLocation(NativeExtensionRegistry.class);
        Path classOverlay = compileVersionClassWithoutNativeConstructor(
                temporaryDirectory.resolve("version-constructor-overlay"));

        ProcessResult result = runVersionProbe(resourceRoot, childTemporaryRoot, classOverlay, childMainClasses);

        assertNotEquals(0, result.exitCode());
        assertTrue(result.output().contains("NoSuchMethodError"), result.output());
    }

    @Test
    void rejectsInvalidExtractionMetadataAndRemovesPartialFiles() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] libraryBytes = {3, 1, 4};

        for (String invalidName : new String[] {"", ".", "..", "nested/library.dll"}) {
            UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractVerified(
                            new ByteArrayInputStream(libraryBytes), sha256(libraryBytes),
                            privateDirectory, invalidName));
            assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
            assertNull(failure.getCause());
        }
        UnsatisfiedLinkError invalidHash = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        new ByteArrayInputStream(libraryBytes), "A".repeat(64),
                        privateDirectory, WINDOWS_LIBRARY));
        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, invalidHash.getMessage());
        assertNull(invalidHash.getCause());

        try (var files = Files.list(privateDirectory)) {
            assertEquals(0, files.count());
        }
    }

    @Test
    void extractionReadFailureCleansStagingFileAndRedactsCause() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        InputStream failingInput = new InputStream() {
            private boolean firstRead = true;

            @Override
            public int read() throws IOException {
                throw new IOException("simulated native contents read failure");
            }

            @Override
            public int read(byte[] buffer, int offset, int length) throws IOException {
                if (firstRead) {
                    firstRead = false;
                    buffer[offset] = 7;
                    return Math.min(1, length);
                }
                throw new IOException("simulated native contents read failure");
            }
        };

        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        failingInput, "0".repeat(64), privateDirectory, WINDOWS_LIBRARY));

        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertNull(failure.getCause());
        try (var files = Files.list(privateDirectory)) {
            assertEquals(0, files.count());
        }
    }

    @Test
    void allowsBundledSelectionOnlyWhenNativeLibraryOverrideIsAbsent() {
        assertNull(NativeLibraryLoader.resolveExplicitLibrary(null));
    }

    @Test
    void acceptsDigestResourcesWithoutNewlineOrWithCrLf() throws Exception {
        byte[] library = {5, 8, 13};
        String resource = "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY;
        for (String suffix : List.of("", "\r\n")) {
            Path extracted = NativeLibraryLoader.extractBundled(classLoader(Map.of(
                    resource, library,
                    resource + ".sha256", (sha256(library) + suffix)
                            .getBytes(java.nio.charset.StandardCharsets.US_ASCII))),
                    "Windows", "amd64", temporaryDirectory, NativeLibraryLoader::createPrivateDirectory);
            assertArrayEquals(library, Files.readAllBytes(extracted));
        }
    }

    @Test
    void rejectsMalformedCrLfDigestAndInvalidOverridePath() throws Exception {
        byte[] library = {4, 2};
        String resource = "META-INF/native/" + WINDOWS_TARGET + "/" + WINDOWS_LIBRARY;
        for (String suffix : List.of("\nx", "\rx")) {
            assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractBundled(classLoader(Map.of(
                            resource, library,
                            resource + ".sha256", (sha256(library) + suffix)
                                    .getBytes(java.nio.charset.StandardCharsets.US_ASCII))),
                            "Windows", "amd64", temporaryDirectory,
                            NativeLibraryLoader::createPrivateDirectory)));
        }
        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.resolveExplicitLibrary("invalid\u0000library")));
        assertDirectoryEmpty(temporaryDirectory);
    }

    @Test
    void directoryFactoryRuntimeFailureAndFailedCleanupRemainRedacted() throws Exception {
        byte[] library = {1, 2};
        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractBundled(classLoader(resources(library, sha256(library))),
                        "Windows", "amd64", temporaryDirectory,
                        ignored -> { throw new IllegalStateException("private filesystem details"); })));
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        Path retained = Files.write(directory.resolve("unrelated-file"), new byte[] {7});
        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractBundled(classLoader(resources(library, "0".repeat(64))),
                        "Windows", "amd64", temporaryDirectory, ignored -> directory)));
        assertArrayEquals(new byte[] {7}, Files.readAllBytes(retained));
        try (var files = Files.list(directory)) {
            assertEquals(List.of(retained), files.toList());
        }
    }

    @Test
    void destinationCreatedWhileReadingIsNeverReplaced() throws Exception {
        byte[] library = {2, 4, 6};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        Path destination = directory.resolve(WINDOWS_LIBRARY);
        InputStream source = sourceWithCloseAction(library,
                () -> Files.write(destination, new byte[] {99}));

        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(source, sha256(library), directory, WINDOWS_LIBRARY)));

        assertArrayEquals(new byte[] {99}, Files.readAllBytes(destination));
        assertDirectoryEmptyExcept(directory, WINDOWS_LIBRARY);
    }

    @Test
    void stagedFilePermissionChangeFailsBeforePublication() throws Exception {
        byte[] library = {3, 5, 8};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        InputStream source = sourceWithCloseAction(library, () -> {
            Path staging = stagingFile(directory);
            if (Files.getFileStore(staging).supportsFileAttributeView("posix")) {
                Files.setPosixFilePermissions(staging, Set.of(PosixFilePermission.OWNER_READ,
                        PosixFilePermission.OWNER_WRITE, PosixFilePermission.OTHERS_READ));
            } else {
                AclFileAttributeView view = Files.getFileAttributeView(staging, AclFileAttributeView.class);
                assertNotNull(view);
                var permissions = EnumSet.allOf(AclEntryPermission.class);
                permissions.remove(AclEntryPermission.EXECUTE);
                view.setAcl(List.of(AclEntry.newBuilder().setType(AclEntryType.ALLOW)
                        .setPrincipal(Files.getOwner(staging)).setPermissions(permissions).build()));
            }
        });

        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(source, sha256(library), directory, WINDOWS_LIBRARY)));

        assertDirectoryEmpty(directory);
    }

    @Test
    void stagedFileReplacedByDirectoryIsRejectedAndRemoved() throws Exception {
        byte[] library = {5, 8};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        InputStream source = sourceWithCloseAction(library, () -> {
            Path staging = stagingFile(directory);
            Files.delete(staging);
            Files.createDirectory(staging);
        });

        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(source, sha256(library), directory, WINDOWS_LIBRARY)));

        assertDirectoryEmpty(directory);
    }

    @Test
    void stagingCleanupFailureRemainsRedactedAndDoesNotPublishLibrary() throws Exception {
        byte[] library = {8, 13};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        InputStream source = sourceWithCloseAction(library, () -> {
            Path staging = stagingFile(directory);
            Files.delete(staging);
            Files.createDirectory(staging);
            Files.write(staging.resolve("retained"), new byte[] {42});
        });

        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(source, sha256(library), directory, WINDOWS_LIBRARY)));

        assertFalse(Files.exists(directory.resolve(WINDOWS_LIBRARY)));
        Path retainedStaging = stagingFile(directory);
        assertTrue(Files.isDirectory(retainedStaging));
        assertArrayEquals(new byte[] {42}, Files.readAllBytes(retainedStaging.resolve("retained")));
        try (var entries = Files.list(directory)) {
            assertEquals(1, entries.count());
        }
    }

    @Test
    void sourceCloseFailureRemovesStagingAndRedactsCause() throws Exception {
        byte[] library = {13, 21};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        InputStream source = sourceWithCloseAction(library,
                () -> { throw new IOException("secret resource close details"); });

        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(source, sha256(library), directory, WINDOWS_LIBRARY)));

        assertDirectoryEmpty(directory);
    }

    @Test
    void rejectsMissingOrNonDirectoryExtractionRoots() throws Exception {
        byte[] library = {1, 3};
        Path file = Files.write(temporaryDirectory.resolve("regular-file"), new byte[] {7});
        for (Path invalidRoot : List.of(file, temporaryDirectory.resolve("missing-directory"))) {
            assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractVerified(new ByteArrayInputStream(library),
                            sha256(library), invalidRoot, WINDOWS_LIBRARY)));
        }
        assertArrayEquals(new byte[] {7}, Files.readAllBytes(file));
    }

    @Test
    void rejectsSymbolicLinkDirectoryAndTemporaryRoot() throws Exception {
        byte[] library = {2, 5};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        Path linkedDirectory = temporaryDirectory.resolve("linked-directory");
        createSymbolicLinkOrSkip(linkedDirectory, directory);

        assertThrows(IOException.class, () -> NativeLibraryLoader.createPrivateDirectory(linkedDirectory));
        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(new ByteArrayInputStream(library),
                        sha256(library), linkedDirectory, WINDOWS_LIBRARY)));

        assertTrue(Files.isSymbolicLink(linkedDirectory));
        assertDirectoryEmpty(directory);
    }

    @Test
    void stagedFileReplacedBySymbolicLinkIsRejectedWithoutTouchingTarget() throws Exception {
        byte[] library = {3, 7};
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        Path target = Files.write(temporaryDirectory.resolve("link-target"), new byte[] {99});
        Path probe = temporaryDirectory.resolve("link-capability-probe");
        createSymbolicLinkOrSkip(probe, target);
        Files.delete(probe);
        InputStream source = sourceWithCloseAction(library, () -> {
            Path staging = stagingFile(directory);
            Files.delete(staging);
            Files.createSymbolicLink(staging, target);
        });

        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(source, sha256(library), directory, WINDOWS_LIBRARY)));

        assertArrayEquals(new byte[] {99}, Files.readAllBytes(target));
        assertDirectoryEmpty(directory);
    }

    @Test
    void filesystemWithoutPrivatePermissionSupportFailsClosed() throws Exception {
        Path archive = temporaryDirectory.resolve("no-private-permissions.zip");
        try (FileSystem filesystem = FileSystems.newFileSystem(URI.create("jar:" + archive.toUri()),
                Map.of("create", "true"))) {
            IOException failure = assertThrows(IOException.class,
                    () -> NativeLibraryLoader.createPrivateDirectory(filesystem.getPath("/")));
            assertEquals("Private permissions are unavailable", failure.getMessage());
            assertDirectoryEmpty(filesystem.getPath("/"));
            IOException privateFileFailure = assertThrows(IOException.class,
                    () -> NativeLibraryLoader.privateFileAttribute(filesystem.getPath("/")));
            assertEquals("Private permissions are unavailable", privateFileFailure.getMessage());

            byte[] library = {1, 2};
            assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractVerified(new ByteArrayInputStream(library),
                            sha256(library), filesystem.getPath("/"), WINDOWS_LIBRARY)));
            assertDirectoryEmpty(filesystem.getPath("/"));
        }
    }

    @Test
    void aclDirectoryVerificationFailureDeletesTheCreatedDirectory() throws Exception {
        var fileStore = Files.getFileStore(temporaryDirectory);
        Assumptions.assumeTrue(
                !fileStore.supportsFileAttributeView("posix") && fileStore.supportsFileAttributeView("acl"),
                "This cleanup path applies to ACL-only file stores");

        IOException failure = assertThrows(IOException.class,
                () -> NativeLibraryLoader.createPrivateDirectory(temporaryDirectory,
                        ignored -> { throw new IOException("synthetic ACL verification failure"); }));

        assertEquals("synthetic ACL verification failure", failure.getMessage());
        assertDirectoryEmpty(temporaryDirectory);
    }

    @Test
    void posixDirectoryVerificationFailureDeletesTheCreatedDirectory() throws Exception {
        Assumptions.assumeTrue(Files.getFileStore(temporaryDirectory).supportsFileAttributeView("posix"));

        IOException failure = assertThrows(IOException.class,
                () -> NativeLibraryLoader.createPrivateDirectory(temporaryDirectory,
                        ignored -> { throw new IOException("synthetic POSIX verification failure"); }));

        assertEquals("synthetic POSIX verification failure", failure.getMessage());
        assertDirectoryEmpty(temporaryDirectory);
    }

    @Test
    void failedDirectoryCleanupIsSuppressedOnTheOriginalVerificationFailure() throws Exception {
        IOException verificationFailure = new IOException("synthetic directory verification failure");
        Path[] retainedFile = new Path[1];

        IOException failure = assertThrows(IOException.class,
                () -> NativeLibraryLoader.createPrivateDirectory(temporaryDirectory, directory -> {
                    retainedFile[0] = Files.write(directory.resolve("retained"), new byte[] {7});
                    throw verificationFailure;
                }));

        assertEquals("synthetic directory verification failure", failure.getMessage());
        assertEquals(1, failure.getSuppressed().length);
        assertTrue(Files.isRegularFile(retainedFile[0]));
        Files.delete(retainedFile[0]);
        Files.delete(retainedFile[0].getParent());
    }

    @Test
    void privateFileVerificationFailureClosesAndDeletesTheStagingFile() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        Path staging = privateDirectory.resolve(".verification-failure.part");

        IOException failure = assertThrows(IOException.class,
                () -> NativeLibraryLoader.openNewPrivateFile(staging, privateDirectory,
                        (file, directory) -> {
                            assertEquals(staging, file);
                            assertEquals(privateDirectory, directory);
                            throw new IOException("synthetic private-file verification failure");
                        }));

        assertEquals("synthetic private-file verification failure", failure.getMessage());
        assertFalse(Files.exists(staging));
        assertDirectoryEmpty(privateDirectory);
    }

    @Test
    void unsupportedAtomicPublicationFailsClosedAndCleansStagingFiles() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] library = {0x4b, 0x4d, 0x49, 0x50};

        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        new ByteArrayInputStream(library), sha256(library), privateDirectory, WINDOWS_LIBRARY,
                        (source, destination) -> {
                            throw new AtomicMoveNotSupportedException(
                                    source.toString(), destination.toString(), "synthetic unsupported move");
                        }));

        assertRedacted(failure);
        assertDirectoryEmpty(privateDirectory);
    }

    @Test
    void rejectsNativeLibraryWhoseIdentityChangesDuringPublication() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] library = {0x4b, 0x4d, 0x49, 0x50};
        byte[] replacement = {1, 3, 3, 7};

        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        new ByteArrayInputStream(library), sha256(library), privateDirectory, WINDOWS_LIBRARY,
                        (staging, destination) -> {
                            Files.move(staging, destination, java.nio.file.StandardCopyOption.ATOMIC_MOVE);
                            Files.delete(destination);
                            Files.write(destination, replacement);
                        }));

        assertRedacted(failure);
        assertArrayEquals(replacement, Files.readAllBytes(privateDirectory.resolve(WINDOWS_LIBRARY)));
        assertDirectoryEmptyExcept(privateDirectory, WINDOWS_LIBRARY);
    }

    @Test
    void rejectsNativeLibraryModifiedAfterPublication() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] library = {0x4b, 0x4d, 0x49, 0x50};
        byte[] modified = {1, 3, 3, 7};

        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        new ByteArrayInputStream(library), sha256(library), privateDirectory, WINDOWS_LIBRARY,
                        (staging, destination) -> {
                            Files.move(staging, destination, java.nio.file.StandardCopyOption.ATOMIC_MOVE);
                            Files.write(destination, modified);
                        }));

        assertRedacted(failure);
        assertArrayEquals(modified, Files.readAllBytes(privateDirectory.resolve(WINDOWS_LIBRARY)));
        assertDirectoryEmptyExcept(privateDirectory, WINDOWS_LIBRARY);
    }

    @Test
    void failedIdentityLinkCleanupDoesNotLeakItsPathOrDetails() throws Exception {
        Path privateDirectory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] library = {0x4b, 0x4d, 0x49, 0x50};
        Path[] retainedIdentityDirectory = new Path[1];

        UnsatisfiedLinkError failure = assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(
                        new ByteArrayInputStream(library), sha256(library), privateDirectory, WINDOWS_LIBRARY,
                        (staging, destination) -> {
                            Files.move(staging, destination, java.nio.file.StandardCopyOption.ATOMIC_MOVE);
                            try (var entries = Files.list(privateDirectory)) {
                                Path identityLink = entries
                                        .filter(path -> path.getFileName().toString().endsWith(".identity"))
                                        .findFirst().orElseThrow();
                                Files.delete(identityLink);
                                Files.createDirectory(identityLink);
                                Files.write(identityLink.resolve("retained"), new byte[] {9});
                                retainedIdentityDirectory[0] = identityLink;
                            }
                        }));

        assertRedacted(failure);
        assertTrue(Files.isRegularFile(privateDirectory.resolve(WINDOWS_LIBRARY)));
        assertTrue(Files.isRegularFile(retainedIdentityDirectory[0].resolve("retained")));
        Files.delete(retainedIdentityDirectory[0].resolve("retained"));
        Files.delete(retainedIdentityDirectory[0]);
        assertDirectoryEmptyExcept(privateDirectory, WINDOWS_LIBRARY);
    }

    @Test
    void missingHashAndLibraryNameFailBeforeReadingNativeBytes() throws Exception {
        Path directory = NativeLibraryLoader.createPrivateDirectory(temporaryDirectory);
        byte[] library = {1, 2};
        InputStream unreadableSource = new InputStream() {
            @Override
            public int read() {
                throw new AssertionError("Invalid metadata must be rejected before reading native bytes");
            }
        };
        for (String hash : new String[] {null, "", "f".repeat(63), "0".repeat(63) + "g"}) {
            assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                    () -> NativeLibraryLoader.extractVerified(unreadableSource,
                            hash, directory, WINDOWS_LIBRARY)));
        }
        assertRedacted(assertThrows(UnsatisfiedLinkError.class,
                () -> NativeLibraryLoader.extractVerified(unreadableSource,
                        sha256(library), directory, null)));
        assertDirectoryEmpty(directory);
    }

    @FunctionalInterface
    private interface CloseAction {
        void run() throws IOException;
    }

    private static InputStream sourceWithCloseAction(byte[] bytes, CloseAction action) {
        return new ByteArrayInputStream(bytes) {
            @Override
            public void close() throws IOException {
                super.close();
                action.run();
            }
        };
    }

    private static Path stagingFile(Path directory) throws IOException {
        try (var entries = Files.list(directory)) {
            List<Path> staging = entries.filter(path -> path.getFileName().toString().endsWith(".part"))
                    .toList();
            assertEquals(1, staging.size());
            return staging.get(0);
        }
    }

    private static void createSymbolicLinkOrSkip(Path link, Path target) throws IOException {
        try {
            Files.createSymbolicLink(link, target);
        } catch (IOException | UnsupportedOperationException unavailable) {
            Assumptions.assumeTrue(false, "Symbolic links are unavailable: " + unavailable.getClass().getName());
        }
    }

    private static void assertRedacted(UnsatisfiedLinkError failure) {
        assertEquals(NativeLibraryLoader.LOAD_FAILURE_MESSAGE, failure.getMessage());
        assertNull(failure.getCause());
        assertEquals(0, failure.getSuppressed().length);
    }

    private static void assertDirectoryEmpty(Path directory) throws IOException {
        try (var files = Files.list(directory)) {
            assertEquals(0, files.count());
        }
    }

    private static void assertDirectoryEmptyExcept(Path directory, String name) throws IOException {
        try (var files = Files.list(directory)) {
            assertEquals(List.of(name), files.map(path -> path.getFileName().toString()).toList());
        }
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

    private static Path classLocation(Class<?> type) throws Exception {
        return Path.of(type.getProtectionDomain().getCodeSource().getLocation().toURI());
    }

    private Path bundledResourcesForChild() throws Exception {
        Path nativeLibrary = Path.of(System.getProperty("kmipkit.nativeLibrary", ""));
        assertTrue(Files.isRegularFile(nativeLibrary), "Maven must build the JNI library before this test");
        byte[] nativeBytes = Files.readAllBytes(nativeLibrary);
        String targetId = NativeLibraryLoader.targetIdFor(
                System.getProperty("os.name", ""), System.getProperty("os.arch", ""));
        Path resourceRoot = Files.createDirectory(temporaryDirectory.resolve("version-probe-classpath"));
        Path resourceLibrary = resourceRoot.resolve("META-INF/native")
                .resolve(targetId).resolve(nativeLibrary.getFileName());
        Files.createDirectories(resourceLibrary.getParent());
        Files.write(resourceLibrary, nativeBytes);
        Files.writeString(resourceLibrary.resolveSibling(resourceLibrary.getFileName() + ".sha256"),
                sha256(nativeBytes) + "\n", StandardCharsets.US_ASCII);
        return resourceRoot;
    }

    private Path copyMainClassesWithoutVersionRecord(Path destination) throws Exception {
        Path mainClasses = classLocation(NativeExtensionRegistry.class);
        Path omittedClass = Path.of("org", "kmipkit", "RuntimeMetadata$Version.class");
        try (var paths = Files.walk(mainClasses)) {
            for (Path source : paths.toList()) {
                Path relative = mainClasses.relativize(source);
                if (relative.equals(omittedClass)) {
                    continue;
                }
                Path target = destination.resolve(relative);
                if (Files.isDirectory(source)) {
                    Files.createDirectories(target);
                } else {
                    Files.createDirectories(target.getParent());
                    Files.copy(source, target);
                }
            }
        }
        assertFalse(Files.exists(destination.resolve(omittedClass)));
        return destination;
    }

    private Path compileVersionClassWithoutNativeConstructor(Path outputDirectory) throws Exception {
        Path source = temporaryDirectory.resolve("version-class-source/org/kmipkit/RuntimeMetadata$Version.java");
        Files.createDirectories(source.getParent());
        Files.createDirectories(outputDirectory);
        Files.writeString(source, """
                package org.kmipkit;
                public final class RuntimeMetadata$Version {
                    public RuntimeMetadata$Version() {}
                }
                """, StandardCharsets.UTF_8);

        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        assertNotNull(compiler, "A JDK compiler is required to compile the compatibility fixture");
        int result = compiler.run(null, null, null,
                "--release", "17", "-d", outputDirectory.toString(), source.toString());
        assertEquals(0, result, "The incompatible Version class fixture must compile");
        assertTrue(Files.isRegularFile(outputDirectory.resolve("org/kmipkit/RuntimeMetadata$Version.class")));
        return outputDirectory;
    }

    private ProcessResult runVersionProbe(
            Path resourceRoot, Path childTemporaryRoot, Path classOverlay, Path childMainClasses) throws Exception {
        Path testClasses = classLocation(NativeLibraryLoadProbe.class);
        Path java = Path.of(System.getProperty("java.home"), "bin",
                System.getProperty("os.name", "").startsWith("Windows") ? "java.exe" : "java");
        List<String> classPathEntries = new java.util.ArrayList<>();
        classPathEntries.add(resourceRoot.toString());
        if (classOverlay != null) {
            classPathEntries.add(classOverlay.toString());
        }
        classPathEntries.add(testClasses.toString());
        classPathEntries.add(childMainClasses.toString());
        String classPath = String.join(File.pathSeparator, classPathEntries);
        List<String> command = new java.util.ArrayList<>(List.of(
                java.toString(), "-Djava.io.tmpdir=" + childTemporaryRoot,
                "--class-path", classPath, NativeLibraryLoadProbe.class.getName(), "version"));
        String jacocoAgent = childJacocoAgentArgument(ManagementFactory.getRuntimeMXBean().getInputArguments());
        if (jacocoAgent != null) {
            command.add(1, jacocoAgent);
        }
        Process process = new ProcessBuilder(command)
                .redirectErrorStream(true)
                .start();
        // The child inherits LLVM_PROFILE_FILE so JNI execution is recorded in its own process profile.
        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        return new ProcessResult(process.waitFor(), output);
    }

    private record ProcessResult(int exitCode, String output) {}

    private static String wrongAbiLibraryName() {
        String osName = System.getProperty("os.name", "");
        if (osName.startsWith("Windows")) {
            return "kmipkit_wrong_abi.dll";
        }
        if (osName.startsWith("Mac")) {
            return "libkmipkit_wrong_abi.dylib";
        }
        return "libkmipkit_wrong_abi.so";
    }

    private static String childJacocoAgentArgument(List<String> inputArguments) {
        for (String argument : inputArguments) {
            if (!argument.startsWith("-javaagent:")) {
                continue;
            }
            int optionsStart = argument.indexOf('=');
            String agentPath = optionsStart < 0
                    ? argument.substring("-javaagent:".length())
                    : argument.substring("-javaagent:".length(), optionsStart);
            if (!agentPath.toLowerCase(java.util.Locale.ROOT).contains("jacoco")) {
                continue;
            }

            List<String> options = new java.util.ArrayList<>();
            if (optionsStart >= 0) {
                for (String option : argument.substring(optionsStart + 1).split(",")) {
                    if (!option.startsWith("append=")) {
                        options.add(option);
                    }
                }
            }
            options.add("append=true");
            return "-javaagent:" + agentPath + "=" + String.join(",", options);
        }
        return null;
    }

    private static void restoreProperty(String name, String value) {
        if (value == null) {
            System.clearProperty(name);
        } else {
            System.setProperty(name, value);
        }
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
