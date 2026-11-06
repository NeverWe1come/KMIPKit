package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;

/** Optional connect, write, read, and total timeout overrides for one exchange. */
public record RequestOptions(
        TransportConfiguration.Timeout connectTimeout,
        TransportConfiguration.Timeout writeTimeout,
        TransportConfiguration.Timeout readTimeout,
        TransportConfiguration.Timeout totalTimeout) {
    public RequestOptions {
        if (connectTimeout == null || writeTimeout == null || readTimeout == null || totalTimeout == null) {
            throw new InvalidInputException("request timeout options are invalid");
        }
    }

    /** Inherit all four timeout values from the constructed client policy. */
    public static RequestOptions defaults() {
        TransportConfiguration.Timeout inherited = TransportConfiguration.Timeout.defaultValue();
        return new RequestOptions(inherited, inherited, inherited, inherited);
    }
}
