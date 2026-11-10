package org.kmipkit.ttlv;

import org.kmipkit.InvalidInputException;

/** The source Item has a different tag from the selected standard attribute. */
public final class TypedAttributeTagMismatchException extends InvalidInputException {
    private static final long serialVersionUID = 1L;

    public TypedAttributeTagMismatchException(String message) {
        super(message);
    }
}
