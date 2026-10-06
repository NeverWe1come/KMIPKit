# Inspeccionar mensajes KMIP

El crate de protocolo puede validar e inspeccionar un mensaje Request o
Response ya decodificado antes de que lo procese otra capa. El modelo conserva
el árbol TTLV completo y ordenado, incluidos los valores desconocidos y las
extensiones de fabricantes.

## Analizar e inspeccionar

Empieza con una `Structure` TTLV genérica obtenida del decodificador acotado.
Transfiere su propiedad a `RequestMessage::try_from_ttlv` o
`ResponseMessage::try_from_ttlv`. Si el sobre, la cabecera, los elementos del
lote o las extensiones no tienen la forma requerida, el constructor devuelve
una categoría de error segura y su posición numérica.

Las peticiones y respuestas ofrecen vistas tipadas de los campos comunes de
cabecera y de cada elemento del lote. Sus accesores permiten consultar la
versión, el número de elementos, las opciones sin normalizar, las marcas de
tiempo, los identificadores de elemento, el estado del resultado y los valores
de correlación. Los datos anidados de Authentication, Nonce, los payloads y
las extensiones de fabricante se prestan mediante callbacks; inspecciona o
copia solo lo que necesite tu aplicación. Los errores y el formato por defecto
no incluyen el contenido de los payloads.

La documentación rustdoc del crate incluye un ejemplo compilado que analiza
una petición, comprueba su versión y número de elementos y recupera el árbol
original mediante `into_ttlv()`.

## Valores predeterminados y resultados asíncronos

Los campos opcionales conservan si estaban presentes. Si faltan, sus
accesores también exponen el valor efectivo de KMIP: Asynchronous Indicator es
Prohibited, Batch Error Continuation Option es Stop, Batch Order Option es
True y Attestation Capable Indicator es False. Los valores Enumeration se
conservan exactamente, aunque esta biblioteca no les asigne un nombre.

Una respuesta puede mezclar elementos completados y Pending en el mismo lote.
Cada resultado Pending debe incluir Asynchronous Correlation Value; el modelo
conserva sus bytes para una operación explícita posterior. Esta capa no hace
Poll, Cancel, espera ni reintentos automáticos.

## Alcance

La validación cubre la estructura común del mensaje y del lote. No valida los
payloads específicos de cada operación, no elige parámetros criptográficos,
no codifica bytes TTLV ni se conecta a un servidor KMIP. La base actual de
ejecución tipada y sus límites se describen en la
[guía de ejecución del cliente](ejecucion-cliente.md). No dispone de
constructor de producción ni backend de red activo.
Consulta la [referencia de arquitectura](../../architecture/public-api.md#kmip-message-model)
para conocer el límite de la API pública.
