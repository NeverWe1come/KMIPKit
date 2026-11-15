package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;

/** The source Item's TTLV value type is not permitted for the selected attribute. */
public final class TypedAttributeValueTypeMismatchException extends InvalidInputException {
    private static final long serialVersionUID = 1L;

    public TypedAttributeValueTypeMismatchException(String message) {
        super(message);
    }
}
