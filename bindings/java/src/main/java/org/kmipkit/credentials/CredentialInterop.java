package org.kmipkit.credentials;

import org.kmipkit.InvalidInputException;
import org.kmipkit.KmipException;
import org.kmipkit.ResourceLimitException;

/** Internal status handling shared by credential envelope conversions. */
final class CredentialInterop {
    static final int SHARED_OWNER_STATUS = 11;

    private CredentialInterop() {}

    static void throwIfFailed(int status, String modelName) {
        switch (status) {
            case 0 -> { return; }
            case 4 -> throw new InvalidInputException(modelName + " structure is invalid");
            case SHARED_OWNER_STATUS -> throw new InvalidInputException(
                    modelName + " structure is still owned by a scoped view");
            case 6 -> throw new ResourceLimitException("configured resource limit was exceeded");
            default -> throw new KmipException("native " + modelName + " conversion failed");
        }
    }
}
