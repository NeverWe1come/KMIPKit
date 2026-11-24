import type { IdeBridge } from "../../bridge/ideBridge";

export interface PlaygroundProfile {
  name: string;
  environment: "DEV" | "STAGING" | "PROD";
  host: string;
  port: number;
  readOnly: boolean;
  tls?: {
    caCertPath?: string;
    clientCertPath?: string;
    clientKeyPath?: string;
  };
}

export interface PlaygroundResult {
  operation: string;
  environment: string;
  latencyMs: number;
  resultStatus: string;
  summary: string;
  supportedVersions: string[];
  rawResponseHex?: string;
}

const MUTATING_OPS = new Set(["Destroy", "Rekey", "Revoke", "Create"]);

export class PlaygroundView {
  private profile: PlaygroundProfile = {
    name: "local-dev",
    environment: "DEV",
    host: "127.0.0.1",
    port: 5696,
    readOnly: false
  };
  private operation = "DiscoverVersions";
  private uniqueId = "";
  private unlockProd = false;
  private lastResult: PlaygroundResult | null = null;
  private errorMessage: string | null = null;

  constructor(
    private readonly container: HTMLElement,
    private readonly bridge: IdeBridge
  ) {}

  setResult(result: PlaygroundResult): void {
    this.lastResult = result;
    this.errorMessage = null;
    this.render();
  }

  setError(message: string): void {
    this.errorMessage = message;
    this.render();
  }

  render(): void {
    this.container.innerHTML = "";

    const card = document.createElement("div");
    card.className = "kmip-card";
    const title = document.createElement("h3");
    title.textContent = "KMIP Live Playground (mTLS + Environment Guardrails)";
    card.appendChild(title);

    const form = document.createElement("div");
    form.style.display = "grid";
    form.style.gridTemplateColumns = "1fr 1fr";
    form.style.gap = "8px";
    form.style.marginBottom = "10px";

    form.innerHTML = `
      <label>Environment:
        <select id="pg-env" class="kmip-input" style="min-height:auto;padding:4px;">
          <option value="DEV" ${this.profile.environment === "DEV" ? "selected" : ""}>DEV</option>
          <option value="STAGING" ${this.profile.environment === "STAGING" ? "selected" : ""}>STAGING</option>
          <option value="PROD" ${this.profile.environment === "PROD" ? "selected" : ""}>PROD (Protected)</option>
        </select>
      </label>
      <label>Operation:
        <select id="pg-op" class="kmip-input" style="min-height:auto;padding:4px;">
          <option value="DiscoverVersions" ${this.operation === "DiscoverVersions" ? "selected" : ""}>DiscoverVersions</option>
          <option value="Ping" ${this.operation === "Ping" ? "selected" : ""}>Ping</option>
          <option value="Query" ${this.operation === "Query" ? "selected" : ""}>Query</option>
          <option value="Locate" ${this.operation === "Locate" ? "selected" : ""}>Locate</option>
          <option value="GetAttributes" ${this.operation === "GetAttributes" ? "selected" : ""}>GetAttributes</option>
          <option value="Get" ${this.operation === "Get" ? "selected" : ""}>Get</option>
          <option value="Destroy" ${this.operation === "Destroy" ? "selected" : ""}>Destroy (Destructive)</option>
        </select>
      </label>
      <label>Host:
        <input id="pg-host" class="kmip-input" style="min-height:auto;padding:4px;" value="${this.profile.host}" />
      </label>
      <label>Port:
        <input id="pg-port" type="number" class="kmip-input" style="min-height:auto;padding:4px;" value="${this.profile.port}" />
      </label>
      <label>UniqueIdentifier (for Get / GetAttributes / Destroy):
        <input id="pg-uid" class="kmip-input" style="min-height:auto;padding:4px;" value="${this.uniqueId}" placeholder="optional" />
      </label>
      <label style="display:flex;align-items:center;gap:6px;">
        <input id="pg-ro" type="checkbox" ${this.profile.readOnly ? "checked" : ""} /> Read-Only Mode
      </label>
    `;
    card.appendChild(form);

    if (this.profile.environment === "PROD" && MUTATING_OPS.has(this.operation)) {
      const warn = document.createElement("div");
      warn.className = "alert-banner";
      warn.innerHTML = `
        <strong>PROD Guardrail Active:</strong> Destructive/mutating operation <code>${this.operation}</code> is blocked by default.
        <label style="margin-left:8px;"><input id="pg-unlock-prod" type="checkbox" ${this.unlockProd ? "checked" : ""} /> Unlock PROD execution</label>
      `;
      card.appendChild(warn);
      warn.querySelector<HTMLInputElement>("#pg-unlock-prod")?.addEventListener("change", (e) => {
        this.unlockProd = (e.target as HTMLInputElement).checked;
      });
    }

    const execBtn = document.createElement("button");
    execBtn.className = "kmip-btn";
    execBtn.textContent = `Execute ${this.operation}`;
    execBtn.addEventListener("click", () => {
      this.syncInputs(card);
      this.bridge.send("executeRequest", {
        profile: this.profile,
        operation: this.operation,
        uniqueIdentifier: this.uniqueId.trim() || undefined,
        unlockProd: this.unlockProd
      });
    });
    card.appendChild(execBtn);

    if (this.errorMessage) {
      const err = document.createElement("div");
      err.className = "error-banner";
      err.style.marginTop = "10px";
      err.textContent = this.errorMessage;
      card.appendChild(err);
    }

    if (this.lastResult) {
      const resBox = document.createElement("div");
      resBox.style.marginTop = "10px";
      resBox.innerHTML = `
        <div class="alert-banner" style="border-color: var(--kmip-accent); background: rgba(14,99,156,0.12);">
          <strong>${this.lastResult.operation}</strong> · Status: <code>${this.lastResult.resultStatus}</code> · Latency: <strong>${this.lastResult.latencyMs} ms</strong><br/>
          ${this.lastResult.summary}
        </div>
      `;
      card.appendChild(resBox);
    }

    this.container.appendChild(card);

    card.querySelector<HTMLSelectElement>("#pg-env")?.addEventListener("change", () => {
      this.syncInputs(card);
      this.unlockProd = false;
      this.render();
    });
    card.querySelector<HTMLSelectElement>("#pg-op")?.addEventListener("change", () => {
      this.syncInputs(card);
      this.render();
    });
  }

  private syncInputs(card: HTMLElement): void {
    this.profile.environment = (card.querySelector<HTMLSelectElement>("#pg-env")?.value as PlaygroundProfile["environment"]) ?? "DEV";
    this.operation = card.querySelector<HTMLSelectElement>("#pg-op")?.value ?? "DiscoverVersions";
    this.profile.host = card.querySelector<HTMLInputElement>("#pg-host")?.value ?? "127.0.0.1";
    this.profile.port = Number(card.querySelector<HTMLInputElement>("#pg-port")?.value ?? "5696");
    this.uniqueId = card.querySelector<HTMLInputElement>("#pg-uid")?.value ?? "";
    this.profile.readOnly = card.querySelector<HTMLInputElement>("#pg-ro")?.checked ?? false;
  }
}
