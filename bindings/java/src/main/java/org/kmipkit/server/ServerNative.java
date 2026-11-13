package org.kmipkit.server;

import org.kmipkit.NativeExtensionRegistry;

/** JNI surface for the listener ABI. Public applications should use the typed classes here. */
final class ServerNative {
    static {
        NativeExtensionRegistry.ensureLoaded();
    }

    private ServerNative() {}

    static native long configurationCreate(int transport, int wireEncoding, String host, int port,
            int certificateEncoding, byte[][] certificateChain, byte[] privateKey,
            byte[][] trustCertificates, byte[][] revocationLists, String targetPath,
            long maxRequestBytes);
    static native long listenerBind(long configuration, boolean https);
    static native int listenerPort(long listener);
    static native long listenerAccept(long listener, long timeoutSeconds, int timeoutNanoseconds);
    static native long connectionReceive(long connection, long[] codecLimits,
            long timeoutSeconds, int timeoutNanoseconds);
    static native void connectionSend(long connection, long response,
            long timeoutSeconds, int timeoutNanoseconds);
    static native int[] batchProtocolVersion(long batch);
    static native long batchCount(long batch);
    static native long batchItemAt(long batch, long index);
    static native int requestOperation(long request);
    static native byte[] requestUniqueIdentifier(long request);
    static native long discoverVersionsCount(long request);
    static native int[] discoverVersionAt(long request, long index);
    static native long notifyDeletedAttributeReferenceCount(long request);
    static native org.kmipkit.extensions.AttributeReference notifyDeletedAttributeReferenceAt(
            long request, long index);
    static native long[] notifyAttributes(long request);
    static native int putFunction(long request);
    static native long putObject(long request);
    static native int putObjectType(long object);
    static native int putObjectTag(long object);
    static native long queryFunctionCount(long request);
    static native int queryFunctionAt(long request, long index);
    static native int endpointRole(long request);
    static native long responseBuilderCreate(long batch);
    static native long responseDiscoverVersions(long request, int[][] versions);
    static native long responseNotify(long request);
    static native long responsePut(long request);
    static native long responseQuery(long request, int[] operations, int[] objectTypes,
            int[] extensionTags, String vendorIdentification);
    static native long responseEndpointRole(long request, int role, String vendorIdentification);
    static native void responseBuilderAddSuccess(long builder, long request, long response);
    static native void responseBuilderAddFailure(long builder, long request, int status,
            boolean hasReason, int reason, String message);
    static native long responseBuilderBuild(long builder);
}
