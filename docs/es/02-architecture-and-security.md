# 02. Arquitectura y Modelo de Seguridad

KMIPKit está diseñado para entornos empresariales regulados (banca, HSMs de pagos, nubes soberanas y gestión de claves de alta seguridad). Cada límite de crate, regla de propiedad de memoria e invariante de transporte se verifica en tiempo de compilación y mediante puertas de seguridad automatizadas.

---

## 1. Arquitectura Modular por Capas

El repositorio se divide en **dos workspaces de Cargo estrictamente aislados**:

```text
KMIPKit/
├── crates/                     # Workspace Cliente Principal (8 crates)
│   ├── kmipkit-ttlv/           # Modelo TTLV genérico ordenado y códec acotado
│   ├── kmipkit-protocol/       # Catálogo normativo, modelos tipados, validación y version gating
│   ├── kmipkit-transport/      # Adaptadores Raw TLS y HTTPS/1.1 mTLS, buffers zeroizados
│   ├── kmipkit-client/         # Ejecución síncrona, lotes, resultados asíncronos y listeners S2C
│   ├── kmipkit/                # Fachada pública Rust y Fluent Builders
│   ├── kmipkit-ffi/            # ABI C11 estable (ÚNICO crate con `unsafe` permitido)
│   ├── kmipkit-test-support/   # PKI efímera, transportes simulados y fixtures (no publicado)
│   └── kmipkit-test-server/    # Servidor KMIP multi-versión independiente para pruebas
├── bindings/                   # Adaptadores Nativos por Lenguaje
│   ├── c/                      # Cabeceras C11, CMake, tests de consumidor ABI
│   ├── java/                   # Puente JNI C++ + API Java 17 idiomática
│   └── python/                 # Puente CFFI + API Python 3.12 tipada
└── kmip-proxy/                 # Workspace Independiente del Proxy
    ├── src/providers/          # 10 puentes Cloud / Enterprise KMS
    └── src/kmip1x/             # Puente Traductor KMIP 1.0-1.4 <-> KMIP 2.0/2.1/3.0
```

---

## 2. Tres Niveles de API Soportados

| Nivel de API | Propósito | Cuándo usarlo |
| :--- | :--- | :--- |
| **1. Alto Nivel (Fluent)** | Builders orientados a tareas (`create_symmetric_key()`, `encrypt()`, `activate()`) | 90% de integraciones de aplicaciones empresariales. |
| **2. Protocolo Tipado** | `ClientBatch`, `ClientBatchItem`, `ClientRequest` (59 operaciones), `validation_report()` | Lotes atómicos multi-operación, `ID Placeholder`, `Poll`/`Cancel`/`Process`. |
| **3. TTLV Genérico** | Árbol ordenado `Item` / `Structure` / `Value` (`kmipkit-ttlv`) | Inspección de tags de fabricante desconocidos, proxies y extensiones a medida. |

---

## 3. Invariantes de Seguridad en Transporte y Memoria

### Política TLS (`kmipkit-transport`)

| Control de Seguridad | Política por Defecto | Compatibilidad Opt-In |
| :--- | :--- | :--- |
| **Versión TLS** | **Solo TLS 1.3** (`TlsVersionPolicy::Tls13Only`) | `Tls12And13Compat` (restringido a suites ECDHE-GCM de OASIS KMIP Profiles) |
| **Certificados Post-Cuánticos** | **ML-DSA-44 / ML-DSA-65 / ML-DSA-87** vía OpenSSL 3.5 | — |
| **Autenticación** | **TLS Mutuo (mTLS) Obligatorio** | — |
| **Verificación de Certificados** | Cadena + Validez + Hostname/SAN + CRLs locales opcionales | Sin búsquedas OCSP/CRL en red; sin modo inseguro |
| **0-RTT / Key Logging** | **Deshabilitados permanentemente** | No se pueden activar |
| **Reanudación de Sesión** | Caché en memoria acotada (máx. 16 tickets, 1 hora de expiración) | Nunca persistida en disco |

### Propiedad de Memoria, Zeroización y Redacción
1. **Zeroización al Liberar:** Claves privadas, contraseñas, buffers de petición y `TransportResponse` usan `zeroize::Zeroizing`.
2. **Redacción Total en Diagnósticos:** Todos los `Debug`/`Display` de credenciales, endpoints y payloads TTLV emiten `[REDACTED]`.
3. **Decodificación Acotada (`CodecLimits`):** Límite estricto de bytes (16 MiB), profundidad (64) y elementos antes de reservar memoria.
4. **Cero `unsafe` fuera de FFI:** `#![forbid(unsafe_code)]` en todos los crates excepto `kmipkit-ffi`.

---

## 4. Garantías de Estado de Entrega (Cero Reintentos Automáticos)

| `RequestDeliveryState` | Significado Exacto | Acción Segura del Llamador |
| :--- | :--- | :--- |
| **`NotSent`** | Fallo local (validación, DNS, handshake TLS) antes de enviar ningún byte. | Seguro corregir o reintentar de inmediato. |
| **`PossiblySent`** | Se escribió al menos un byte en TLS, pero no llegó respuesta antes del timeout/reset. | Estado ambiguo: consultar estado en el servidor (`Locate`/`GetAttributes`) antes de reintentar. |
| **`ResponseStarted`** | Llegó al menos un byte de respuesta y falló el framing o la conexión. | El servidor procesó la petición; reconciliar estado antes de reintentar. |

---

## 5. Aislamiento de Handles FFI y Contención de Panics

* **Handles Opacos Permutados:** Los punteros exportados son tokens generados mediante una **permutación Feistel de 8 rondas con clave aleatoria HMAC-SHA-256 por proceso**, respaldados por una tabla de handles activos.
* **Contención Total de Panics:** Toda función `extern "C"` envuelve su ejecución en `catch_unwind` y devuelve `KMIPKIT_ERROR_INTERNAL (7)` sin filtrar datos sensibles.
