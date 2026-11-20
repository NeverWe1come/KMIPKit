package org.kmipkit.internal;

import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import java.util.IdentityHashMap;
import java.util.function.Function;
import java.util.function.LongFunction;
import java.util.function.Supplier;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Shared lifecycle state for Java wrappers around opaque native handles. */
public final class NativeHandle {
    private static final Cleaner CLEANER = Cleaner.create();
    // Protects lifecycle transitions only. Native calls execute outside this monitor.
    private static final Object STATE_LOCK = new Object();
    private static final ThreadLocal<Scope> CURRENT_SCOPE = new ThreadLocal<>();

    private final State state;
    private final Cleaner.Cleanable cleanable;

    private NativeHandle(long value, int handleKind) {
        if (value == 0) throw new InvalidInputException("native operation returned an empty handle");
        state = new State(value, handleKind);
        cleanable = CLEANER.register(this, state);
    }

    public static NativeHandle owned(long value, int handleKind) {
        return new NativeHandle(value, handleKind);
    }

    public long get() {
        synchronized (STATE_LOCK) {
            if (state.value == 0) throw new InvalidInputException("native handle is closed");
            Scope scope = CURRENT_SCOPE.get();
            if (scope != null) scope.retain(this);
            Reference.reachabilityFence(this);
            return state.value;
        }
    }

    /** Holds lifecycle ownership until a synchronous native operation returns. */
    public <T> T withValue(LongFunction<T> operation) {
        if (operation == null) throw new InvalidInputException("native handle operation is invalid");
        return withValues(new NativeHandle[] {this}, values -> operation.apply(values[0]));
    }

    /** Retains handles obtained within this scope until the native call returns. */
    public static <T> T withNativeHandles(Supplier<T> operation) {
        if (operation == null) throw new InvalidInputException("native handle operation is invalid");
        if (CURRENT_SCOPE.get() != null) return operation.get();
        Scope scope = new Scope();
        CURRENT_SCOPE.set(scope);
        try {
            return operation.get();
        } finally {
            CURRENT_SCOPE.remove();
            scope.close();
        }
    }

    /** Holds several native handles for one synchronous native operation. */
    public static <T> T withValues(NativeHandle[] handles, Function<long[], T> operation) {
        if (handles == null || operation == null) {
            throw new InvalidInputException("native handle operation is invalid");
        }
        return withNativeHandles(() -> {
            long[] values = new long[handles.length];
            synchronized (STATE_LOCK) {
                for (NativeHandle handle : handles) {
                    if (handle == null || handle.state.value == 0) {
                        throw new InvalidInputException("native handle is closed");
                    }
                }
                for (int index = 0; index < handles.length; index++) {
                    values[index] = handles[index].get();
                }
            }
            return operation.apply(values);
        });
    }

    /** Transfers the native handle to a consuming native call without releasing it. */
    public long transfer() { return transferAll(this)[0]; }

    /** Atomically transfers distinct handles, respecting active native ownership scopes. */
    public static long[] transferAll(NativeHandle... handles) {
        if (handles == null) throw new InvalidInputException("native handle transfer inputs are invalid");
        long[] transferred = new long[handles.length];
        synchronized (STATE_LOCK) {
            Scope scope = CURRENT_SCOPE.get();
            for (int index = 0; index < handles.length; index++) {
                NativeHandle handle = handles[index];
                if (handle == null) throw new InvalidInputException("native handle is closed");
                boolean ownLoan = scope != null && scope.owners.containsKey(handle.state);
                boolean closingLoan = ownLoan && handle.state.waitingForRelease && handle.state.retired != 0;
                if (handle.state.value == 0 && !closingLoan) {
                    throw new InvalidInputException("native handle is closed");
                }
                // A scoped caller also uses transfer to finish a consuming C call.
                // External close must not release an owner already consumed by that call.
                if (!ownLoan && handle.state.loans != 0) {
                    throw new InvalidInputException("native handle is in use by another thread");
                }
                for (int prior = 0; prior < index; prior++) {
                    if (handles[prior] == handle) {
                        throw new InvalidInputException("native handle transfer inputs are repeated");
                    }
                }
            }
            for (int index = 0; index < handles.length; index++) {
                State state = handles[index].state;
                transferred[index] = state.value != 0 ? state.value : state.retired;
                state.value = 0;
                state.retired = 0;
            }
        }
        for (NativeHandle handle : handles) handle.cleanable.clean();
        return transferred;
    }

    /** Invalidates this owner immediately and releases it after outstanding calls finish. */
    public void close() { cleanable.clean(); }

    private static void release(int kind, long value) {
        if (value == 0) return;
        NativeExtensionRegistry.ensureLoaded();
        NativeExtensionRegistry.release(kind, value);
    }

    private static final class Scope {
        private final IdentityHashMap<State, NativeHandle> owners = new IdentityHashMap<>();

        // Called with STATE_LOCK held. Each thread borrows a state once per outer scope.
        private void retain(NativeHandle owner) {
            if (!owners.containsKey(owner.state)) {
                owners.put(owner.state, owner);
                owner.state.loans++;
            }
        }

        private void close() {
            RuntimeException failure = null;
            for (State state : owners.keySet()) {
                long retired = 0;
                synchronized (STATE_LOCK) {
                    state.loans--;
                    if (state.loans == 0 && !state.waitingForRelease) {
                        retired = state.retired;
                        state.retired = 0;
                    }
                    STATE_LOCK.notifyAll();
                }
                try {
                    release(state.handleKind, retired);
                } catch (RuntimeException error) {
                    if (failure == null) failure = error;
                    else failure.addSuppressed(error);
                }
            }
            Reference.reachabilityFence(owners);
            if (failure != null) throw failure;
        }
    }

    private static final class State implements Runnable {
        private long value;
        private long retired;
        private int loans;
        private boolean waitingForRelease;
        private final int handleKind;

        private State(long value, int handleKind) {
            this.value = value;
            this.handleKind = handleKind;
        }

        @Override
        public void run() {
            long toRelease;
            boolean interrupted = false;
            synchronized (STATE_LOCK) {
                retired = value;
                value = 0;
                Scope scope = CURRENT_SCOPE.get();
                if (scope != null && scope.owners.containsKey(this)) return;
                waitingForRelease = true;
                while (loans != 0) {
                    try {
                        STATE_LOCK.wait();
                    } catch (InterruptedException error) {
                        interrupted = true;
                    }
                }
                waitingForRelease = false;
                toRelease = retired;
                retired = 0;
            }
            if (interrupted) Thread.currentThread().interrupt();
            try {
                release(handleKind, toRelease);
            } catch (LinkageError ignored) {
                // Cleaner threads cannot report unload or shutdown failures to callers.
            }
        }
    }
}
