package org.kmipkit.ide.eclipse;

import org.eclipse.core.commands.AbstractHandler;
import org.eclipse.core.commands.ExecutionEvent;
import org.eclipse.core.commands.ExecutionException;
import org.eclipse.jface.dialogs.MessageDialog;
import org.eclipse.jface.text.ITextSelection;
import org.eclipse.jface.viewers.ISelection;
import org.eclipse.ui.IEditorPart;
import org.eclipse.ui.handlers.HandlerUtil;
import org.eclipse.ui.texteditor.ITextEditor;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.TimeUnit;

public class DecodeTtlvHandler extends AbstractHandler {
    @Override
    public Object execute(ExecutionEvent event) throws ExecutionException {
        IEditorPart editor = HandlerUtil.getActiveEditor(event);
        if (!(editor instanceof ITextEditor textEditor)) {
            return null;
        }
        ISelection selection = textEditor.getSelectionProvider().getSelection();
        if (!(selection instanceof ITextSelection textSel) || textSel.isEmpty()) {
            return null;
        }
        String hexInput = textSel.getText().trim();
        String binary = System.getenv("KMIP_LSP_PATH");
        if (binary == null || binary.isBlank()) {
            binary = "kmip-lsp";
        }
        try {
            Process proc = new ProcessBuilder(binary, "decode", "--hex", hexInput)
                .redirectErrorStream(true)
                .start();
            proc.waitFor(10, TimeUnit.SECONDS);
            String output = new String(proc.getInputStream().readAllBytes(), StandardCharsets.UTF_8);
            MessageDialog.openInformation(
                HandlerUtil.getActiveShell(event),
                "KMIP TTLV Inspector",
                output
            );
        } catch (IOException | InterruptedException ex) {
            throw new ExecutionException("Failed to invoke kmip-lsp", ex);
        }
        return null;
    }
}
