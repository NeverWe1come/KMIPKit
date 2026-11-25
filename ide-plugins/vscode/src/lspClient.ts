import * as vscode from "vscode";
import {
  LanguageClient,
  type LanguageClientOptions,
  type ServerOptions
} from "vscode-languageclient/node";

export class KmipLspClient {
  private client: LanguageClient | undefined;

  async start(binaryPath: string, outputChannel: vscode.OutputChannel): Promise<void> {
    const serverOptions: ServerOptions = {
      command: binaryPath,
      args: ["--stdio"]
    };

    const clientOptions: LanguageClientOptions = {
      documentSelector: [
        { scheme: "file", language: "kmip-json" },
        { scheme: "file", language: "kmip-ttlv" },
        { scheme: "file", language: "rust" },
        { scheme: "file", language: "java" },
        { scheme: "file", language: "python" },
        { scheme: "file", language: "c" },
        { scheme: "file", pattern: "**/*.kmip.json" },
        { scheme: "file", pattern: "**/*.kmip-config.json" },
        { scheme: "file", pattern: "**/*.kmip-policy.json" }
      ],
      outputChannel
    };

    this.client = new LanguageClient(
      "kmipkit-lsp",
      "KMIPKit Language Server",
      serverOptions,
      clientOptions
    );

    await this.client.start();
  }

  async stop(): Promise<void> {
    if (this.client) {
      await this.client.stop();
      this.client = undefined;
    }
  }

  async decodeTtlv(
    input: string,
    revealOffsets: number[] = []
  ): Promise<unknown> {
    if (!this.client) {
      throw new Error("KMIPKit Language Server is not running.");
    }
    return this.client.sendRequest("kmip/decodeTtlv", {
      input,
      format: "auto",
      revealOffsets
    });
  }

  async lookupSpec(query: string): Promise<unknown> {
    if (!this.client) {
      throw new Error("KMIPKit Language Server is not running.");
    }
    return this.client.sendRequest("kmip/lookupSpec", { query });
  }

  async executeRequest(params: unknown): Promise<unknown> {
    if (!this.client) {
      throw new Error("KMIPKit Language Server is not running.");
    }
    return this.client.sendRequest("kmip/executeRequest", params);
  }

  async generateTest(input: string, language: string): Promise<{ language: string; code: string }> {
    if (!this.client) {
      throw new Error("KMIPKit Language Server is not running.");
    }
    return this.client.sendRequest("kmip/generateTest", { input, language });
  }
}
