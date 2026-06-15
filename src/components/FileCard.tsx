import { Download, Trash2, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import clsx from "clsx";
import type { ConvertFile } from "../converters/types";
import FormatDropdown from "./FormatDropdown";
import ProgressBar from "./ProgressBar";
import { save } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store";

interface FileCardProps {
  file: ConvertFile;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "—";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function FileCard({ file }: FileCardProps) {
  const { t } = useTranslation();
  const removeFile = useAppStore((s) => s.removeFile);
  const setTargetFormat = useAppStore((s) => s.setTargetFormat);
  const addToast = useAppStore((s) => s.addToast);

  const isConverting = file.status === "converting";
  const isDone = file.status === "done";
  const isError = file.status === "error";

  const handleDownload = async () => {
    if (!file.outputPath) return;

    try {
      const savePath = await save({
        defaultPath: `${file.name.replace(/\.[^.]+$/, "")}.${file.targetFormat}`,
        filters: [{ name: file.targetFormat.toUpperCase(), extensions: [file.targetFormat] }],
      });

      if (savePath) {
        await invoke("save_file", { from: file.outputPath, to: savePath });
        addToast(`${file.name} → ${file.targetFormat.toUpperCase()} saved`, "success");
      }
    } catch {
      addToast(t("errors.saveError"), "error");
    }
  };

  return (
    <div
      className={clsx(
        "flex flex-col gap-3 rounded-xl border p-4 transition-all",
        isDone && "border-green-200 bg-green-50/50 dark:border-green-800 dark:bg-green-950/20",
        isError && "border-red-200 bg-red-50/50 dark:border-red-800 dark:bg-red-950/20",
        !isDone && !isError && "border-[rgb(var(--border))] bg-[rgb(var(--card))]",
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium text-[rgb(var(--fg))]">{file.name}</p>
          <p className="mt-0.5 text-xs text-[rgb(var(--muted))]">
            {formatBytes(file.sizeBytes)} · {file.extension.toUpperCase()} · {file.category}
          </p>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          {!isConverting && (
            <FormatDropdown
              category={file.category}
              value={file.targetFormat}
              onChange={(fmt) => setTargetFormat(file.id, fmt)}
              disabled={isDone}
            />
          )}

          {isDone && (
            <button
              onClick={handleDownload}
              className="flex items-center gap-1.5 rounded-lg bg-brand-500 px-3 py-1.5 text-xs font-medium text-white hover:bg-brand-600"
            >
              <Download size={13} />
              {t("convert.download")}
            </button>
          )}

          {isConverting ? (
            <button className="rounded-lg p-1.5 text-[rgb(var(--muted))] hover:text-red-500" title={t("convert.cancel")}>
              <X size={16} />
            </button>
          ) : (
            <button
              onClick={() => removeFile(file.id)}
              className="rounded-lg p-1.5 text-[rgb(var(--muted))] hover:text-red-500"
              title={t("convert.remove")}
            >
              <Trash2 size={16} />
            </button>
          )}
        </div>
      </div>

      {isConverting && <ProgressBar value={file.progress} />}

      {isError && file.error && (
        <p className="text-xs text-red-600 dark:text-red-400">{file.error}</p>
      )}
    </div>
  );
}
