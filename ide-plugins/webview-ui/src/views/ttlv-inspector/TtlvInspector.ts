import type { IdeBridge } from "../../bridge/ideBridge";
import type { DecodeResult, TtlvNode } from "./types";

export class TtlvInspectorView {
  private lastInput = "";
  private revealOffsets = new Set<number>();
  private result: DecodeResult | null = null;
  private errorMessage: string | null = null;
  private selectedNode: TtlvNode | null = null;

  constructor(
    private readonly container: HTMLElement,
    private readonly bridge: IdeBridge
  ) {}

  setResult(result: DecodeResult, sourceInput?: string): void {
    if (sourceInput !== undefined) {
      this.lastInput = sourceInput;
    }
    this.result = result;
    this.errorMessage = null;
    this.selectedNode = result.root;
    this.render();
  }

  setInputAndDecode(rawInput: string): void {
    this.lastInput = rawInput;
    this.revealOffsets.clear();
    this.requestDecode();
  }

  setError(message: string): void {
    this.errorMessage = message;
    this.render();
  }

  private requestDecode(): void {
    if (!this.lastInput.trim()) return;
    this.bridge.send("decodeTtlv", {
      input: this.lastInput,
      format: "auto",
      revealOffsets: Array.from(this.revealOffsets)
    });
  }

  render(): void {
    this.container.innerHTML = "";

    const inputEl = document.createElement("textarea");
    inputEl.className = "kmip-input";
    inputEl.placeholder = "Paste TTLV Hex or Base64 payload (or use 'KMIP: Decode TTLV Selection' in editor)...";
    inputEl.value = this.lastInput;
    inputEl.addEventListener("input", () => {
      this.lastInput = inputEl.value;
    });

    const toolbar = document.createElement("div");
    toolbar.className = "kmip-toolbar";

    const decodeBtn = document.createElement("button");
    decodeBtn.className = "kmip-btn";
    decodeBtn.textContent = "Decode TTLV";
    decodeBtn.addEventListener("click", () => {
      this.revealOffsets.clear();
      this.requestDecode();
    });

    const hideAllBtn = document.createElement("button");
    hideAllBtn.className = "kmip-btn-secondary";
    hideAllBtn.textContent = "Re-redact Sensitive";
    hideAllBtn.addEventListener("click", () => {
      this.revealOffsets.clear();
      this.requestDecode();
    });

    const genLang = document.createElement("select");
    genLang.className = "kmip-btn-secondary";
    genLang.innerHTML = `
      <option value="rust">Rust Test</option>
      <option value="python">Python Test</option>
      <option value="java">Java Test</option>
      <option value="c">C Test</option>
    `;

    const genTestBtn = document.createElement("button");
    genTestBtn.className = "kmip-btn-secondary";
    genTestBtn.textContent = "Export Anonymized Test";
    genTestBtn.addEventListener("click", () => {
      if (!this.lastInput.trim()) return;
      this.bridge.send("generateTest", {
        input: this.lastInput,
        language: genLang.value
      });
    });

    toolbar.appendChild(decodeBtn);
    toolbar.appendChild(hideAllBtn);
    toolbar.appendChild(genLang);
    toolbar.appendChild(genTestBtn);

    this.container.appendChild(inputEl);
    this.container.appendChild(toolbar);

    if (this.errorMessage) {
      const err = document.createElement("div");
      err.className = "error-banner";
      err.textContent = this.errorMessage;
      this.container.appendChild(err);
    }

    if (!this.result) return;

    for (const alert of this.result.alerts) {
      const banner = document.createElement("div");
      banner.className = "alert-banner";
      banner.textContent = `[${alert.code}] ${alert.message} (offset ${alert.offset})`;
      this.container.appendChild(banner);
    }

    const split = document.createElement("div");
    split.className = "kmip-split";

    const treeCard = document.createElement("div");
    treeCard.className = "kmip-card";
    const treeTitle = document.createElement("h3");
    treeTitle.textContent = `TTLV Tree (${this.result.byteLength} bytes)`;
    treeCard.appendChild(treeTitle);
    treeCard.appendChild(this.renderTreeNode(this.result.root, true));

    const hexCard = document.createElement("div");
    hexCard.className = "kmip-card";
    const hexTitle = document.createElement("h3");
    hexTitle.textContent = "Synchronized Hex View (Blue: Header · Green: Value · Yellow: Padding)";
    hexCard.appendChild(hexTitle);
    hexCard.appendChild(this.renderHexGrid(this.result.rawHex));

    split.appendChild(treeCard);
    split.appendChild(hexCard);
    this.container.appendChild(split);
  }

