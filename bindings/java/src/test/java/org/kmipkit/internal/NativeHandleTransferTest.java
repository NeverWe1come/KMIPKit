package org.kmipkit.internal;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

final class NativeHandleTransferTest {
    @Test
    void transferDisarmsCleanerAndMakesOwnerUnavailable() {
        NativeHandle owner = NativeHandle.owned(42, NativeExtensionRegistry.TTLV_VALUE);

        assertEquals(42, owner.transfer());
        assertThrows(InvalidInputException.class, owner::get);
        assertDoesNotThrow(owner::close);
    }

    @Test
    void secondTransferFailsInsteadOfReleasingTransferredHandle() {
        NativeHandle owner = NativeHandle.owned(42, NativeExtensionRegistry.TTLV_VALUE);
        owner.transfer();

        assertThrows(InvalidInputException.class, owner::transfer);
        assertDoesNotThrow(owner::close);
    }

    @Test
    void multiHandleTransferDoesNotDetachAnyHandleWhenOneIsAlreadyClosed() {
        NativeHandle first = NativeHandle.owned(42, NativeExtensionRegistry.TTLV_VALUE);
        NativeHandle closed = NativeHandle.owned(84, NativeExtensionRegistry.TTLV_ITEM);
        closed.transfer();

        assertThrows(InvalidInputException.class, () -> NativeHandle.transferAll(first, closed));
        assertEquals(42, first.get());
        assertEquals(42, first.transfer());
        assertDoesNotThrow(first::close);
        assertDoesNotThrow(closed::close);
    }
}
