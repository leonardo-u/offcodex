//! OSS provider utilities shared between TUI and exec.

use codex_core::config::Config;
use codex_model_provider_info::LMSTUDIO_OSS_PROVIDER_ID;
use codex_model_provider_info::OLLAMA_OSS_PROVIDER_ID;
use std::io;
use std::path::PathBuf;

pub const DEFAULT_LOCAL_TEMPERATURE: f32 = 0.1;
pub const DEFAULT_LOCAL_NUM_CTX: u32 = 16_384;

/// Recommend a context window from the first NVIDIA GPU's reported VRAM.
pub fn recommended_local_num_ctx() -> u32 {
    let output = std::process::Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output();
    let memory_mb = output
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| output.lines().next()?.trim().parse::<u32>().ok());
    match memory_mb {
        Some(memory_mb) if memory_mb <= 8_192 => 8_192,
        Some(memory_mb) if memory_mb <= 12_288 => 12_288,
        Some(memory_mb) if memory_mb <= 16_384 => 16_384,
        Some(memory_mb) if memory_mb <= 24_576 => 32_768,
        Some(_) => 65_536,
        None => DEFAULT_LOCAL_NUM_CTX,
    }
}

fn global_default_path() -> io::Result<PathBuf> {
    dirs::config_dir()
        .map(|path| path.join("offcodex").join("default-model"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "could not locate config directory"))
}

/// Read the model selected as the global offcodex default, if one was saved.
pub fn read_global_default_model() -> io::Result<Option<String>> {
    let path = global_default_path()?;
    match std::fs::read_to_string(path) {
        Ok(model) => {
            let model = model.trim();
            Ok((!model.is_empty()).then(|| model.to_string()))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

/// Save a model as the global offcodex default.
pub fn write_global_default_model(model: &str) -> io::Result<()> {
    let path = global_default_path()?;
    let Some(parent) = path.parent() else {
        return Err(io::Error::other("invalid offcodex config path"));
    };
    std::fs::create_dir_all(parent)?;
    std::fs::write(path, format!("{model}\n"))
}

/// Remove the global offcodex default so the next startup asks again.
pub fn reset_global_default_model() -> io::Result<()> {
    let path = global_default_path()?;
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Return the conventional name for a model customized by offcodex.
pub fn offcodex_model_name(base_model: &str) -> String {
    if base_model.starts_with("offcodex-") {
        base_model.to_string()
    } else {
        format!("offcodex-{}", base_model.replace('/', "-"))
    }
}

/// Returns the default model for a given OSS provider.
pub fn get_default_model_for_oss_provider(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        LMSTUDIO_OSS_PROVIDER_ID => Some(codex_lmstudio::DEFAULT_OSS_MODEL),
        OLLAMA_OSS_PROVIDER_ID => Some(codex_ollama::DEFAULT_OSS_MODEL),
        _ => None,
    }
}

/// Ensures the specified OSS provider is ready (models downloaded, service reachable).
pub async fn ensure_oss_provider_ready(
    provider_id: &str,
    config: &Config,
) -> Result<(), std::io::Error> {
    match provider_id {
        LMSTUDIO_OSS_PROVIDER_ID => {
            codex_lmstudio::ensure_oss_ready(config)
                .await
                .map_err(|e| std::io::Error::other(format!("OSS setup failed: {e}")))?;
        }
        OLLAMA_OSS_PROVIDER_ID => {
            let client = codex_ollama::OllamaClient::try_from_oss_provider(config).await?;
            codex_ollama::ensure_responses_supported(&client).await?;
            codex_ollama::ensure_oss_ready(config, &client)
                .await
                .map_err(|e| std::io::Error::other(format!("OSS setup failed: {e}")))?;
        }
        _ => {
            // Unknown provider, skip setup
        }
    }

    Ok(())
}

/// Return model names installed in the selected local provider.
pub async fn local_provider_models(
    provider_id: &str,
    config: &Config,
) -> Result<Vec<String>, std::io::Error> {
    match provider_id {
        OLLAMA_OSS_PROVIDER_ID => {
            let client = codex_ollama::OllamaClient::try_from_oss_provider(config).await?;
            client.fetch_models().await
        }
        _ => Ok(Vec::new()),
    }
}

/// Return explicit textual tool-call wrappers declared by the selected local model.
pub async fn local_provider_tool_call_profile(
    provider_id: &str,
    config: &Config,
    model: &str,
) -> Result<Vec<String>, std::io::Error> {
    match provider_id {
        OLLAMA_OSS_PROVIDER_ID => {
            let client = codex_ollama::OllamaClient::try_from_oss_provider(config).await?;
            Ok(client.fetch_tool_call_profile(model).await?.wrappers)
        }
        _ => Ok(Vec::new()),
    }
}

/// Create an Ollama model with the offcodex tool-calling instructions.
pub async fn create_local_model_variant(
    provider_id: &str,
    config: &Config,
    base_model: &str,
    model: &str,
    temperature: f32,
    num_ctx: u32,
) -> Result<(), io::Error> {
    match provider_id {
        OLLAMA_OSS_PROVIDER_ID => {
            let client = codex_ollama::OllamaClient::try_from_oss_provider(config).await?;
            client
                .create_model(model, base_model, temperature, num_ctx)
                .await
        }
        _ => Err(io::Error::other(
            "selected provider cannot create local variants",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_model_for_provider_lmstudio() {
        let result = get_default_model_for_oss_provider(LMSTUDIO_OSS_PROVIDER_ID);
        assert_eq!(result, Some(codex_lmstudio::DEFAULT_OSS_MODEL));
    }

    #[test]
    fn test_get_default_model_for_provider_ollama() {
        let result = get_default_model_for_oss_provider(OLLAMA_OSS_PROVIDER_ID);
        assert_eq!(result, Some(codex_ollama::DEFAULT_OSS_MODEL));
    }

    #[test]
    fn test_get_default_model_for_provider_unknown() {
        let result = get_default_model_for_oss_provider("unknown-provider");
        assert_eq!(result, None);
    }
}
