# Operaciones de ciclo de vida con Java y Python

Java y Python permiten activar, archivar, localizar, recuperar y revocar objetos
mediante `client.request()` y métodos directos del cliente. En ambos casos la
biblioteca construye la petición tipada existente, realiza un único intercambio
y devuelve la respuesta tipada habitual.

Las opciones `RequestOptions` solo sustituyen los tiempos límite del
intercambio. El indicador asíncrono del Request Header se configura por separado
en el builder. Ninguna de estas operaciones reintenta, ejecuta Poll ni deduce el
estado remoto del objeto.

## Activate, Archive y Recover

```java
UniqueIdentifier id = UniqueIdentifier.textString("registro-42");
try (ClientBatchResponse response = client.request().activate(id).execute(limits)) {
    // Consulta el resultado tipado: Success, Pending o Failure.
}
```

```python
identifier = extensions.UniqueIdentifier.text_string("registro-42")
result = client.request().activate(identifier).execute(limits)
try:
    # Consulta result.items[0] y la respuesta tipada de la operación.
    pass
finally:
    result.close()
```

Las mismas formas de builder están disponibles para `archive` y `recover`.
También existen los métodos directos `activateById`, `archiveById` y
`recoverById` en Java, y `activate_by_id`, `archive_by_id` y `recover_by_id` en
Python. Cada método `*_by_id` requiere un identificador tipado explícito.

## Locate

Locate requiere un `AttributeSet` tipado. Los filtros de paginación, estado de
almacenamiento y pertenencia a un grupo son opcionales; KMIPKit no agrega
valores implícitos.

```java
try (ClientBatchResponse response = client.request().locate(attributes)
        .maximumItems(25).execute(limits)) {
    // Lee los identificadores devueltos por el servidor.
}
```

```python
result = client.locate(attributes, limits, maximum_items=25)
try:
    # Lee result.items[0] y los identificadores que devuelve Locate.
    pass
finally:
    result.close()
```

## Revoke

Revoke exige un identificador y un `RevocationReason` explícitos. La fecha de
compromiso es opcional y se proporciona por separado. Los mensajes del motivo
se ocultan en la representación del builder. Java y Python pueden conservar
copias de texto mientras el runtime las mantenga; KMIPKit limpia los buffers
temporales que codifica y posee.

```java
try (RevocationReason reason = RevocationReason.create(1, "política de rotación");
        ClientBatchResponse response = client.request().revoke(id, reason)
                .withOptions(RequestOptions.defaults()).execute(limits)) {
    // El resultado sigue siendo el resultado del servidor KMIP.
}
```

```python
with extensions.RevocationReason.create(1, "política de rotación") as reason:
    result = client.request().revoke(identifier, reason).execute(limits)
    try:
        pass
    finally:
        result.close()
```

Las funciones directas se llaman `revokeById` en Java y `revoke_by_id` en
Python. Las llamadas posteriores, como Poll o Get, son responsabilidad
explícita del programa.

Consulta la [guía inglesa de operaciones de ciclo de vida](../en/lifecycle-operations.md)
para los detalles KMIP de las operaciones y sus resultados.
