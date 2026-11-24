import * as vscode from "vscode";
import { resolveCoreBinaryPath } from "./binaryManager";
import { KmipLspClient } from "./lspClient";
import { KmipSecretStore } from "./secretStore";
import { KmipExplorerProvider } from "./views/kmipExplorerView";
import { TtlvInspectorPanel } from "./views/ttlvInspectorPanel";

let lspClient: KmipLspClient | undefined;
let outputChannel: vscode.OutputChannel | undefined;

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  outputChannel = vscode.window.createOutputChannel("KMIPKit");
  lspClient = new KmipLspClient();

  // Initialize SecretStorage wrapper for Phase 2 profile credentials
  const _secretStore = new KmipSecretStore(context.secrets);

  context.subscriptions.push(
    vscode.window.registerTreeDataProvider("kmipkit.explorer", new KmipExplorerProvider())
  );

  await startServer(context);

  context.subscriptions.push(
    vscode.commands.registerCommand("kmip.openInspector", () => {
      if (lspClient) {
        TtlvInspectorPanel.createOrShow(context.extensionUri, lspClient);
      }
    }),
    vscode.commands.registerCommand("kmip.openPlayground", () => {
      if (lspClient) {
        TtlvInspectorPanel.createOrShow(context.extensionUri, lspClient);
      }
    }),
    vscode.commands.registerCommand("kmip.decodeTtlvSelection", () => {
      const editor = vscode.window.activeTextEditor;
      const selectionText = editor?.document.getText(editor.selection).trim();
      if (!selectionText) {
        vscode.window.showWarningMessage("Select a hex or Base64 TTLV payload first.");
        return;
      }
      if (lspClient) {
        TtlvInspectorPanel.createOrShow(context.extensionUri, lspClient, selectionText);
      }
    }),
    vscode.commands.registerCommand("kmip.restartServer", async () => {
      await lspClient?.stop();
      await startServer(context);
      vscode.window.showInformationMessage("KMIPKit Language Server restarted.");
    })
  );
}

async function startServer(context: vscode.ExtensionContext): Promise<void> {
  if (!lspClient || !outputChannel) return;
  const cfg = vscode.workspace.getConfiguration("kmip");
  const configuredPath = cfg.get<string>("core.binaryPath");
  const pinnedChecksums = cfg.get<Record<string, string>>("core.pinnedSha256");
  const workspaceRoots = vscode.workspace.workspaceFolders?.map((f) => f.uri.fsPath) ?? [];

  try {
    const binPath = await resolveCoreBinaryPath({
      configuredPath,
      extensionPath: context.extensionPath,
      globalStoragePath: context.globalStorageUri.fsPath,
      workspaceRoots,
      pinnedChecksums
    });
    outputChannel.appendLine(`Starting kmip-lsp: ${binPath}`);
    await lspClient.start(binPath, outputChannel);
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    outputChannel.appendLine(`kmip-lsp start failed: ${msg}`);
    vscode.window.showWarningMessage(`KMIPKit: ${msg}`);
  }
}

export async function deactivate(): Promise<void> {
  await lspClient?.stop();
}
