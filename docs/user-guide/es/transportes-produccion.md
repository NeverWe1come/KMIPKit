# Transportes de producción con TLS y HTTPS

El cliente síncrono de Rust de KMIPKit puede usar TTLV sobre TLS directo o
TTLV sobre HTTPS/HTTP 1.1. El cliente envía únicamente la operación solicitada
por la aplicación; al construirlo no resuelve el endpoint, no se conecta ni
envía un Discover Versions oculto. El alcance de protocolo de 1.0 es KMIP 2.1
con codificación TTLV.

## Elegir un endpoint

Usa `Endpoint::raw_tls(host, port)` cuando el servidor KMIP acepte una trama
TTLV directamente sobre TLS. Una conexión TLS directa transporta una petición
y una respuesta y después se cierra. Usa `Endpoint::https("https://host:port")`
cuando el servidor ofrezca el enlace HTTPS de KMIP. HTTPS usa HTTP/1.1, emplea
`/kmip` como ruta predeterminada y puede reutilizar una conexión sana para
intercambios posteriores del mismo cliente. `target_uri` permite elegir otra
ruta y consulta en formato origin-form. No se siguen redirecciones.

## Configurar TLS mutuo y la confianza

El llamador debe proporcionar un certificado de cliente y una clave privada,
y elegir explícitamente cómo confiar en el servidor. La confianza puede usar
certificados CA proporcionados por el llamador o el almacén de certificados de
la plataforma. El ejemplo usa archivos PEM y una CA propia; la misma API
acepta DER y permite seleccionar explícitamente la confianza de la plataforma.

```rust,no_run
use std::time::Duration;

use kmipkit_client::extension_registry::{ClientConfiguration, client_extension_registry};
use kmipkit_client::{Client, ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::extension;
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, TimeoutLimit, TimeoutPolicy,
    TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let transport = TransportConfig::builder(Endpoint::https("https://kmip.example:5696"))
        .client_identity(ClientIdentity::new(
            CertificateInput::from_pem_file("client-chain.pem")?,
            PrivateKeyInput::from_pem_file("client-key.pem")?,
        ))
        .trust_source(TrustSource::certificate_authorities(vec![
            CertificateInput::from_pem_file("server-ca.pem")?,
        ]))
        .timeouts(TimeoutPolicy::default().with_total(TimeoutLimit::Bounded(
            Duration::from_secs(60),
        )))
        .build()?;

    let registry = client_extension_registry(Vec::new(), extension::defaults())?;
    let mut client = Client::new(ClientConfiguration::new(registry), transport)?;
    let request = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
    let response = client.execute(request, &CodecLimits::defaults())?;
    assert_eq!(response.len(), 1);
    Ok(())
}
```

Los nombres de archivo son rutas administradas por la aplicación. KMIPKit lee
cada archivo seleccionado una sola vez al construir la entrada y no conserva
la ruta. Las claves privadas deben estar sin cifrar y usar una codificación
PEM o DER compatible. Limita la lectura de los archivos de clave a la cuenta
del servicio. También hay entradas en memoria para aplicaciones que obtengan
credenciales desde otra fuente segura.

TLS queda fijado a TLS 1.3 con autenticación mutua, rustls y AWS-LC. Se
comprueban la cadena del certificado del servidor, su periodo de validez y el
nombre del endpoint. No existe una opción de producción para aceptar cualquier
certificado. Se permite cambiar el nombre TLS en endpoints IP, sin desactivar
la validación del certificado. Las CRL opcionales se proporcionan localmente;
no se descargan. Están desactivados los datos anticipados, las redirecciones,
los proxies, la compresión HTTP y el registro de claves TLS.

`TrustSource::platform()` selecciona las raíces nativas de la plataforma. En
ese modo, si `SSL_CERT_FILE` está definido, el cargador de la plataforma usa
ese paquete seleccionado. La confianza de plataforma no importa todas las
decisiones del sistema operativo sobre desconfianza o revocación. Usa una CA
proporcionada explícitamente si el servidor tiene una PKI privada o si la
política de la aplicación exige un conjunto de CA concreto.

Los tickets de sesión TLS pertenecen a una configuración de cliente, están
limitados a 16 y caducan localmente al cabo de una hora. Una sesión reanudada
hereda la identidad del par y la decisión de confianza/CRL de su handshake
completo validado. Reconstruye el cliente después de cambiar la confianza o la
identidad para empezar con una caché de tickets vacía.

