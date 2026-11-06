# Interfaces tipadas, validación y datos de respuesta

KMIPKit ofrece vistas tipadas de estructuras KMIP comunes junto con acceso TTLV genérico. La vía genérica sigue disponible para campos repetidos, valores desconocidos por esta versión de la biblioteca y extensiones de fabricante. La existencia de una API no demuestra por sí sola que un servidor admita esa capacidad, que un perfil sea conforme ni que se haya demostrado interoperabilidad.

[English version](../en/typed-bindings-and-response-data.md).

## Atributos tipados

Usa la definición catalogada del atributo para construir o comprobar un atributo directo. Los wrappers tipados verifican el tag asignado y los tipos de valor TTLV permitidos; no eliminan la representación genérica.

| Lenguaje | Construir o comprobar un atributo | Añadir y leer ocurrencias en un Attribute Set |
| --- | --- | --- |
| Rust | <code>Attribute::new(value)</code> o <code>Attribute::from_item(item)</code> con un tipo marcador del catálogo | <code>AttributeSet::try_push_typed(attribute)</code> y <code>typed::&lt;Definition&gt;()</code> |
| C | <code>kmipkit_typed_attribute_from_value(...)</code> o <code>kmipkit_typed_attribute_from_item(...)</code> | <code>kmipkit_attribute_set_try_push_typed(...)</code>, <code>kmipkit_attribute_set_typed_count(...)</code> y <code>kmipkit_attribute_set_typed_at(...)</code> |
| Java | <code>TypedAttribute.create(...)</code> o <code>TypedAttribute.fromItem(...)</code> | <code>AttributeSet.tryPushTyped(...)</code>, <code>typedCount(...)</code>, <code>typedAt(...)</code> o <code>typed(...)</code> |
| Python | <code>TypedAttribute.create(...)</code> o <code>TypedAttribute.from_item(...)</code> | <code>AttributeSet.try_push_typed(...)</code>, <code>typed_count(...)</code>, <code>typed_at(...)</code> o <code>typed(...)</code> |

La inserción en Attribute Set conserva el orden de cable y las ocurrencias repetidas. No aplica cardinalidades específicas de una operación cuando el estándar las condiciona al contexto. Conserva los Items genéricos para atributos de fabricante y campos no reconocidos. Rust usa tipos marcadores del catálogo; C, Java y Python usan el catálogo generado Standard Attribute.

Consulta [Operaciones de atributos](attribute-operations.md) para ver las definiciones de atributos y su uso por operación.

## Objetos gestionados y estructuras de clave

Los modelos tipados cubren los nueve tipos de objeto gestionado estándar y las estructuras Key Block, Key Value, Key Material y Key Wrapping Data. Usa constructores y lectores tipados para los campos reconocidos, y conserva los Items TTLV genéricos para los campos que la aplicación no interprete.

| Lenguaje | Construir a partir de campos | Leer o inspeccionar campos |
| --- | --- | --- |
| Rust | Usa los constructores específicos de objeto y el builder <code>KeyBlock::try_new(...)</code>; <code>KeyMaterial::byte_string(...)</code> y <code>KeyMaterial::structure(...)</code> cubren las formas de material de clave. | <code>ManagedObject::try_from_item(...)</code>, <code>KeyBlock::try_from_item(...)</code>, <code>KeyValue::try_from_item(...)</code>, <code>KeyMaterial::try_from_item(...)</code> y <code>KeyWrappingData::try_from_item(...)</code>; usa <code>ManagedValueReader</code> para lecturas por tag. |
| C | <code>kmipkit_managed_object_from_fields(...)</code>, <code>kmipkit_key_block_from_fields(...)</code>, <code>kmipkit_key_value_from_fields(...)</code>, <code>kmipkit_key_material_from_fields(...)</code> y <code>kmipkit_key_wrapping_data_from_fields(...)</code>; también hay constructores específicos para los objetos. | Los lectores correspondientes <code>..._from_item(...)</code> y <code>kmipkit_managed_value_field_read(...)</code> / <code>kmipkit_managed_value_field_item_read(...)</code>. |
| Java | <code>ManagedObject.fromFields(...)</code>, helpers específicos como <code>ManagedObject.symmetricKey(...)</code>, y <code>KeyBlock.fromFields(...)</code>, <code>KeyValue.fromFields(...)</code>, <code>KeyMaterial.fromFields(...)</code>, <code>KeyWrappingData.fromFields(...)</code>. | Los lectores correspondientes <code>fromItem(...)</code>, lectores escalares con nombre y <code>fieldItem(rawTag)</code>. |
| Python | <code>ManagedObject.from_fields(...)</code>, helpers específicos como <code>ManagedObject.symmetric_key(...)</code>, y <code>KeyBlock.from_fields(...)</code>, <code>KeyValue.from_fields(...)</code>, <code>KeyMaterial.from_fields(...)</code>, <code>KeyWrappingData.from_fields(...)</code>. | Los lectores correspondientes <code>from_item(...)</code>, lectores escalares con nombre y <code>field_item(raw_tag)</code>. |

