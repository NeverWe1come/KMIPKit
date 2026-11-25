# Plan Maestro: Plugins Oficiales de IDEs para KMIPKit

> **Decisión de Arquitectura (`core/`):** El núcleo compartido `kmip-lsp` está implementado en **Rust (Edition 2024, `#![forbid(unsafe_code)]`)**, compilando directamente contra los *crates* oficiales `kmipkit-ttlv` y `kmipkit-protocol` (y `kmipkit-client` bajo `feature = "live-client"` en Fase 2). Esto produce un único binario nativo autocontenido por plataforma sin runtime externo.

## Estructura del Directorio `ide-plugins/`

- `PLAN.md` — Este documento de arquitectura y roadmap
- `README.md` — Guía rápida para contribuidores de plugins
- `VERSION` — Versión sincronizada de los plugins (`0.1.0`)
- `core/` — Servidor LSP + CLI/RPC (`kmip-lsp`) basado en `kmipkit-ttlv` y `kmipkit-protocol`
- `schemas/` — JSON Schemas oficiales (`kmip-config.schema.json`, `kmip-message.schema.json`, `kmip-policy.schema.json`)
- `webview-ui/` — UI web compartida (VS Code Webview + JetBrains JCEF)
- `vscode/` — Extensión oficial para VS Code, Cursor, Windsurf y VSCodium
- `jetbrains/` — Plugin unificado (`org.kmipkit.ide`) para toda la familia JetBrains
- `scripts/` — Scripts de automatización (`sync-versions.sh`, `build-webview.sh`, `verify-checksums.sh`)

## Hoja de Ruta

- **Fase 0 & Fase 1 (`v0.1.0`, actual):** Núcleo `kmip-lsp` (LSP + `kmip/decodeTtlv` + `kmip/encodeTtlv` + `kmip/lookupSpec` + redacción obligatoria), `schemas/`, `webview-ui/` (Inspector TTLV + vista hex sincronizada), `vscode/`, `jetbrains/` y CI multiplataforma.
- **Fase 2 (`v0.2.0`):** KMIP Explorer & Playground con cliente mTLS en vivo (`feature = "live-client"`), almacenamiento en `SecretStorage` / `PasswordSafe`, guardrails `DEV`/`STAGING`/`PROD` y adaptadores `neovim/` y `zed/`.
- **Fase 3 (`v0.3.0+`):** Diagnósticos profundos sobre el SDK en código fuente, quick-fixes de migración KMIP 1.4 → 2.1/3.0, generador de tests desde capturas TTLV anonimizadas y adaptador `visual-studio/`.
