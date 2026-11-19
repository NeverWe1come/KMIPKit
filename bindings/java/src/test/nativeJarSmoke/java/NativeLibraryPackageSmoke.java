import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.Permission;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.Enumeration;
import java.util.HexFormat;
import java.util.List;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.jar.JarOutputStream;

import org.kmipkit.RuntimeMetadata;

/** Offline consumer probes for the actual Java native classifier JAR. */
public final class NativeLibraryPackageSmoke {
    private static final String NATIVE_RESOURCE_PREFIX = "META-INF/native/";
    private static final String FAILURE_MESSAGE =
            "KMIPKit native library could not be loaded; install a matching package for this OS and architecture "
                    + "or set -Dkmipkit.nativeLibrary to a compatible library.";

    private static volatile boolean networkAttempted;

    private NativeLibraryPackageSmoke() {}

    public static void main(String[] args) throws Exception {
        if (args.length == 2 && args[0].equals("--probe")) {
            System.exit(runProbe(args[1]));
        }
        if (args.length != 2) {
            throw new IllegalArgumentException("The package smoke requires its JAR and wrong-ABI fixture paths.");
        }

        Path packagedJar = Path.of(args[0]).toAbsolutePath();
        Path wrongAbiLibrary = Path.of(args[1]).toAbsolutePath();
        verifyPublicPkcs11ApiPresent(packagedJar);
        Path classes = Path.of(NativeLibraryPackageSmoke.class.getProtectionDomain()
                .getCodeSource().getLocation().toURI());
        Path temporaryDirectory = Files.createTempDirectory("kmipkit-java-native-package-smoke-");
        try {
            verifyLoadSuccess(packagedJar, classes);
            Path missingBundle = createVariant(packagedJar, temporaryDirectory.resolve("missing.jar"), null, true);
            verifyMissingBundle(missingBundle, classes);
            verifyUnsupportedArchitecture(packagedJar, classes);
            Path wrongAbiBundle = createVariant(
                    packagedJar, temporaryDirectory.resolve("wrong-abi.jar"), wrongAbiLibrary, false);
            verifyWrongAbi(wrongAbiBundle, classes);
            System.out.println("packaged JNI load and negative probes passed offline");
        } finally {
            deleteTree(temporaryDirectory);
        }
    }

    private static void verifyLoadSuccess(Path packagedJar, Path classes) throws Exception {
        ProcessResult result = runProbe(packagedJar, classes, "success", null);
        checkNetworkWasBlocked(result);
        if (result.exitCode() != 0 || !result.output().contains("PACKAGE_NATIVE_LOAD_OK ABI=1")) {
            throw new IllegalStateException("The packaged JNI library did not load successfully offline.");
        }
    }

    private static void verifyMissingBundle(Path missingBundle, Path classes) throws Exception {
        ProcessResult result = runProbe(missingBundle, classes, "failure", null);
        checkNetworkWasBlocked(result);
        requireRedactedActionableFailure(result, "package", "-Dkmipkit.nativeLibrary");
    }

    private static void verifyUnsupportedArchitecture(Path packagedJar, Path classes) throws Exception {
        ProcessResult result = runProbe(packagedJar, classes, "failure", "kmipkit-t028-unsupported");
        checkNetworkWasBlocked(result);
        requireRedactedActionableFailure(result, "architecture", "-Dkmipkit.nativeLibrary");
    }

    private static void verifyWrongAbi(Path wrongAbiBundle, Path classes) throws Exception {
        ProcessResult result = runProbe(wrongAbiBundle, classes, "failure", null);
        checkNetworkWasBlocked(result);
        requireRedactedActionableFailure(result, "ABI", "-Dkmipkit.nativeLibrary");
    }

    private static void requireRedactedActionableFailure(ProcessResult result, String action, String overrideHint) {
        if (result.exitCode() != 0
                || !result.output().contains("PACKAGE_NATIVE_LOAD_ERROR ")
                || !result.output().contains(action)
                || !result.output().contains(overrideHint)
                || result.output().contains("kmipkit-java-native-package-smoke-")) {
            throw new IllegalStateException("A packaged native load failure was not redacted and actionable.");
        }
    }

