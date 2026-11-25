package org.kmipkit.ide.eclipse;

import org.eclipse.lsp4e.server.ProcessStreamConnectionProvider;
import java.util.List;

public class KmipLanguageServerConnectionProvider extends ProcessStreamConnectionProvider {
    public KmipLanguageServerConnectionProvider() {
        String binary = System.getenv("KMIP_LSP_PATH");
        if (binary == null || binary.isBlank()) {
            binary = "kmip-lsp";
        }
        setCommands(List.of(binary, "--stdio"));
    }
}