## Plazos y estado de entrega

Los plazos predeterminados son 10 segundos para conectar, 30 segundos sin
progreso de escritura, 30 segundos sin progreso de lectura y 60 segundos para
el intercambio completo. `TimeoutPolicy` define los valores predeterminados
del cliente. `RequestOptions` puede reemplazar fases individuales en una
llamada; las fases sin reemplazo mantienen su valor configurado. Una duración
acotada de cero significa vencimiento inmediato. Usa
`TimeoutLimit::Unbounded` solo cuando la aplicación quiera intencionadamente
no limitar esa fase.

Para una operación que pueda tardar más, aplica un reemplazo solo a esa
llamada. El estado de entrega ayuda a la aplicación a decidir si debe
reconciliar el estado del servidor; no recomienda reintentar.

```rust,no_run
use std::time::Duration;

use kmipkit_client::{Client, ClientBatch, ClientError};
use kmipkit_transport::{RequestDeliveryState, RequestOptions, TimeoutLimit};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_with_deadline(
    client: &mut Client,
    request: ClientBatch,
) -> Result<(), ClientError> {
    let limits = CodecLimits::defaults();
    let options = RequestOptions::default()
        .with_total(TimeoutLimit::Bounded(Duration::from_secs(90)));

    match client.execute_with_options(request, &limits, &options) {
        Ok(_response) => Ok(()),
        Err(error) => {
            match error.delivery_state() {
                Some(RequestDeliveryState::NotSent) => {
                    // El transporte no observó bytes de petición en la red.
                }
                Some(
                    RequestDeliveryState::PossiblySent
                    | RequestDeliveryState::ResponseStarted,
                ) => {
                    // Reconcilia la operación antes de enviar otra petición.
                }
                None => {
                    // Este error no contiene estado local de entrega.
                }
                Some(_) => {
                    // Trata futuros estados como inciertos; no reintentes a ciegas.
                }
            }
            Err(error)
        }
    }
}
```

Cada fallo local informa del estado de entrega más preciso disponible:

- `NotSent`: no se enviaron bytes de la petición.
- `PossiblySent`: comenzó el envío, pero no se observó ningún byte de respuesta
  descifrado.
- `ResponseStarted`: se observó al menos un byte de respuesta, pero no se
  aceptó una respuesta completa y válida.

`PossiblySent` y `ResponseStarted` no demuestran que la operación no haya
tenido efecto. KMIPKit nunca reintenta una petición automáticamente. El
llamador debe decidir cómo reconciliar un resultado incierto según la semántica
de la operación y el estado de la aplicación. Una conexión fallida o cancelada
no se reutiliza.

El cliente tipado usa `CodecLimits` tanto para codificar peticiones como para
decodificar respuestas; el límite predeterminado del mensaje completo es 16
MiB. El transporte comprueba el límite de respuesta antes de entregar los
bytes al decodificador TTLV. El límite directo de petición del transporte
también es de 16 MiB por defecto. Estos límites locales no afirman que el
servidor vaya a aceptar la petición o respetar un límite de respuesta
solicitado.

## Errores, secretos e interoperabilidad

Los errores de configuración y transporte solo exponen categorías seguras
fijas, no bytes de credenciales, texto de claves privadas, mensajes de
dependencias, cuerpos KMIP ni valores de ruta/consulta configurados. Las
aplicaciones deberían registrar la categoría segura y el estado de entrega,
nunca credenciales ni cuerpos de mensajes. KMIPKit pone a cero los bytes
inicializados de las asignaciones que posee, incluidas las entradas de claves
privadas y los buffers de respuesta del transporte. Esto no borra copias del
llamador ni copias retenidas por las bibliotecas TLS, el sistema operativo o
runtimes externos.

KMIPKit transporta y administra material criptográfico; no realiza
operaciones criptográficas locales. El éxito de una operación depende de las
capacidades, configuración y política del servidor. El rechazo de un servidor
no demuestra por sí solo que el cliente tenga un defecto de protocolo.
Consulta la [arquitectura de seguridad del transporte](../../architecture/transport-security.md)
y la [guía de ejecución del cliente](ejecucion-cliente.md) para ver los
contratos detallados de la API y la entrega.

## Ejemplo local y ejecutable de una petición

Crear un lote tipado no se conecta a un servidor. Este ejemplo se ejecuta como
prueba de documentación de Rust:

```rust
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};

let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::discover_versions()));
assert_eq!(batch.items().len(), 1);
```
