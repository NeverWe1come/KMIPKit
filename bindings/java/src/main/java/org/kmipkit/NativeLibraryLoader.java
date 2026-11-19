package org.kmipkit;

import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.channels.SeekableByteChannel;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.OpenOption;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.nio.file.attribute.AclEntry;
import java.nio.file.attribute.AclEntryPermission;
import java.nio.file.attribute.AclEntryType;
import java.nio.file.attribute.AclFileAttributeView;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileAttribute;
import java.nio.file.attribute.PosixFilePermission;
import java.nio.file.attribute.PosixFilePermissions;
import java.nio.file.attribute.UserPrincipal;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.EnumSet;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.UUID;
import org.jspecify.annotations.Nullable;

/** Secure selection and loading of the bundled JNI library. */
final class NativeLibraryLoader {
    static final String LOAD_FAILURE_MESSAGE = "KMIPKit native library could not be loaded; install a matching "
            + "package for this OS and architecture or set -Dkmipkit.nativeLibrary to a compatible library.";
    static final String ABI_FAILURE_MESSAGE = "KMIPKit native library has an incompatible ABI; install a matching "
            + "package or set -Dkmipkit.nativeLibrary to a compatible library.";

    private static final String LIBRARY_BASE_NAME = "kmipkit_jni";
    private static final int SHA256_HEX_LENGTH = 64;
    private static final int COPY_BUFFER_SIZE = 32 * 1024;
    private static final Set<PosixFilePermission> PRIVATE_DIRECTORY_PERMISSIONS = Set.of(
            PosixFilePermission.OWNER_READ,
            PosixFilePermission.OWNER_WRITE,
            PosixFilePermission.OWNER_EXECUTE);
    private static final Set<PosixFilePermission> PRIVATE_FILE_PERMISSIONS = Set.of(
            PosixFilePermission.OWNER_READ,
            PosixFilePermission.OWNER_WRITE);

    @FunctionalInterface
    interface PrivateDirectoryFactory {
        Path create(Path temporaryRoot) throws IOException;
    }

    @FunctionalInterface
    interface PrivateDirectoryVerifier {
        void verify(Path directory) throws IOException;
    }

    @FunctionalInterface
    interface AtomicPublisher {
        void publish(Path staging, Path destination) throws IOException;
    }

    @FunctionalInterface
    interface PrivateFileVerifier {
        void verify(Path file, Path privateDirectory) throws IOException;
    }

    private NativeLibraryLoader() {}

    static void load() {
        try {
            Path explicitLibrary = resolveExplicitLibrary(
                    System.getProperty("kmipkit.nativeLibrary"));
            if (explicitLibrary != null) {
                loadAndVerify(explicitLibrary);
                return;
            }

            Path library = extractBundled(
                    NativeLibraryLoader.class.getClassLoader(),
                    System.getProperty("os.name", ""),
                    System.getProperty("os.arch", ""),
                    Path.of(System.getProperty("java.io.tmpdir", "")),
                    NativeLibraryLoader::createPrivateDirectory);
            loadAndVerify(library);
        } catch (UnsatisfiedLinkError failure) {
            if (failure instanceof NativeAbiMismatchError) {
                throw failure;
            }
            throw loadFailure();
        } catch (RuntimeException failure) {
            throw loadFailure();
        }
    }

    static @Nullable Path resolveExplicitLibrary(@Nullable String configuredPath) {
        if (configuredPath == null) {
            return null;
        }
        if (configuredPath.isBlank()) {
            throw loadFailure();
        }
        try {
            Path explicitLibrary = Path.of(configuredPath);
            if (!explicitLibrary.isAbsolute()) {
                throw loadFailure();
            }
            return explicitLibrary;
        } catch (RuntimeException failure) {
            throw loadFailure();
        }
    }

    static void validateNativeAbiVersion(int abiVersion) {
        if (abiVersion != 1) {
            throw new NativeAbiMismatchError();
        }
    }

    private static void loadAndVerify(Path library) {
        System.load(library.toAbsolutePath().normalize().toString());
        validateNativeAbiVersion(NativeExtensionRegistry.nativeAbiVersion());
    }

