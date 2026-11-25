use std::fs;
use zed_extension_api::{self as zed, LanguageServerId, Result};

struct KmipKitExtension {
    cached_binary_path: Option<String>,
}

impl KmipKitExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = worktree.which("kmip-lsp") {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path
            && fs::metadata(path).is_ok_and(|stat| stat.is_file())
        {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            "NeverWe1come/KMIPKit",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (platform, arch) = zed::current_platform();
        let platform_dir = match (platform, arch) {
            (zed::Os::Mac, zed::Architecture::Aarch64) => "darwin-arm64",
            (zed::Os::Mac, zed::Architecture::X8664) => "darwin-x64",
            (zed::Os::Linux, zed::Architecture::Aarch64) => "linux-arm64",
            (zed::Os::Linux, zed::Architecture::X8664) => "linux-x64",
            (zed::Os::Windows, zed::Architecture::X8664) => "win32-x64",
            _ => return Err("Unsupported platform for kmip-lsp".into()),
        };

        let exe_name = match platform {
            zed::Os::Windows => "kmip-lsp.exe",
            _ => "kmip-lsp",
        };
        let asset_name = format!("kmip-lsp-{platform_dir}-{exe_name}");

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("Release asset '{asset_name}' not found"))?;

        let version_dir = format!("kmip-lsp-{}", release.version);
        fs::create_dir_all(&version_dir).map_err(|e| format!("failed to create dir: {e}"))?;
        let binary_path = format!("{version_dir}/{exe_name}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )?;
            zed::make_file_executable(&binary_path)?;
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for KmipKitExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_path = self.language_server_binary_path(language_server_id, worktree)?;
        Ok(zed::Command {
            command: binary_path,
            args: vec!["--stdio".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(KmipKitExtension);
