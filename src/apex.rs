use std::fs;

use std::io;
use zed_extension_api::{
    lsp::{Completion, CompletionKind},
    make_file_executable, register_extension, set_language_server_installation_status, CodeLabel,
    CodeLabelSpan, Extension, LanguageServerId, LanguageServerInstallationStatus, Result, Worktree,
};

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

        set_language_server_installation_status(
            &language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );

        // download_file(&download_url, &version_dir)
        // .map_err(|e| format!("failed to download file: {e}"))?;
        let resp = reqwest::blocking::get(download_url).expect("request failed");
        let body = resp.text().expect("body invalid");
        let mut out = fs::File::create(&binary_path).expect("failed to create file");
        io::copy(&mut body.as_bytes(), &mut out).expect("failed to copy content");

        make_file_executable(&binary_path)?;

        // let entries =
        //     fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
        // for entry in entries {
        //     let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
        // }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl Extension for ApexExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        _: &Worktree,
    ) -> Result<zed_extension_api::Command> {
        Ok(zed_extension_api::Command {
            command: self.language_server_binary_path(language_server_id)?,
            args: Vec::new(),
            env: Default::default(),
        })
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        match completion.kind? {
            CompletionKind::Method => {
                let (name_and_params, return_type) = completion.label.split_once(" : ")?;
                let name = name_and_params.split('(').next()?;
                let code = format!("{return_type} {name_and_params}");

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::code_range(return_type.len() + 1..code.len()),
                        CodeLabelSpan::literal(" : ", None),
                        CodeLabelSpan::code_range(0..return_type.len()),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Constructor => {
                let new = "new ";
                let code = format!("{new}{}", completion.label);
                let name = completion.label.split('(').next()?;

                Some(CodeLabel {
                    spans: vec![CodeLabelSpan::code_range(new.len()..code.len())],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Variable | CompletionKind::Field | CompletionKind::Constant => {
                let (name, r#type) = completion.label.split_once(" : ")?;
                let code = format!("{type} {name}");
                let highlight_name = match completion.kind? {
                    CompletionKind::Field => Some("property".to_string()),
                    CompletionKind::Constant => Some("constant".to_string()),
                    _ => None,
                };

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::literal(name, highlight_name),
                        CodeLabelSpan::literal(" : ", None),
                        CodeLabelSpan::code_range(0..r#type.len()),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Class | CompletionKind::Interface | CompletionKind::Enum => {
                let (name, namespace) = completion.label.split_once(" - ")?;
                let namespace_hint = format!(" ({namespace})");
                let code = format!("{name}{namespace_hint}");

                Some(CodeLabel {
                    spans: vec![
                        CodeLabelSpan::literal(name, Some("type".to_string())),
                        CodeLabelSpan::literal(namespace_hint, None),
                    ],
                    filter_range: (0..name.len()).into(),
                    code,
                })
            }
            CompletionKind::Keyword => Some(CodeLabel {
                spans: vec![CodeLabelSpan::code_range(0..completion.label.len())],
                filter_range: (0..completion.label.len()).into(),
                code: completion.label,
            }),
            CompletionKind::EnumMember => {
                let name = completion.label.split(" : ").next()?;

                Some(CodeLabel {
                    code: name.to_string(),
                    spans: vec![CodeLabelSpan::code_range(0..name.len())],
                    filter_range: (0..name.len()).into(),
                })
            }
            _ => None,
        }
    }
}

register_extension!(ApexExtension);