    static String targetIdFor(String osName, String osArch) {
        String os = osName.toLowerCase(Locale.ROOT);
        String arch = osArch.toLowerCase(Locale.ROOT);
        if (os.startsWith("windows") && isX86_64(arch)) {
            return "windows-x86_64-msvc";
        }
        if (os.equals("linux")) {
            if (isX86_64(arch)) {
                return "linux-glibc-x86_64";
            }
            if (isAarch64(arch)) {
                return "linux-glibc-aarch64";
            }
        }
        if ((os.startsWith("mac") || os.equals("darwin")) && isX86_64(arch)) {
            return "macos-x86_64";
        }
        if ((os.startsWith("mac") || os.equals("darwin")) && isAarch64(arch)) {
            return "macos-aarch64";
        }
        throw loadFailure();
    }

    static Path extractBundled(
            ClassLoader resources,
            String osName,
            String osArch,
            Path temporaryRoot,
            PrivateDirectoryFactory directoryFactory) {
        Path privateDirectory = null;
        String libraryName = null;
        try {
            String targetId = targetIdFor(osName, osArch);
            libraryName = libraryFileName(targetId);
            String resourceBase = "META-INF/native/" + targetId + "/" + libraryName;
            try (InputStream library = resources.getResourceAsStream(resourceBase);
                    InputStream digest = resources.getResourceAsStream(resourceBase + ".sha256")) {
                if (library == null || digest == null) {
                    throw loadFailure();
                }
                String expectedHash = readExpectedHash(digest);
                privateDirectory = directoryFactory.create(temporaryRoot);
                verifyPrivateDirectory(privateDirectory);
                return extractVerified(library, expectedHash, privateDirectory, libraryName);
            }
        } catch (UnsatisfiedLinkError failure) {
            cleanupFailedExtraction(privateDirectory, libraryName);
            throw loadFailure();
        } catch (IOException | RuntimeException failure) {
            cleanupFailedExtraction(privateDirectory, libraryName);
            throw loadFailure();
        }
    }

    static Path createPrivateDirectory(Path temporaryRoot) throws IOException {
        return createPrivateDirectory(temporaryRoot, NativeLibraryLoader::verifyPrivateDirectory);
    }

    static Path createPrivateDirectory(Path temporaryRoot, PrivateDirectoryVerifier directoryVerifier)
            throws IOException {
        Path root = temporaryRoot.toAbsolutePath().normalize();
        if (Files.isSymbolicLink(root) || !Files.isDirectory(root, LinkOption.NOFOLLOW_LINKS)) {
            throw new IOException("Temporary directory is unavailable");
        }

        if (Files.getFileStore(root).supportsFileAttributeView("posix")) {
            FileAttribute<Set<PosixFilePermission>> permissions = PosixFilePermissions
                    .asFileAttribute(PRIVATE_DIRECTORY_PERMISSIONS);
            Path directory = Files.createTempDirectory(root, "kmipkit-native-", permissions);
            return verifyPrivateDirectoryAndCleanupOnFailure(directory, directoryVerifier);
        }

        if (Files.getFileStore(root).supportsFileAttributeView("acl")) {
            UserPrincipal owner = Files.getOwner(root, LinkOption.NOFOLLOW_LINKS);
            FileAttribute<List<AclEntry>> acl = privateAcl(owner);
            Path directory = Files.createTempDirectory(root, "kmipkit-native-", acl);
            return verifyPrivateDirectoryAndCleanupOnFailure(directory, directoryVerifier);
        }
        throw new IOException("Private permissions are unavailable");
    }

    private static Path verifyPrivateDirectoryAndCleanupOnFailure(
            Path directory,
            PrivateDirectoryVerifier directoryVerifier) throws IOException {
        try {
            directoryVerifier.verify(directory);
            return directory;
        } catch (IOException | RuntimeException failure) {
            try {
                Files.deleteIfExists(directory);
            } catch (IOException | RuntimeException cleanupFailure) {
                failure.addSuppressed(cleanupFailure);
            }
            throw failure;
        }
    }

    static Path extractVerified(InputStream library, String expectedHash, Path privateDirectory, String libraryName) {
        return extractVerified(library, expectedHash, privateDirectory, libraryName,
                (staging, destination) -> Files.move(staging, destination, StandardCopyOption.ATOMIC_MOVE));
    }

