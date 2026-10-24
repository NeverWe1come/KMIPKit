# Sesiones de credenciales

Esta guía cubre las operaciones de cliente KMIP 2.1 Delegated Login, Login y
Logout: OASIS KMIP Specification v2.1 §§6.1.12, 6.1.30 y 6.1.31. Estas
operaciones intercambian tickets de sesión controlados por el llamador con el
servidor. KMIPKit no instala un Ticket como estado implícito del cliente, no lo
añade a peticiones posteriores ni ejecuta Logout automáticamente.

## Login

Todos los campos de la petición Login son opcionales. El llamador puede enviar
una petición vacía o elegir explícitamente Lease Time, Request Count y Usage
Limits. Este ejemplo construye un lote tipado, pero no lo envía.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::LoginRequest;

fn main() {
    let request = LoginRequest::new().with_lease_time_interval(300);
    let batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::login(request)));
    assert_eq!(batch.items().len(), 1);
}
```

Una respuesta Login correcta devuelve un `Ticket`. Consérvalo como valor
explícito; KMIPKit nunca lo aplica a peticiones posteriores. El accessor de la
respuesta actual presta el Ticket, mientras que Logout consume su propiedad. Si
el llamador necesita un Ticket propio para Logout, puede crear deliberadamente
una copia mediante el accessor de callback que se muestra abajo. El ticket del
ejemplo es sintético y no debe usarse para autenticarse.

## Pasar un Ticket a Logout

Ticket Value contiene datos secretos. `Ticket` y la petición Logout lo ocultan
en `Debug`; accede al valor solo mediante el callback. La copia siguiente es
explícita y queda bajo el control del llamador. No registres ni guardes los
bytes del ejemplo.

```rust,kmipkit-test
use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
use kmipkit_protocol::{
    LoginRequest, LogoutRequest, SecretBytes, Ticket, TicketType, TicketValidationError,
};

fn owned_ticket(ticket: &Ticket) -> Result<Ticket, TicketValidationError> {
    let value = ticket.with_ticket_value(|bytes| bytes.to_vec());
    Ticket::new(ticket.ticket_type(), SecretBytes::new(value))
}

fn main() {
    let login_batch = ClientBatch::new(ClientBatchItem::new(
        ClientRequest::login(LoginRequest::new()),
    ));
    assert_eq!(login_batch.items().len(), 1);

    // Sustituto sintético de un Ticket devuelto por el servidor.
    let returned_ticket = Ticket::new(
        TicketType::LOGIN,
        SecretBytes::new(b"synthetic-ticket-value".to_vec()),
    )
    .expect("el Ticket sintético tiene una estructura válida");
    assert!(!format!("{returned_ticket:?}").contains("synthetic-ticket-value"));

    let logout_ticket = owned_ticket(&returned_ticket)
        .expect("el Ticket sigue siendo válido");
    let logout = LogoutRequest::new(logout_ticket);
    let logout_batch = ClientBatch::new(ClientBatchItem::new(ClientRequest::logout(logout)));
    assert_eq!(logout_batch.items().len(), 1);
}
```

Crear el segundo `Ticket` propio copia sus bytes bajo el control del llamador.
KMIPKit pone a cero sus propias asignaciones `SecretBytes` al liberarlas; los
runtimes y las copias creadas por la aplicación tienen sus propias limitaciones
de ciclo de vida de memoria. Logout envía exactamente el Ticket que recibe. Una
respuesta Logout correcta tiene un payload de operación vacío; Failure y
Pending conservan los metadatos de resultado KMIP. KMIPKit no reintenta, no
ejecuta Poll ni reutiliza el Ticket silenciosamente.

## Delegated Login

Delegated Login requiere una estructura Rights del llamador y devuelve un
Ticket si la operación tiene éxito. Lease Time, Request Count y Usage Limits
siguen siendo opcionales. Usa `DelegatedLoginRequest` cuando la aplicación tenga
una estructura de derechos definida para su servidor KMIP. La biblioteca
conserva esa estructura TTLV; no interpreta los derechos ni afirma que el
servidor los haya concedido.

Las entradas tipadas están disponibles en los cuatro lenguajes de 1.0:

| Lenguaje | Mapeo de la petición | Mapeo de la respuesta tipada |
| --- | --- | --- |
| Rust | `ClientRequest::delegated_login(DelegatedLoginRequest::new(rights))` | `DelegatedLoginResponse::ticket()` y los accesores de `Ticket` |
| C | `kmipkit_client_batch_item_delegated_login(...)` | `kmipkit_client_batch_item_response_delegated_login(...)` |
| Java | `ClientBatchItem.delegatedLogin(...)` | `ClientBatchItemResult.delegatedLoginResponse()` |
| Python | `client_batch_item_delegated_login(...)` | `BatchItemResult.delegated_login_response` |

Cada enlace también expone la estructura TTLV completa y ordenada del Ticket,
incluidos sus hijos desconocidos, y conserva los valores desconocidos de Ticket
Type. Trata el Ticket Value y la estructura TTLV genérica del Ticket como datos
secretos. El span de Ticket Value de C pertenece al elemento de respuesta y deja
de ser válido al liberarlo. `ticketValue()` de Java devuelve un `byte[]` del
llamador, que debe borrar; KMIPKit borra sus buffers Java internos y la copia
propia de la respuesta al cerrarla. `ticket_value` de Python devuelve `bytes`
inmutables, que el runtime de Python no puede borrar de forma determinista;
KMIPKit borra su buffer mutable y la copia TTLV genérica propia de la respuesta
al llamar a `close()`. Rust expone Ticket Value mediante un callback y lo posee
en un tipo secreto que se pone a cero. No registres un Ticket, su TTLV genérico
ni ninguna copia de Ticket Value.

## Alcance y limitaciones

Los ejemplos de código se compilan contra las API Rust de cliente y protocolo.
El manifiesto aprobado KMIPKIT-0043 y su registro de trazabilidad documentan los
mapeos y la paridad de Delegated Login entre lenguajes. Las pruebas del proyecto
cubren el fixture Login CN01 §2.48 fijado en el repositorio; las pruebas
derivadas de la fuente para los demás casos no son fixtures oficiales. Esta guía
no afirma interoperabilidad con servidores ni certificación formal.
