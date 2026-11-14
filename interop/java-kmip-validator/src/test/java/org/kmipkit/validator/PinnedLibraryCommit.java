package org.kmipkit.validator;

import com.fasterxml.jackson.databind.ObjectMapper;

import java.nio.file.Files;
import java.nio.file.Path;

final class PinnedLibraryCommit {
    private PinnedLibraryCommit() { }

    static String read() throws Exception {
        return new ObjectMapper().readTree(Files.readString(Path.of("library-pin.json")))
                .path("commit").asText();
    }
}
