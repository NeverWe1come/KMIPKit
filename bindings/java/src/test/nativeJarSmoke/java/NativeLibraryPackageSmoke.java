public final class NativeLibraryPackageSmoke {
    private NativeLibraryPackageSmoke() {}

    public static void main(String[] args) throws ClassNotFoundException {
        Class.forName("org.kmipkit.NativeExtensionRegistry");
        System.out.print("bundled JNI loaded");
    }
}
