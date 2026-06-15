import { invoke } from "@tauri-apps/api/core";
import type { ConvertFile, ConvertOptions } from "./types";

export interface ConvertRequest {
  input_path: string;
  output_format: string;
  options: {
    quality?: number;
    audio_bitrate?: number;
    sample_rate?: number;
    preserve_metadata?: boolean;
  };
}

export interface ConvertResult {
  output_path: string;
  size_bytes: number;
}

export async function convertFile(
  file: ConvertFile,
  options: ConvertOptions,
): Promise<ConvertResult> {
  const request: ConvertRequest = {
    input_path: file.filePath,
    output_format: file.targetFormat,
    options: {
      quality: options.quality,
      audio_bitrate: options.audioBitrate,
      sample_rate: options.sampleRate,
      preserve_metadata: options.preserveMetadata,
    },
  };

  return await invoke<ConvertResult>("convert_file", { request });
}
