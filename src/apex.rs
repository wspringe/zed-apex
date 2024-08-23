use std::fs;

use zed::LanguageServerId;
use zed_extension_api::{self as zed, Result};

struct ApexExtension {
    cached_binary_path: Option<String>,
}

impl ApexExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let download_url = "https://raw.githubusercontent.com/forcedotcom/salesforcedx-vscode/develop/packages/salesforcedx-vscode-apex/out/apex-jorje-lsp.jar";
        let binary_path = format!("apex-jorje-lsp.jar");

        zed::set_language_server_installation_status(
            &language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        zed::download_file(
            &download_url,
            &binary_path,
            zed::DownloadedFileType::Uncompressed,
        )
        .map_err(|e| format!("failed to download file error: {e}"))?;

        zed::make_file_executable(&binary_path)
            .map_err(|e| format!("failed to make binary executable: {e}"))?;

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for ApexExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        Ok(zed::Command {
            command: self.language_server_binary_path(language_server_id)?,
            args: vec![],
            env: Default::default(),
        })
    }
}

zed::register_extension!(ApexExtension);
