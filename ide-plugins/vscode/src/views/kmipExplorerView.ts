import * as vscode from "vscode";

export class KmipExplorerProvider implements vscode.TreeDataProvider<KmipTreeItem> {
  private readonly items: KmipTreeItem[] = [
    new KmipTreeItem("Open TTLV Inspector", "kmip.openInspector", "inspect"),
    new KmipTreeItem("Open Playground (Phase 2)", "kmip.openPlayground", "play"),
    new KmipTreeItem("Restart kmip-lsp Server", "kmip.restartServer", "refresh")
  ];

  getTreeItem(element: KmipTreeItem): vscode.TreeItem {
    return element;
  }

  getChildren(): Thenable<KmipTreeItem[]> {
    return Promise.resolve(this.items);
  }
}

class KmipTreeItem extends vscode.TreeItem {
  constructor(label: string, commandId: string, iconName: string) {
    super(label, vscode.TreeItemCollapsibleState.None);
    this.command = { command: commandId, title: label };
    this.iconPath = new vscode.ThemeIcon(iconName);
  }
}
