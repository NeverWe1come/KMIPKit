package org.kmipkit.extensions;

import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ResourceLimitException;

/** Explicit profile IDs selected for one client exchange. */
public record ProfileValidationSelection(List<String> profileIds) {
    public ProfileValidationSelection {
        if (profileIds == null) throw new InvalidInputException("profile selection is invalid");
        if (profileIds.stream().anyMatch(value -> value == null)) {
            throw new InvalidInputException("profile selection is invalid");
        }
        if (profileIds.size() > 4_096) {
            throw new ResourceLimitException("profile selection exceeds the configured limit");
        }
        long totalBytes = 0;
        for (String profileId : profileIds) {
            if (profileId.length() > 4_096) {
                throw new ResourceLimitException("profile selection exceeds the configured limit");
            }
            int length = profileId.getBytes(java.nio.charset.StandardCharsets.UTF_8).length;
            if (length > 4_096 || totalBytes > 16L * 1024 * 1024 - length) {
                throw new ResourceLimitException("profile selection exceeds the configured limit");
            }
            totalBytes += length;
        }
        profileIds = List.copyOf(profileIds);
    }

    @Override
    public String toString() {
        return "ProfileValidationSelection[profileCount=" + profileIds.size() + "]";
    }
}
