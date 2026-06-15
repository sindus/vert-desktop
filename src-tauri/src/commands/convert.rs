use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use uuid::Uuid;

use crate::error::{AppError, Result};

#[derive(Debug, serde::Deserialize)]
pub struct ConvertRequest {
    pub input_path: String,
    pub output_format: String,
    pub options: ConvertOptions,
}

#[derive(Debug, serde::Deserialize)]
pub struct ConvertOptions {
    pub quality: Option<u8>,
    pub audio_bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub preserve_metadata: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
pub struct ConvertResult {
    pub output_path: String,
    pub size_bytes: u64,
}

fn detect_category(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        "mp3" | "wav" | "flac" | "ogg" | "opus" | "aac" | "alac" | "m4a" | "wma" | "aiff"
        | "mp2" | "au" | "m4b" | "voc" => "audio",
        "mp4" | "mkv" | "webm" | "avi" | "mov" | "wmv" | "gif" | "mts" | "ts" | "m2ts"
        | "mpg" | "mpeg" | "flv" | "vob" | "m4v" | "3gp" | "ogv" => "video",
        "docx" | "doc" | "md" | "html" | "rtf" | "csv" | "rst" | "epub" | "odt" => "document",
        _ => "image",
    }
}

#[tauri::command]
pub async fn convert_file(
    app: AppHandle,
    request: ConvertRequest,
) -> std::result::Result<ConvertResult, AppError> {
    let input_path = PathBuf::from(&request.input_path);

    let input_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let category = detect_category(&input_ext);
    let output_format = request.output_format.to_lowercase();

    let tmp_dir = tempfile::tempdir().map_err(AppError::from)?;
    let output_filename = format!("{}.{}", Uuid::new_v4(), output_format);
    let output_path = tmp_dir.path().join(&output_filename);

    match category {
        "image" => {
            convert_image(&app, &input_path, &output_path, &request.options).await?
        }
        "audio" => {
            convert_audio(&app, &input_path, &output_path, &request.options).await?
        }
        "video" => {
            convert_video(&app, &input_path, &output_path, &request.options).await?
        }
        "document" => {
            convert_document(&app, &input_path, &output_path).await?
        }
        _ => return Err(AppError::UnsupportedFormat(input_ext)),
    }

    let size_bytes = std::fs::metadata(&output_path)
        .map_err(AppError::from)?
        .len();

    // Move output to a stable temp location before the tmp_dir is dropped
    let stable_output = std::env::temp_dir().join(&output_filename);
    std::fs::copy(&output_path, &stable_output).map_err(AppError::from)?;

    Ok(ConvertResult {
        output_path: stable_output.to_string_lossy().to_string(),
        size_bytes,
    })
}

