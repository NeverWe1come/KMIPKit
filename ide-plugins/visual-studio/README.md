# KMIPKit Visual Studio Extension (`ide-plugins/visual-studio/`)

Out-of-process **VisualStudio.Extensibility** extension for Visual Studio 2022+ (Windows), connecting to `kmip-lsp --stdio` over `IDuplexStream`.

## Build (Windows + .NET 8 SDK)
```powershell
dotnet build ide-plugins/visual-studio/KmipKit.VisualStudio.csproj -c Release
```
