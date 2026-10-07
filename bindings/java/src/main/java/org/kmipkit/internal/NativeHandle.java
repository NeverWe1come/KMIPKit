package org.kmipkit.internal;

import java.lang.ref.Cleaner;
import java.util.concurrent.atomic.AtomicLong;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Shared lifecycle state for Java wrappers around opaque native handles. */
public final class NativeHandle {
    private static final Cleaner CLEANER = Cleaner.create();
    private static final Object STATE_LOCK = new Object();

    private final State state;
    private final Cleaner.Cleanable cleanable;

    private NativeHandle(long value, int handleKind) {
        if (value == 0) {
            throw new InvalidInputException("native operation returned an empty handle");
        }
        state = new State(value, handleKind);
        cleanable = CLEANER.register(this, state);
    }

    public static NativeHandle owned(long value, int handleKind) {
        return new NativeHandle(value, handleKind);
    }

    public long get() {
        long value = state.value.get();
        if (value == 0) {
            throw new InvalidInputException("native handle is closed");
        }
        return value;
    }

    /** Transfers the native handle to a consuming native call without releasing it. */
    public long transfer() {
        return transferAll(this)[0];
    }

    /**
     * Atomically transfers several handles for one consuming native call.
     *
     * <p>All owners remain active if any owner is null, closed, or repeated.</p>
     */
    public static long[] transferAll(NativeHandle... handles) {
        if (handles == null) {
            throw new InvalidInputException("native handle transfer inputs are invalid");
        }

        long[] transferred = new long[handles.length];
        synchronized (STATE_LOCK) {
            for (int index = 0; index < handles.length; index++) {
                NativeHandle handle = handles[index];
                if (handle == null || handle.state.value.get() == 0) {
                    throw new InvalidInputException("native handle is closed");
                }
                for (int prior = 0; prior < index; prior++) {
                    if (handles[prior] == handle) {
                        throw new InvalidInputException("native handle transfer inputs are repeated");
                    }
                }
            }
            for (int index = 0; index < handles.length; index++) {
                transferred[index] = handles[index].state.value.getAndSet(0);
            }
        }
        for (NativeHandle handle : handles) {
            handle.cleanable.clean();
        }
        return transferred;
    }

    public void close() {
        long value;
        synchronized (STATE_LOCK) {
            value = state.value.getAndSet(0);
        }
        if (value != 0) {
            NativeExtensionRegistry.ensureLoaded();
            NativeExtensionRegistry.release(state.handleKind, value);
        }
        cleanable.clean();
    }

    private static final class State implements Runnable {
        private final AtomicLong value;
        private final int handleKind;

        private State(long value, int handleKind) {
            this.value = new AtomicLong(value);
            this.handleKind = handleKind;
        }

        @Override
        public void run() {
            long toRelease;
            synchronized (STATE_LOCK) {
                toRelease = value.getAndSet(0);
            }
            if (toRelease != 0) {
                try {
                    NativeExtensionRegistry.ensureLoaded();
                    NativeExtensionRegistry.release(handleKind, toRelease);
                } catch (LinkageError ignored) {
                    // Cleaner threads cannot report unload or shutdown failures to callers.
                }
            }
        }
    }
}
