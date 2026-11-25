import "./styles/theme.css";
import { createIdeBridge } from "./bridge/ideBridge";
import { TtlvInspectorView } from "./views/ttlv-inspector/TtlvInspector";
import type { DecodeResult } from "./views/ttlv-inspector/types";
import { PlaygroundView } from "./views/playground/Playground";

const bridge = createIdeBridge();
document.body.dataset.theme = bridge.getTheme();

const root = document.getElementById("app");
if (root) {
  root.innerHTML = `
    <div class="kmip-shell">
      <div class="kmip-tabs">
        <button class="kmip-tab active" data-tab="inspector">TTLV Inspector</button>
        <button class="kmip-tab" data-tab="playground">Playground</button>
      </div>
      <div class="kmip-content" id="kmip-tab-content"></div>
    </div>
  `;

  const contentEl = document.getElementById("kmip-tab-content")!;
  const inspector = new TtlvInspectorView(contentEl, bridge);
  const playground = new PlaygroundView(contentEl, bridge);

  let activeTab: "inspector" | "playground" = "inspector";

  const renderTab = () => {
    if (activeTab === "inspector") {
      inspector.render();
    } else {
      playground.render();
    }
  };

  root.querySelectorAll<HTMLButtonElement>(".kmip-tab").forEach((btn) => {
    btn.addEventListener("click", () => {
      root.querySelectorAll(".kmip-tab").forEach((b) => b.classList.remove("active"));
      btn.classList.add("active");
      activeTab = btn.dataset.tab === "playground" ? "playground" : "inspector";
      renderTab();
    });
  });

  bridge.onMessage((type, payload) => {
    if (type === "decodeResult") {
      const p = payload as { result: DecodeResult; sourceInput?: string };
      activeTab = "inspector";
      inspector.setResult(p.result, p.sourceInput);
    } else if (type === "decodeInput") {
      const p = payload as { input: string };
      activeTab = "inspector";
      inspector.setInputAndDecode(p.input);
    } else if (type === "decodeError") {
      const p = payload as { message: string };
      inspector.setError(p.message);
    } else if (type === "executeResult") {
      activeTab = "playground";
      playground.setResult(payload as never);
    } else if (type === "executeError") {
      const p = payload as { message: string };
      activeTab = "playground";
      playground.setError(p.message);
    } else if (type === "themeChanged") {
      const p = payload as { theme: "light" | "dark" | "high-contrast" };
      document.body.dataset.theme = p.theme;
    }
  });

  renderTab();
  bridge.send("ready", {});
}
