KMIPKit Python API Reference
============================

Memory-safe **OASIS KMIP 2.0, 2.1, and 3.0** client for Python 3.10+, backed by the
authoritative KMIPKit Rust core via CFFI.

.. toctree::
   :maxdepth: 2
   :caption: Contents:

   api
   installation
   native-library
   testing

Quick Example
-------------

.. code-block:: python

   from kmipkit import Client, ClientConfiguration, TransportConfiguration, WireEncoding
   from kmipkit.protocol import CryptographicAlgorithm, KeyUsage

   transport = (
       TransportConfiguration.raw_tls("kms.internal.example.com", 5696)
       .with_client_identity_pem_files("/etc/kmip/client.crt", "/etc/kmip/client.key")
       .with_trust_ca_pem_file("/etc/kmip/ca.crt")
       .build()
   )

   with Client(ClientConfiguration.defaults().with_wire_encoding(WireEncoding.TTLV), transport) as client:
       created = (
           client.create_symmetric_key()
           .algorithm(CryptographicAlgorithm.AES)
           .length(256)
           .usage_mask(KeyUsage.ENCRYPT | KeyUsage.DECRYPT)
           .name("python-dek")
           .execute()
       )
       client.activate(created.unique_identifier).execute()