    private static ProcessResult runProbe(
            Path packagedJar, Path classes, String mode, String architectureOverride) throws Exception {
        Path javaExecutable = Path.of(System.getProperty("java.home"), "bin",
                System.getProperty("os.name", "").startsWith("Windows") ? "java.exe" : "java");
        String classPath = String.join(java.io.File.pathSeparator, packagedJar.toString(), classes.toString());
        List<String> command = new ArrayList<>(List.of(
                javaExecutable.toString(),
                "-Djava.security.manager=allow",
                "--class-path", classPath,
                NativeLibraryPackageSmoke.class.getName(), "--probe", mode));
        if (architectureOverride != null) {
            command.add(1, "-Dos.arch=" + architectureOverride);
        }
        Process process = new ProcessBuilder(command).redirectErrorStream(true).start();
        String output = new String(process.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
        return new ProcessResult(process.waitFor(), output);
    }

    private static int runProbe(String mode) {
        installOfflineNetworkGuard();
        if (System.getProperty("kmipkit.nativeLibrary") != null) {
            System.out.println("PACKAGE_SMOKE_OVERRIDE_PRESENT");
            return 8;
        }
        try {
            RuntimeMetadata.Version version = RuntimeMetadata.version();
            if (!mode.equals("success")) {
                System.out.println("PACKAGE_NATIVE_LOAD_UNEXPECTEDLY_SUCCEEDED");
                return 9;
            }
            System.out.println("PACKAGE_NATIVE_LOAD_OK ABI=" + version.abiVersion());
        } catch (Throwable failure) {
            if (mode.equals("success")) {
                System.out.println("PACKAGE_NATIVE_LOAD_UNEXPECTED_FAILURE");
                return 10;
            }
            System.out.println("PACKAGE_NATIVE_LOAD_ERROR " + nativeLoadMessage(failure));
        }
        System.out.println("OFFLINE_NETWORK_ATTEMPTS=" + (networkAttempted ? 1 : 0));
        return 0;
    }

    private static void verifyPublicPkcs11ApiPresent(Path packagedJar) throws IOException {
        List<String> publicClassEntries = List.of(
                "org/kmipkit/NativeExtensionRegistry.class",
                "org/kmipkit/extensions/ClientBatchItem.class",
                "org/kmipkit/extensions/ClientBatchItemResult.class",
                "org/kmipkit/extensions/ClientBatchItemResult$Pkcs11Response.class");
        try (JarFile jar = new JarFile(packagedJar.toFile())) {
            for (String publicClass : publicClassEntries) {
                var entry = jar.getJarEntry(publicClass);
                if (entry == null) {
                    throw new IllegalStateException("The Java native JAR is missing a public PKCS#11 API class.");
                }
                String constantPool = new String(
                        jar.getInputStream(entry).readAllBytes(), StandardCharsets.ISO_8859_1);
                if (publicClass.contains("Pkcs11Response")
                        && (!constantPool.contains("responseTtlv") || !constantPool.contains("close"))) {
                    throw new IllegalStateException("The Java JAR is missing PKCS#11 response accessors.");
                }
            }
        }
    }

    private static String nativeLoadMessage(Throwable failure) {
        for (Throwable current = failure; current != null; current = current.getCause()) {
            if (current instanceof UnsatisfiedLinkError && current.getMessage() != null) {
                return current.getMessage();
            }
        }
        return FAILURE_MESSAGE;
    }

    @SuppressWarnings("removal")
    private static void installOfflineNetworkGuard() {
        System.setSecurityManager(new SecurityManager() {
            @Override
            public void checkPermission(Permission permission) {}

            @Override
            public void checkConnect(String host, int port) {
                blockNetwork();
            }

            @Override
            public void checkConnect(String host, int port, Object context) {
                blockNetwork();
            }

            @Override
            public void checkListen(int port) {
                blockNetwork();
            }

            @Override
            public void checkAccept(String host, int port) {
                blockNetwork();
            }
        });
    }

    private static void blockNetwork() {
        networkAttempted = true;
        throw new SecurityException("Native package smoke runs without network access.");
    }

    private static void checkNetworkWasBlocked(ProcessResult result) {
        if (!result.output().contains("OFFLINE_NETWORK_ATTEMPTS=0")) {
            throw new IllegalStateException("The packaged native loader attempted network access.");
        }
    }

    private static Path createVariant(
            Path source, Path destination, Path replacementLibrary, boolean omitNativeResources) throws Exception {
        int libraryEntries = 0;
        int omittedNativeLibraryEntries = 0;
        try (JarFile input = new JarFile(source.toFile());
                JarOutputStream output = new JarOutputStream(Files.newOutputStream(destination))) {
            Enumeration<JarEntry> entries = input.entries();
            while (entries.hasMoreElements()) {
                JarEntry entry = entries.nextElement();
                String name = entry.getName();
                if (omitNativeResources && name.startsWith(NATIVE_RESOURCE_PREFIX)) {
                    if (isNativeLibrary(name)) {
                        omittedNativeLibraryEntries++;
                    }
                    continue;
                }

                byte[] contents;
                if (replacementLibrary != null && isNativeLibrary(name)) {
                    libraryEntries++;
                    contents = Files.readAllBytes(replacementLibrary);
                } else if (replacementLibrary != null && name.endsWith(".sha256")) {
                    contents = (sha256(Files.readAllBytes(replacementLibrary)) + "\n")
                            .getBytes(StandardCharsets.US_ASCII);
                } else {
                    try (InputStream content = input.getInputStream(entry)) {
                        contents = content.readAllBytes();
                    }
                }
                output.putNextEntry(new JarEntry(name));
                output.write(contents);
                output.closeEntry();
            }
        }

        if (omitNativeResources && omittedNativeLibraryEntries != 1) {
            throw new IllegalStateException("The missing-library package variant did not omit its native library.");
        }
        if (replacementLibrary != null && libraryEntries != 1) {
            throw new IllegalStateException("The package must contain exactly one host-native resource.");
        }
        return destination;
    }

    private static boolean isNativeLibrary(String name) {
        return name.startsWith(NATIVE_RESOURCE_PREFIX)
                && (name.endsWith(".dll") || name.endsWith(".dylib") || name.endsWith(".so"));
    }

    private static String sha256(byte[] contents) throws Exception {
        return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(contents));
    }

    private static void deleteTree(Path root) throws IOException {
        try (var paths = Files.walk(root)) {
            paths.sorted(Comparator.reverseOrder()).forEach(path -> {
                try {
                    Files.deleteIfExists(path);
                } catch (IOException failure) {
                    throw new UncheckedIOException(failure);
                }
            });
        } catch (UncheckedIOException failure) {
            throw failure.getCause();
        }
    }

    private record ProcessResult(int exitCode, String output) {}
}