async fn convert_image(
    app: &AppHandle,
    input: &PathBuf,
    output: &PathBuf,
    opts: &ConvertOptions,
) -> Result<()> {
    let quality = opts.quality.unwrap_or(92).to_string();

    let output = app
        .shell()
        .sidecar("magick")
        .map_err(|e| AppError::SidecarNotFound(e.to_string()))?
        .args([
            input.to_string_lossy().as_ref(),
            "-quality",
            &quality,
            output.to_string_lossy().as_ref(),
        ])
        .output()
        .await
        .map_err(|e| AppError::ProcessError(e.to_string()))?;

    if !output.status.success() {
        return Err(AppError::ConversionFailed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    Ok(())
}

async fn convert_audio(
    app: &AppHandle,
    input: &PathBuf,
    output: &PathBuf,
    opts: &ConvertOptions,
) -> Result<()> {
    let mut args: Vec<String> = vec![
        "-i".into(),
        input.to_string_lossy().to_string(),
        "-y".into(),
    ];

    if let Some(bitrate) = opts.audio_bitrate {
        args.push("-b:a".into());
        args.push(format!("{}k", bitrate));
    }

    if let Some(sample_rate) = opts.sample_rate {
        args.push("-ar".into());
        args.push(sample_rate.to_string());
    }

    if opts.preserve_metadata.unwrap_or(true) {
        args.push("-map_metadata".into());
        args.push("0".into());
    }

    args.push(output.to_string_lossy().to_string());

    let result = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| AppError::SidecarNotFound(e.to_string()))?
        .args(args)
        .output()
        .await
        .map_err(|e| AppError::ProcessError(e.to_string()))?;

    if !result.status.success() {
        return Err(AppError::ConversionFailed(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }

    Ok(())
}

async fn convert_video(
    app: &AppHandle,
    input: &PathBuf,
    output: &PathBuf,
    opts: &ConvertOptions,
) -> Result<()> {
    let mut args: Vec<String> = vec![
        "-i".into(),
        input.to_string_lossy().to_string(),
        "-y".into(),
    ];

    if opts.preserve_metadata.unwrap_or(true) {
        args.push("-map_metadata".into());
        args.push("0".into());
    }

    args.push(output.to_string_lossy().to_string());

    let result = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| AppError::SidecarNotFound(e.to_string()))?
        .args(args)
        .output()
        .await
        .map_err(|e| AppError::ProcessError(e.to_string()))?;

    if !result.status.success() {
        return Err(AppError::ConversionFailed(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }

    Ok(())
}

async fn convert_document(
    app: &AppHandle,
    input: &PathBuf,
    output: &PathBuf,
) -> Result<()> {
    let result = app
        .shell()
        .sidecar("pandoc")
        .map_err(|e| AppError::SidecarNotFound(e.to_string()))?
        .args([
            input.to_string_lossy().as_ref(),
            "-o",
            output.to_string_lossy().as_ref(),
        ])
        .output()
        .await
        .map_err(|e| AppError::ProcessError(e.to_string()))?;

    if !result.status.success() {
        return Err(AppError::ConversionFailed(
            String::from_utf8_lossy(&result.stderr).to_string(),
        ));
    }

    Ok(())
}

#[tauri::command]
pub async fn save_file(from: String, to: String) -> std::result::Result<(), AppError> {
    std::fs::copy(&from, &to)
        .map(|_| ())
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn get_file_info(path: String) -> std::result::Result<FileInfo, AppError> {
    let path = PathBuf::from(&path);
    let meta = std::fs::metadata(&path).map_err(AppError::from)?;

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let category = detect_category(&ext);

    Ok(FileInfo {
        name: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        size_bytes: meta.len(),
        extension: ext,
        category: category.to_string(),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct FileInfo {
    pub name: String,
    pub size_bytes: u64,
    pub extension: String,
    pub category: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_category_audio() {
        assert_eq!(detect_category("mp3"), "audio");
        assert_eq!(detect_category("MP3"), "audio");
        assert_eq!(detect_category("flac"), "audio");
        assert_eq!(detect_category("wav"), "audio");
        assert_eq!(detect_category("ogg"), "audio");
    }

    #[test]
    fn test_detect_category_video() {
        assert_eq!(detect_category("mp4"), "video");
        assert_eq!(detect_category("mkv"), "video");
        assert_eq!(detect_category("avi"), "video");
        assert_eq!(detect_category("webm"), "video");
    }

    #[test]
    fn test_detect_category_document() {
        assert_eq!(detect_category("docx"), "document");
        assert_eq!(detect_category("md"), "document");
        assert_eq!(detect_category("epub"), "document");
        assert_eq!(detect_category("html"), "document");
    }

    #[test]
    fn test_detect_category_image() {
        assert_eq!(detect_category("png"), "image");
        assert_eq!(detect_category("jpg"), "image");
        assert_eq!(detect_category("webp"), "image");
        assert_eq!(detect_category("svg"), "image");
    }

    #[test]
    fn test_detect_category_unknown_is_image() {
        assert_eq!(detect_category("xyz"), "image");
        assert_eq!(detect_category(""), "image");
    }
}
