using System.Diagnostics;
using System.IO.Pipelines;
using Microsoft.VisualStudio.Extensibility;
using Microsoft.VisualStudio.Extensibility.Editor;
using Microsoft.VisualStudio.Extensibility.LanguageServer;
using Nerdbank.Streams;

namespace KmipKit.VisualStudio;

[VisualStudioContribution]
internal class KmipLanguageServerProvider : LanguageServerProvider
{
    public override LanguageServerProviderConfiguration LanguageServerProviderConfiguration => new(
        "%KmipKit.LanguageServer%",
        new[]
        {
            DocumentFilter.FromDocumentType("json"),
            DocumentFilter.FromDocumentType("C/C++"),
            DocumentFilter.FromDocumentType("CSharp"),
        });

    public override Task<IDuplexStream?> CreateServerConnectionAsync(CancellationToken cancellationToken)
    {
        var binaryPath = Environment.GetEnvironmentVariable("KMIP_LSP_PATH") ?? "kmip-lsp.exe";
        var startInfo = new ProcessStartInfo
        {
            FileName = binaryPath,
            Arguments = "--stdio",
            RedirectStandardInput = true,
            RedirectStandardOutput = true,
            UseShellExecute = false,
            CreateNoWindow = true,
        };

        var process = Process.Start(startInfo);
        if (process == null)
        {
            return Task.FromResult<IDuplexStream?>(null);
        }

        return Task.FromResult<IDuplexStream?>(
            FullDuplexStream.Splice(process.StandardOutput.BaseStream, process.StandardInput.BaseStream));
    }
}
