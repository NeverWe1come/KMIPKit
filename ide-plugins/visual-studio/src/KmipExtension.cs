using Microsoft.VisualStudio.Extensibility;

namespace KmipKit.VisualStudio;

[VisualStudioContribution]
internal class KmipExtension : Extension
{
    public override ExtensionConfiguration ExtensionConfiguration => new()
    {
        Metadata = new(
            id: "KmipKit.VisualStudio.9b3f2b6a-1c2e-4f0b-8c7a-000000000001",
            version: ExtensionAssemblyVersion,
            publisherName: "KMIPKit",
            displayName: "KMIPKit — OASIS KMIP & TTLV Inspector",
            description: "Out-of-proc VisualStudio.Extensibility integration with kmip-lsp.")
    };
}