  private renderTreeNode(node: TtlvNode, isRoot = false): HTMLElement {
    const wrapper = document.createElement("div");
    if (!isRoot) wrapper.className = "ttlv-node";

    const row = document.createElement("div");
    row.className = "ttlv-row";
    if (this.selectedNode?.offset === node.offset) {
      row.classList.add("selected");
    }

    const tagSpan = document.createElement("span");
    tagSpan.className = "ttlv-tag";
    tagSpan.textContent = node.tagHex;

    const nameSpan = document.createElement("span");
    nameSpan.className = "ttlv-name";
    nameSpan.textContent = node.tagName;

    const typeBadge = document.createElement("span");
    typeBadge.className = "ttlv-badge";
    typeBadge.textContent = `${node.itemType} (${node.valueLength}B)`;

    const valSpan = document.createElement("span");
    valSpan.className = node.redacted ? "ttlv-val ttlv-redacted" : "ttlv-val";
    valSpan.textContent = node.displayValue;

    row.appendChild(tagSpan);
    row.appendChild(nameSpan);
    row.appendChild(typeBadge);
    row.appendChild(valSpan);

    if (node.sensitive && node.redacted) {
      const revealBtn = document.createElement("button");
      revealBtn.className = "kmip-btn-secondary";
      revealBtn.textContent = "Reveal";
      revealBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        this.revealOffsets.add(node.offset);
        this.requestDecode();
      });
      row.appendChild(revealBtn);
    }

    row.addEventListener("click", () => {
      this.selectedNode = node;
      this.render();
    });

    wrapper.appendChild(row);
    for (const child of node.children) {
      wrapper.appendChild(this.renderTreeNode(child));
    }
    return wrapper;
  }

  private renderHexGrid(rawHex: string): HTMLElement {
    const grid = document.createElement("div");
    grid.className = "hex-grid";
    const sel = this.selectedNode;

    for (let i = 0; i < rawHex.length; i += 2) {
      const byteIndex = i / 2;
      const cell = document.createElement("span");
      cell.className = "hex-byte";
      cell.textContent = rawHex.slice(i, i + 2).toUpperCase();
      cell.title = `Offset ${byteIndex} (0x${byteIndex.toString(16)})`;

      if (sel) {
        if (byteIndex >= sel.offset && byteIndex < sel.offset + sel.headerLength) {
          cell.classList.add("hl-header");
        } else if (byteIndex >= sel.valueOffset && byteIndex < sel.valueOffset + sel.valueLength) {
          cell.classList.add("hl-value");
        } else if (byteIndex >= sel.valueOffset + sel.valueLength && byteIndex < sel.valueOffset + sel.paddedLength) {
          cell.classList.add("hl-padding");
        }
      }

      cell.addEventListener("click", () => {
        if (!this.result) return;
        const found = this.findDeepestNodeAtOffset(this.result.root, byteIndex);
        if (found) {
          this.selectedNode = found;
          this.render();
        }
      });

      grid.appendChild(cell);
    }
    return grid;
  }

  private findDeepestNodeAtOffset(node: TtlvNode, byteOffset: number): TtlvNode | null {
    const end = node.valueOffset + node.paddedLength;
    if (byteOffset < node.offset || byteOffset >= end) return null;
    for (const child of node.children) {
      const match = this.findDeepestNodeAtOffset(child, byteOffset);
      if (match) return match;
    }
    return node;
  }
}
