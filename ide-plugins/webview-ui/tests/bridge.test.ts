// @vitest-environment jsdom
import { describe, expect, it, vi } from "vitest";
import { createIdeBridge } from "../src/bridge/ideBridge";
import { TtlvInspectorView } from "../src/views/ttlv-inspector/TtlvInspector";
import { PlaygroundView } from "../src/views/playground/Playground";

describe("IdeBridge & TtlvInspectorView", () => {
  it("uses JCEF bridge when __kmipJcefSend is present and delivers incoming messages", () => {
    const jcefSend = vi.fn();
    window.__kmipJcefSend = jcefSend;

    const bridge = createIdeBridge();
    const received: unknown[] = [];
    bridge.onMessage((type, payload) => received.push({ type, payload }));

    bridge.send("decodeTtlv", { input: "420078" });
    expect(jcefSend).toHaveBeenCalledWith(
      JSON.stringify({ type: "decodeTtlv", payload: { input: "420078" } })
    );

    window.__kmipReceiveMessage?.("decodeError", { message: "bad input" });
    expect(received).toEqual([{ type: "decodeError", payload: { message: "bad input" } }]);

    delete window.__kmipJcefSend;
  });

  it("renders redacted sensitive nodes with Reveal button", () => {
    const container = document.createElement("div");
    const send = vi.fn();
    const bridge = {
      send,
      onMessage: () => () => {},
      getTheme: () => "dark" as const
    };

    const view = new TtlvInspectorView(container, bridge);
    view.setResult(
      {
        rawHex: "42004201000000104200430800000004****************",
        byteLength: 24,
        alerts: [],
        root: {
          tagHex: "0x420042",
          tagName: "Key Block",
          itemType: "Structure",
          offset: 0,
          headerLength: 8,
          valueOffset: 8,
          valueLength: 16,
          paddedLength: 16,
          displayValue: "Structure (1 items)",
          redacted: false,
          sensitive: false,
          children: [
            {
              tagHex: "0x420043",
              tagName: "Key Material",
              itemType: "ByteString",
              offset: 8,
              headerLength: 8,
              valueOffset: 16,
              valueLength: 4,
              paddedLength: 8,
              displayValue: "[REDACTED - Click to reveal] (4 bytes)",
              redacted: true,
              sensitive: true,
              children: []
            }
          ]
        }
      },
      "420042..."
    );

    expect(container.textContent).toContain("[REDACTED - Click to reveal]");
    const revealBtn = Array.from(container.querySelectorAll("button")).find(
      (b) => b.textContent === "Reveal"
    );
    expect(revealBtn).toBeTruthy();
    revealBtn?.click();
    expect(send).toHaveBeenCalledWith(
      "decodeTtlv",
      expect.objectContaining({ revealOffsets: [8] })
    );
  });

  it("renders Playground with PROD guardrail when Destroy is selected", () => {
    const container = document.createElement("div");
    const send = vi.fn();
    const bridge = {
      send,
      onMessage: () => () => {},
      getTheme: () => "dark" as const
    };
    const pg = new PlaygroundView(container, bridge);
    pg.render();

    const envSelect = container.querySelector<HTMLSelectElement>("#pg-env")!;
    const opSelect = container.querySelector<HTMLSelectElement>("#pg-op")!;
    envSelect.value = "PROD";
    envSelect.dispatchEvent(new Event("change"));
    opSelect.value = "Destroy";
    opSelect.dispatchEvent(new Event("change"));

    expect(container.textContent).toContain("PROD Guardrail Active");
  });
});