Los constructores externos <code>fromFields</code> / <code>from_fields</code> copian los Items de campo proporcionados por quien llama y ordenan los campos reconocidos según la tabla KMIP, independientemente del orden de entrada. Los lectores tipados validan los campos conocidos y conservan el árbol TTLV original, incluidos los hijos desconocidos, su orden y los valores repetidos. Los builders de Rust también emiten los campos definidos en el orden especificado. Un Key Value envuelto se representa como bytes envueltos; no se presenta como Key Material en claro.

Los accesores de campos de valores gestionados leen un hijo directo mediante su tag bruto. Para inspeccionar un descendiente más profundo, continúa navegando por la Structure o la vista TTLV devuelta. Así la aplicación puede alcanzar nodos con tags de fabricante sin que KMIPKit tenga que interpretar su semántica. Consulta [Objetos gestionados y material de clave](objetos-gestionados-y-material-de-clave.md) para conocer los modelos.

## Propiedad y datos secretos

Respeta las reglas de vida útil de cada binding:

- En Rust, los valores modelo propios se liberan al salir de su ámbito; las vistas prestadas de campos o Structures no pueden sobrevivir a su propietario o callback.
- En C, los constructores devuelven handles que quien llama debe liberar con la función correspondiente. Los punteros prestados, como P Source en el registro de campos de Cryptographic Parameters, solo son válidos mientras viva el handle propietario.
- En Java, los propietarios respaldados por código nativo implementan <code>AutoCloseable</code>; cierra cada propietario y cada Item o Structure hijo con propiedad independiente cuando termines. Los arrays de bytes devueltos a Java son copias propiedad de la aplicación.
- En Python, los propietarios nativos ofrecen <code>close()</code> y soporte de context manager; cierra los Items, Structures y wrappers tipados hijos que se devuelvan. Los objetos bytes de Python son copias inmutables propiedad de la aplicación.

KMIPKit limpia los buffers secretos que posee cuando se libera el objeto propietario. Las copias devueltas a Java o Python, o copiadas posteriormente por el código de la aplicación, quedan fuera de la garantía de borrado determinista de KMIPKit. Los mensajes de error y las representaciones de depuración/cadena evitan imprimir valores secretos o mensajes KMIP completos.

## Cryptographic Parameters

Cryptographic Parameters conserva los valores brutos de Enumeration, incluso aquellos que esta versión de la biblioteca no conoce. Los builders emiten los miembros conocidos en el orden de KMIP 2.1 Tabla 59. Los accesores distinguen entre un campo ausente y uno presente con valor cero o <code>false</code>. Los campos modelados son Block Cipher Mode, Padding Method, Hashing Algorithm, Key Role Type, Digital Signature Algorithm, Cryptographic Algorithm, Random IV, IV Length, Tag Length, Fixed Field Length, Invocation Field Length, Counter Length, Initial Counter Value, Salt Length, Mask Generator, Mask Generator Hashing Algorithm, P Source y Trailer Field.

| Lenguaje | Construir o analizar | Leer campos | Liberar |
| --- | --- | --- | --- |
| Rust | <code>CryptographicParametersBuilder::new()</code> o <code>CryptographicParameters::try_from_ttlv(...)</code> | <code>block_cipher_mode()</code>, <code>random_iv()</code>, <code>p_source()</code> y los demás getters de Tabla 59 | Liberar el valor propietario |
| C | Rellena <code>kmipkit_cryptographic_parameters_fields_v1_t</code> y llama a <code>kmipkit_cryptographic_parameters_from_fields_v1</code>, o llama a <code>kmipkit_cryptographic_parameters_from_ttlv</code> | <code>kmipkit_cryptographic_parameters_read_fields_v1</code> rellena un registro v1 propiedad de quien llama | <code>kmipkit_cryptographic_parameters_release</code> |
| Java | <code>CryptographicParameters.builder()</code> o <code>fromTtlv(...)</code> | <code>blockCipherMode()</code>, <code>randomIv()</code>, <code>pSource()</code> y los demás getters de Tabla 59 | <code>close()</code> |
| Python | <code>CryptographicParameters.builder()</code> o <code>from_ttlv(...)</code> | <code>block_cipher_mode()</code>, <code>random_iv()</code>, <code>p_source()</code> y los demás getters de Tabla 59 | <code>close()</code> o un bloque <code>with</code> |