    static Path extractVerified(
            InputStream library,
            String expectedHash,
            Path privateDirectory,
            String libraryName,
            AtomicPublisher atomicPublisher) {
        Path staging = null;
        Path identityLink = null;
        try {
            if (!isValidHash(expectedHash) || !isLeafName(libraryName)) {
                throw new IOException("Invalid bundled library metadata");
            }
            verifyPrivateDirectory(privateDirectory);

            Path destination = privateDirectory.toAbsolutePath().normalize().resolve(libraryName);
            if (Files.exists(destination, LinkOption.NOFOLLOW_LINKS)) {
                throw new IOException("Native library destination already exists");
            }
            staging = privateDirectory.toAbsolutePath().normalize().resolve(
                    ".kmipkit-native-" + UUID.randomUUID() + ".part");

            MessageDigest digest = sha256Digest();
            byte[] copyBuffer = new byte[COPY_BUFFER_SIZE];
            try (SeekableByteChannel channel = openNewPrivateFile(staging, privateDirectory);
                    InputStream source = library) {
                ByteBuffer buffer = ByteBuffer.wrap(copyBuffer);
                int read;
                while ((read = source.read(copyBuffer)) != -1) {
                    digest.update(copyBuffer, 0, read);
                    buffer.clear();
                    buffer.limit(read);
                    while (buffer.hasRemaining()) {
                        channel.write(buffer);
                    }
                }
                if (channel instanceof java.nio.channels.FileChannel fileChannel) {
                    fileChannel.force(true);
                } else {
                    throw new IOException("Native library file cannot be synchronized");
                }
            }

            byte[] copiedDigest = digest.digest();
            if (!MessageDigest.isEqual(copiedDigest, HexFormat.of().parseHex(expectedHash))) {
                throw new IOException("Native library digest mismatch");
            }
            BasicFileAttributes stagedAttributes = readRegularFileAttributes(staging);
            verifyPrivateFile(staging, privateDirectory);
            identityLink = privateDirectory.toAbsolutePath().normalize().resolve(
                    ".kmipkit-native-" + UUID.randomUUID() + ".identity");
            Files.createLink(identityLink, staging);
            verifyPrivateFile(identityLink, privateDirectory);

            if (Files.exists(destination, LinkOption.NOFOLLOW_LINKS)) {
                throw new IOException("Native library destination already exists");
            }
            try {
                atomicPublisher.publish(staging, destination);
            } catch (AtomicMoveNotSupportedException unsupported) {
                throw new IOException("Atomic native library publication is unavailable");
            }
            staging = null;

            BasicFileAttributes finalAttributes = readRegularFileAttributes(destination);
            boolean sameIdentity = stagedAttributes.fileKey() != null
                    && stagedAttributes.fileKey().equals(finalAttributes.fileKey());
            if (!sameIdentity && !Files.isSameFile(identityLink, destination)) {
                throw new IOException("Native library identity changed");
            }
            verifyPrivateFile(destination, privateDirectory);
            if (!MessageDigest.isEqual(copiedDigest, digestFile(destination))) {
                throw new IOException("Native library changed after publication");
            }
            Files.delete(identityLink);
            identityLink = null;
            return destination.toAbsolutePath().normalize();
        } catch (IOException | RuntimeException failure) {
            throw loadFailure();
        } finally {
            if (staging != null) {
                try {
                    Files.deleteIfExists(staging);
                } catch (IOException ignored) {
                    // The private temporary directory is inaccessible to other users.
                }
            }
            if (identityLink != null) {
                try {
                    Files.deleteIfExists(identityLink);
                } catch (IOException ignored) {
                    // The private temporary directory is inaccessible to other users.
                }
            }
        }
    }

    private static Set<OpenOption> newFileOptions() {
        return Set.of(StandardOpenOption.CREATE_NEW, StandardOpenOption.WRITE, LinkOption.NOFOLLOW_LINKS);
    }

    private static SeekableByteChannel openNewPrivateFile(Path path, Path privateDirectory) throws IOException {
        return openNewPrivateFile(path, privateDirectory, NativeLibraryLoader::verifyPrivateFile);
    }

