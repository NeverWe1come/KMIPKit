export type IdeTheme = "light" | "dark" | "high-contrast";

export interface IdeBridge {
  send(type: string, payload: unknown): void;
  onMessage(handler: (type: string, payload: unknown) => void): () => void;
  getTheme(): IdeTheme;
}

interface VsCodeApi {
  postMessage(message: unknown): void;
}

declare global {
  interface Window {
    acquireVsCodeApi?: () => VsCodeApi;
    __kmipJcefSend?: (jsonPayload: string) => void;
    __kmipReceiveMessage?: (type: string, payload: unknown) => void;
    __kmipTheme?: IdeTheme;
  }
}

export function createIdeBridge(): IdeBridge {
  if (typeof window !== "undefined" && typeof window.acquireVsCodeApi === "function") {
    const api = window.acquireVsCodeApi();
    return {
      send(type: string, payload: unknown) {
        api.postMessage({ type, payload });
      },
      onMessage(handler) {
        const listener = (event: MessageEvent) => {
          const data = event.data;
          if (data && typeof data.type === "string") {
            handler(data.type, data.payload);
          }
        };
        window.addEventListener("message", listener);
        return () => window.removeEventListener("message", listener);
      },
      getTheme(): IdeTheme {
        const cls = document.body.classList;
        if (cls.contains("vscode-high-contrast")) return "high-contrast";
        if (cls.contains("vscode-light")) return "light";
        return "dark";
      }
    };
  }

  // JetBrains JCEF bridge (via JBCefJSQuery + executeJavaScript)
  const handlers = new Set<(type: string, payload: unknown) => void>();
  if (typeof window !== "undefined") {
    window.__kmipReceiveMessage = (type: string, payload: unknown) => {
      for (const h of handlers) {
        h(type, payload);
      }
    };
  }

  return {
    send(type: string, payload: unknown) {
      if (typeof window !== "undefined" && typeof window.__kmipJcefSend === "function") {
        window.__kmipJcefSend(JSON.stringify({ type, payload }));
      }
    },
    onMessage(handler) {
      handlers.add(handler);
      return () => handlers.delete(handler);
    },
    getTheme(): IdeTheme {
      if (typeof window !== "undefined" && window.__kmipTheme) {
        return window.__kmipTheme;
      }
      if (typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: light)").matches) {
        return "light";
      }
      return "dark";
    }
  };
}