El puntero P Source del registro C está prestado desde el handle vivo de Cryptographic Parameters. Java y Python devuelven copias de los bytes P Source, mientras que Rust los presta solo durante el callback del accessor. Las copias realizadas por la aplicación siguen las reglas de propiedad y borrado descritas arriba.

## Authentication del Request Header y parámetros del lote

KMIP Authentication es distinta de la autenticación TLS del cliente. Se adjunta a un mensaje de petición en el Request Header y se aplica a sus Batch Items. No se conserva como configuración global de Client.

| Lenguaje | Crear Authentication | Adjuntar a un lote |
| --- | --- | --- |
| Rust | <code>Authentication::new(credentials)</code> | <code>ClientBatch::with_authentication(authentication)</code> |
| C | <code>kmipkit_authentication_create(...)</code> | <code>kmipkit_client_batch_set_authentication(batch, authentication)</code> |
| Java | <code>Authentication.create(credentials)</code> | <code>ClientBatch.setAuthentication(authentication)</code> |
| Python | <code>Authentication.create(credentials)</code> | <code>Batch.set_authentication(authentication)</code> |

Si el adjunto tiene éxito, el lote adquiere la propiedad del valor Authentication. Si la llamada se rechaza, la propiedad sigue siendo de quien llama. Cierra o libera los valores según las reglas del binding. KMIPKit no presupone que el servidor acepte una credencial ni la reutiliza silenciosamente en mensajes posteriores.

Cryptographic Parameters y otros parámetros específicos pertenecen a la estructura de petición de la operación correspondiente. Configúralos en cada petición; no los trates como opciones de conexión. Para TLS, consulta [Transportes de producción](transportes-produccion.md).

## Validar antes del envío

Un informe de validación contiene metadatos seguros: código, ruta, gravedad y referencia normativa. Omite los valores de campo rechazados, las credenciales y el cuerpo de la petición.

| Lenguaje | Validar una petición | Validar una Structure genérica |
| --- | --- | --- |
| Rust | <code>ClientRequest::validation_report()</code> | <code>validation::validate_structure(structure, schema)</code> |
| C | <code>kmipkit_client_batch_item_validation_report(...)</code> | <code>kmipkit_ttlv_structure_validation_report_v1(...)</code> |
| Java | <code>ValidationReport.validateRequest(item)</code> | <code>ValidationReport.validateStructure(structure, schema)</code> |
| Python | <code>validation.validate_request(item)</code> | <code>validation.validate_structure(structure, schema=...)</code> |

En una Structure genérica, omite el esquema solo si no se conoce el esquema de la operación. KMIPKit no deduce la operación a partir de los valores de los campos. Si se proporciona una identidad de esquema compatible, se realizan las comprobaciones estructurales correspondientes.

La ejecución del cliente aplica las comprobaciones compartidas antes de escribir. Una infracción incondicional que pueda demostrarse localmente falla con estado de entrega <code>NotSent</code> y escribe cero bytes de petición. Las comprobaciones que dependen de capacidades del servidor, autorización, políticas o estado actual del objeto siguen correspondiendo al servidor. <code>NotSent</code> describe el estado de entrega local; no significa que un servidor haya evaluado la petición.

## Respuestas tipadas y acceso TTLV decodificado

Usa primero el lector de respuesta específico de la operación cuando esté disponible y después la vista del árbol decodificado genérico para los campos que no cubra ese modelo. El Response Payload puede estar ausente o presente pero vacío; se conserva la diferencia. El Response Header se puede leer de forma independiente.

| Lenguaje | Response Payload | Response Header | Vida útil |
| --- | --- | --- | --- |
| Rust | <code>ClientResponseView::raw()</code> devuelve una <code>StructureView</code> prestada opcional | <code>response_header_raw()</code> devuelve una vista prestada | Las vistas dependen del propietario de respuesta |
| C | <code>kmipkit_client_batch_item_response_raw_ttlv(...)</code> devuelve un indicador de presencia y un handle Structure propietario opcional | <code>kmipkit_client_batch_item_response_header_ttlv(...)</code> | Libera los handles con <code>kmipkit_ttlv_structure_release</code> |
| Java | <code>ClientBatchItemResult.raw()</code> devuelve <code>Optional&lt;TtlvStructure&gt;</code> | <code>responseHeaderTtlv()</code> devuelve <code>TtlvStructure</code> | Las Structures devueltas pertenecen a quien llama; ciérralas |
| Python | <code>BatchItemResult.raw()</code> devuelve <code>Structure</code> o <code>None</code> | <code>response_header_ttlv()</code> devuelve <code>Structure</code> | Las Structures devueltas pertenecen a quien llama; ciérralas |