    static SeekableByteChannel openNewPrivateFile(
            Path path, Path privateDirectory, PrivateFileVerifier fileVerifier) throws IOException {
        FileAttribute<?> attribute = privateFileAttribute(privateDirectory);
        SeekableByteChannel channel = Files.newByteChannel(path, newFileOptions(), attribute);
        try {
            fileVerifier.verify(path, privateDirectory);
            return channel;
        } catch (IOException failure) {
            channel.close();
            Files.deleteIfExists(path);
            throw failure;
        }
    }

    static FileAttribute<?> privateFileAttribute(Path privateDirectory) throws IOException {
        if (Files.getFileStore(privateDirectory).supportsFileAttributeView("posix")) {
            return PosixFilePermissions.asFileAttribute(PRIVATE_FILE_PERMISSIONS);
        }
        if (Files.getFileStore(privateDirectory).supportsFileAttributeView("acl")) {
            UserPrincipal owner = Files.getOwner(privateDirectory, LinkOption.NOFOLLOW_LINKS);
            return privateAcl(owner);
        }
        throw new IOException("Private permissions are unavailable");
    }

    private static void verifyPrivateDirectory(Path directory) throws IOException {
        Path normalized = directory.toAbsolutePath().normalize();
        if (Files.isSymbolicLink(normalized)) {
            throw new IOException("Private directory is a symbolic link");
        }
        BasicFileAttributes attributes = Files.readAttributes(
                normalized, BasicFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
        if (!attributes.isDirectory()) {
            throw new IOException("Private directory is not a directory");
        }
        verifyPrivatePermissions(normalized, false);
    }

    private static void verifyPrivateFile(Path file, Path privateDirectory) throws IOException {
        BasicFileAttributes attributes = readRegularFileAttributes(file);
        verifyPrivatePermissions(file, true);
        UserPrincipal expectedOwner = Files.getOwner(privateDirectory, LinkOption.NOFOLLOW_LINKS);
        if (!expectedOwner.equals(Files.getOwner(file, LinkOption.NOFOLLOW_LINKS))) {
            throw new IOException("Native library owner is not private");
        }
    }

    private static BasicFileAttributes readRegularFileAttributes(Path file) throws IOException {
        if (Files.isSymbolicLink(file)) {
            throw new IOException("Native library is a symbolic link");
        }
        BasicFileAttributes attributes = Files.readAttributes(
                file, BasicFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
        if (!attributes.isRegularFile()) {
            throw new IOException("Native library is not a regular file");
        }
        return attributes;
    }

    private static void verifyPrivatePermissions(Path path, boolean file) throws IOException {
        if (Files.getFileStore(path).supportsFileAttributeView("posix")) {
            Set<PosixFilePermission> expected = file
                    ? PRIVATE_FILE_PERMISSIONS : PRIVATE_DIRECTORY_PERMISSIONS;
            if (!Files.getPosixFilePermissions(path, LinkOption.NOFOLLOW_LINKS).equals(expected)) {
                throw new IOException("Native library permissions are not private");
            }
            return;
        }

        AclFileAttributeView aclView = Files.getFileAttributeView(
                path, AclFileAttributeView.class, LinkOption.NOFOLLOW_LINKS);
        if (aclView == null) {
            throw new IOException("Private ACL is unavailable");
        }
        UserPrincipal owner = aclView.getOwner();
        List<AclEntry> entries = aclView.getAcl();
        if (entries.size() != 1 || !isPrivateOwnerEntry(entries.get(0), owner)) {
            throw new IOException("Native library ACL is not private");
        }
    }

    private static boolean isPrivateOwnerEntry(AclEntry entry, UserPrincipal owner) {
        return entry.type() == AclEntryType.ALLOW
                && entry.principal().equals(owner)
                && entry.permissions().equals(EnumSet.allOf(AclEntryPermission.class))
                && entry.flags().isEmpty();
    }

    private static FileAttribute<List<AclEntry>> privateAcl(UserPrincipal owner) {
        AclEntry entry = AclEntry.newBuilder()
                .setType(AclEntryType.ALLOW)
                .setPrincipal(owner)
                .setPermissions(EnumSet.allOf(AclEntryPermission.class))
                .build();
        return new FileAttribute<>() {
            @Override
            public String name() {
                return "acl:acl";
            }

            @Override
            public List<AclEntry> value() {
                return List.of(entry);
            }
        };
    }

    private static String readExpectedHash(InputStream input) throws IOException {
        byte[] bytes = input.readNBytes(SHA256_HEX_LENGTH + 3);
        if (bytes.length < SHA256_HEX_LENGTH || bytes.length > SHA256_HEX_LENGTH + 2) {
            throw new IOException("Invalid native library digest resource");
        }
        if (bytes.length == SHA256_HEX_LENGTH + 1 && bytes[SHA256_HEX_LENGTH] != '\n') {
            throw new IOException("Invalid native library digest resource");
        }
        if (bytes.length == SHA256_HEX_LENGTH + 2
                && (bytes[SHA256_HEX_LENGTH] != '\r' || bytes[SHA256_HEX_LENGTH + 1] != '\n')) {
            throw new IOException("Invalid native library digest resource");
        }
        String value = new String(bytes, 0, SHA256_HEX_LENGTH, java.nio.charset.StandardCharsets.US_ASCII);
        if (!isValidHash(value)) {
            throw new IOException("Invalid native library digest resource");
        }
        return value;
    }

    private static boolean isValidHash(String value) {
        if (value == null || value.length() != SHA256_HEX_LENGTH) {
            return false;
        }
        for (int index = 0; index < value.length(); index++) {
            char digit = value.charAt(index);
            if (!((digit >= '0' && digit <= '9') || (digit >= 'a' && digit <= 'f'))) {
                return false;
            }
        }
        return true;
    }

    private static byte[] digestFile(Path file) throws IOException {
        MessageDigest digest = sha256Digest();
        byte[] copyBuffer = new byte[COPY_BUFFER_SIZE];
        try (InputStream input = Files.newInputStream(file, StandardOpenOption.READ, LinkOption.NOFOLLOW_LINKS)) {
            int read;
            while ((read = input.read(copyBuffer)) != -1) {
                digest.update(copyBuffer, 0, read);
            }
        }
        return digest.digest();
    }

    private static MessageDigest sha256Digest() {
        try {
            return MessageDigest.getInstance("SHA-256");
        } catch (NoSuchAlgorithmException unavailable) {
            throw new IllegalStateException("SHA-256 is unavailable");
        }
    }

    private static String libraryFileName(String targetId) {
        return switch (targetId) {
            case "windows-x86_64-msvc" -> LIBRARY_BASE_NAME + ".dll";
            case "linux-glibc-x86_64", "linux-glibc-aarch64" -> "lib" + LIBRARY_BASE_NAME + ".so";
            case "macos-x86_64", "macos-aarch64" -> "lib" + LIBRARY_BASE_NAME + ".dylib";
            default -> throw loadFailure();
        };
    }

    private static boolean isLeafName(String name) {
        return name != null && !name.isBlank() && Path.of(name).getNameCount() == 1
                && !name.equals(".") && !name.equals("..");
    }

    private static boolean isX86_64(String arch) {
        return arch.equals("amd64") || arch.equals("x86_64");
    }

    private static boolean isAarch64(String arch) {
        return arch.equals("aarch64") || arch.equals("arm64");
    }

    private static void cleanupFailedExtraction(Path privateDirectory, String libraryName) {
        if (privateDirectory == null) {
            return;
        }
        try {
            if (libraryName != null) {
                Files.deleteIfExists(privateDirectory.resolve(libraryName));
            }
            Files.deleteIfExists(privateDirectory);
        } catch (IOException ignored) {
            // Cleanup is best effort inside a private, uniquely named directory.
        }
    }

    private static UnsatisfiedLinkError loadFailure() {
        return new UnsatisfiedLinkError(LOAD_FAILURE_MESSAGE);
    }

    private static final class NativeAbiMismatchError extends UnsatisfiedLinkError {
        private static final long serialVersionUID = 1L;

        private NativeAbiMismatchError() {
            super(ABI_FAILURE_MESSAGE);
        }
    }
}
