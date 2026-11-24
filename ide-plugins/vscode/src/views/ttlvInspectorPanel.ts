import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import type { KmipLspClient } from "../lspClient";

export class TtlvInspectorPanel {
  public static currentPanel: TtlvInspectorPanel | undefined;
  private readonly panel: vscode.WebviewPanel;
  private disposables: vscode.Disposable[] = [];
  private pendingInput: string | undefined;
  private isReady = false;

  public static createOrShow(
    extensionUri: vscode.Uri,
    lspClient: KmipLspClient,
    initialInput?: string
  ): TtlvInspectorPanel {
    const column = vscode.window.activeTextEditor?.viewColumn
      ? vscode.ViewColumn.Beside
      : vscode.ViewColumn.One;

    if (TtlvInspectorPanel.currentPanel) {
      TtlvInspectorPanel.currentPanel.panel.reveal(column);
      if (initialInput) {
        TtlvInspectorPanel.currentPanel.decodeAndShow(initialInput);
      }
      return TtlvInspectorPanel.currentPanel;
    }

    const mediaWebviewRoot = vscode.Uri.joinPath(extensionUri, "media", "webview");
    const panel = vscode.window.createWebviewPanel(
      "kmipkit.ttlvInspector",
      "KMIP TTLV Inspector",
      column,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [mediaWebviewRoot]
      }
    );

    TtlvInspectorPanel.currentPanel = new TtlvInspectorPanel(
      panel,
      extensionUri,
      lspClient,
      initialInput
    );
    return TtlvInspectorPanel.currentPanel;
  }

  private constructor(
    panel: vscode.WebviewPanel,
    private readonly extensionUri: vscode.Uri,
    private readonly lspClient: KmipLspClient,
    initialInput?: string
  ) {
    this.panel = panel;
    this.pendingInput = initialInput;
    this.panel.webview.html = this.getHtml();

    this.panel.onDidDispose(() => this.dispose(), null, this.disposables);

    this.panel.webview.onDidReceiveMessage(
      async (msg: { type: string; payload?: Record<string, unknown> }) => {
        if (msg.type === "ready") {
          this.isReady = true;
          if (this.pendingInput) {
            const input = this.pendingInput;
            this.pendingInput = undefined;
            await this.decodeAndShow(input);
          }
        } else if (msg.type === "decodeTtlv" && msg.payload) {
          const input = String(msg.payload.input ?? "");
          const revealOffsets = Array.isArray(msg.payload.revealOffsets)
            ? (msg.payload.revealOffsets as number[])
            : [];
          await this.decodeAndShow(input, revealOffsets);
        } else if (msg.type === "executeRequest" && msg.payload) {
          try {
            const result = await this.lspClient.executeRequest(msg.payload);
            await this.panel.webview.postMessage({ type: "executeResult", payload: result });
          } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            await this.panel.webview.postMessage({ type: "executeError", payload: { message } });
          }
        } else if (msg.type === "generateTest" && msg.payload) {
          try {
            const input = String(msg.payload.input ?? "");
            const language = String(msg.payload.language ?? "rust");
            const generated = await this.lspClient.generateTest(input, language);
            const doc = await vscode.workspace.openTextDocument({
              content: generated.code,
              language: generated.language
            });
            await vscode.window.showTextDocument(doc, vscode.ViewColumn.Beside);
          } catch (err) {
            const message = err instanceof Error ? err.message : String(err);
            vscode.window.showErrorMessage(`KMIPKit Test Generator: ${message}`);
          }
        }
      },
      null,
      this.disposables
    );
  }

  public async decodeAndShow(input: string, revealOffsets: number[] = []): Promise<void> {
    if (!this.isReady) {
      this.pendingInput = input;
      return;
    }
    try {
      const result = await this.lspClient.decodeTtlv(input, revealOffsets);
      await this.panel.webview.postMessage({
        type: "decodeResult",
        payload: { result, sourceInput: input }
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      await this.panel.webview.postMessage({
        type: "decodeError",
        payload: { message }
      });
    }
  }

  private getHtml(): string {
    const webviewDir = path.join(this.extensionUri.fsPath, "media", "webview");
    const scriptUri = this.panel.webview.asWebviewUri(
      vscode.Uri.file(path.join(webviewDir, "assets", "app.js"))
    );
    const styleUri = this.panel.webview.asWebviewUri(
      vscode.Uri.file(path.join(webviewDir, "assets", "app.css"))
    );
    const nonce = crypto.randomBytes(16).toString("hex");
    const exists = fs.existsSync(path.join(webviewDir, "assets", "app.js"));

    if (!exists) {
      return `<!DOCTYPE html><html><body><p>webview-ui bundle not found. Run <code>ide-plugins/scripts/build-webview.sh</code>.</p></body></html>`;
    }

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${this.panel.webview.cspSource}; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <link rel="stylesheet" href="${styleUri}">
  <title>KMIP TTLV Inspector</title>
</head>
<body>
  <div id="app"></div>
  <script nonce="${nonce}" type="module" src="${scriptUri}"></script>
</body>
</html>`;
  }

  private dispose(): void {
    TtlvInspectorPanel.currentPanel = undefined;
    while (this.disposables.length) {
      this.disposables.pop()?.dispose();
    }
  }
}