Los accesores exponen el árbol TTLV decodificado y conservan el orden de los Items, las repeticiones, los tipos, los tags desconocidos y los tags anidados de fabricante, como <code>0x54xxxx</code>. <code>fieldItem(rawTag)</code> / <code>field_item(raw_tag)</code> en un wrapper de objeto gestionado o clave devuelve un Item hijo propietario con todo su subárbol. Continúa navegándolo mediante las APIs TTLV genéricas para inspeccionar descendientes arbitrarios, incluidos campos dentro de Key Block o Server Information. Las vistas Rust toman prestada la respuesta; C, Java y Python devuelven handles o estructuras propietarias que quien llama debe liberar o cerrar.

El decoder no conserva los bytes de padding; por eso <code>raw()</code> es una vista o copia del árbol TTLV decodificado, no una reproducción byte por byte de la trama de red. No registres los cuerpos de respuesta ni los incluyas en errores.

### Campos tipados de la respuesta Query

Los cuatro bindings exponen campos clasificados por Rust para la respuesta Query estándar de KMIP 2.1. Java, Python y C leen la clasificación que proporciona Rust, en vez de inferir los tipos por su cuenta.

| Lenguaje | Entrada tipada | Acceso tipado habitual | Acceso TTLV genérico |
| --- | --- | --- | --- |
| Rust | <code>ClientResponseView::query()</code> devuelve un <code>QueryResponse</code> opcional | <code>operations()</code>, <code>object_types()</code>, <code>response_fields()</code>, <code>unknown_items()</code> | <code>to_ttlv_payload()</code> reconstruye el payload genérico ordenado |
| C | <code>kmipkit_client_batch_item_response_query_field_count(...)</code> y <code>kmipkit_client_batch_item_response_query_field_at(...)</code> | Lee el tag, el tipo de campo y el valor Enumeration bruto; los demás resultados devuelven un Item TTLV propietario | <code>kmipkit_client_batch_item_response_query_ttlv(...)</code> |
| Java | <code>ClientBatchItemResult.typedQueryResponse()</code> devuelve un <code>QueryResponse</code> opcional | <code>operations()</code>, <code>applicationNamespaces()</code>, <code>fieldAt(index)</code>, <code>unknownItems()</code> | <code>rawView()</code>; <code>ClientBatchItemResult.queryResponse()</code> devuelve la Structure propietaria del payload |
| Python | <code>BatchItemResult.typed_query_response()</code> devuelve <code>QueryResponse</code> o <code>None</code> | <code>operations</code>, <code>application_namespaces</code>, <code>field_at(index)</code>, <code>unknown_items</code> | <code>raw_view()</code>; <code>BatchItemResult.raw()</code> devuelve la Structure genérica del payload |

La salida de tipo de campo de C usa <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_ENUMERATION</code>, <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_TEXT_STRING</code>, <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_STRUCTURE</code> o <code>KMIPKIT_QUERY_RESPONSE_FIELD_KIND_UNKNOWN</code>. Los valores Enumeration usan la salida de valor bruto; los demás tipos devuelven un Item propietario que quien llama debe liberar.

Las vistas de campos Query exponen el tag bruto y el tipo de campo elegido por Rust. El acceso a Enumeration conserva el valor sin signo, incluidos los valores añadidos después de esta versión de KMIPKit. Los accesores Java y Python que devuelven colecciones exponen las ocurrencias coincidentes en el orden recibido; sus accesores opcionales singulares muestran como máximo su primer campo coincidente. Usa <code>fieldAt(index)</code> / <code>field_at(index)</code> o el acceso indexado de C para conservar el orden intercalado completo, incluidos los campos repetidos; un campo desconocido sigue disponible como su Item TTLV genérico original. En C, libera los Items devueltos con <code>kmipkit_ttlv_item_release</code>. Las vistas independientes Java <code>rawView()</code> y Python <code>raw_view()</code> conservan el payload decodificado completo y los campos de fabricante anidados. Cierra <code>QueryResponse</code> y las vistas nativas devueltas por Java; en Python usa el context manager o <code>close()</code> y cierra también las vistas devueltas.

Rust expone los mismos miembros clasificados mediante <code>QueryResponseField</code>, accesores tipados y <code>unknown_items()</code>. Su <code>to_ttlv_payload()</code> reconstruye la secuencia de campos y conserva los Items desconocidos y las Structures anidadas. El ejemplo de ejecución Query en [Operaciones Query y Ping](operaciones-query-ping.md) muestra cómo obtener una respuesta Rust; su equivalente en inglés es [Query and Ping](../en/query-ping.md). Estas descripciones de API no demuestran compatibilidad del servidor ni interoperabilidad.

La API de alto nivel no expone el intercambio de bytes del transporte. Los resultados asíncronos siguen siendo explícitos y KMIPKit no consulta ni reintenta una petición automáticamente. Consulta [Ejecución del cliente](ejecucion-cliente.md) para ver las llamadas de operaciones y los resultados de entrega.
