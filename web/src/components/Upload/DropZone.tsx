import { useState, useCallback, useRef } from "react";
import { Upload, FileImage, CheckCircle2, AlertTriangle, X } from "lucide-react";
import { useUploadDocument } from "../../api/hooks";
import * as pdfjsLib from "pdfjs-dist";

pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
  "pdfjs-dist/build/pdf.worker.mjs",
  import.meta.url,
).toString();

/** Render first page of a PDF to a JPEG blob. */
async function pdfToJpeg(file: File): Promise<File> {
  const buf = await file.arrayBuffer();
  const pdf = await pdfjsLib.getDocument({ data: buf }).promise;
  const page = await pdf.getPage(1);
  const scale = 2; // 2x for high-quality rendering
  const viewport = page.getViewport({ scale });

  const canvas = document.createElement("canvas");
  canvas.width = viewport.width;
  canvas.height = viewport.height;
  const ctx = canvas.getContext("2d")!;
  // @ts-expect-error pdfjs types require canvas but canvasContext works in browser
  await page.render({ canvasContext: ctx, viewport }).promise;

  const blob = await new Promise<Blob>((resolve) =>
    canvas.toBlob((b) => resolve(b!), "image/jpeg", 0.92),
  );
  const name = file.name.replace(/\.pdf$/i, ".jpg");
  return new File([blob], name, { type: "image/jpeg" });
}

interface UploadItem {
  name: string;
  progress: number;
  status: "converting" | "queued" | "uploading" | "done" | "error";
  error?: string;
}

export default function DropZone() {
  const [items, setItems] = useState<UploadItem[]>([]);
  const [dragOver, setDragOver] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const upload = useUploadDocument();

  const updateItem = useCallback(
    (idx: number, patch: Partial<UploadItem>) =>
      setItems((prev) =>
        prev.map((item, i) => (i === idx ? { ...item, ...patch } : item)),
      ),
    [],
  );

  const uploadFile = useCallback(
    (file: File, idx: number) => {
      updateItem(idx, { status: "uploading" });
      upload.mutate(
        {
          file,
          onProgress: (pct) => updateItem(idx, { progress: pct }),
        },
        {
          onSuccess: () => updateItem(idx, { status: "done", progress: 100 }),
          onError: (err) =>
            updateItem(idx, {
              status: "error",
              error: err instanceof Error ? err.message : "Upload failed",
            }),
        },
      );
    },
    [upload, updateItem],
  );

  const processFile = useCallback(
    async (file: File, idx: number) => {
      if (file.type === "application/pdf") {
        updateItem(idx, { status: "converting" });
        try {
          const jpeg = await pdfToJpeg(file);
          uploadFile(jpeg, idx);
        } catch (err) {
          updateItem(idx, {
            status: "error",
            error: err instanceof Error ? err.message : "PDF conversion failed",
          });
        }
      } else {
        uploadFile(file, idx);
      }
    },
    [uploadFile, updateItem],
  );

  const addFiles = useCallback(
    (files: FileList | File[]) => {
      const accepted = Array.from(files).filter(
        (f) => f.type.startsWith("image/") || f.type === "application/pdf",
      );
      const newItems: UploadItem[] = accepted.map((f) => ({
        name: f.name,
        progress: 0,
        status: "queued" as const,
      }));
      const startIdx = items.length;
      setItems((prev) => [...prev, ...newItems]);
      accepted.forEach((file, i) => processFile(file, startIdx + i));
    },
    [items.length, processFile],
  );

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      setDragOver(false);
      addFiles(e.dataTransfer.files);
    },
    [addFiles],
  );

  return (
    <div className="space-y-4">
      {/* Drop area */}
      <div
        onDragOver={(e) => {
          e.preventDefault();
          setDragOver(true);
        }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
        onClick={() => inputRef.current?.click()}
        className={`flex cursor-pointer flex-col items-center justify-center rounded-2xl border-2 border-dashed px-6 py-16 transition-colors ${
          dragOver
            ? "border-accent bg-accent/5"
            : "border-edge hover:border-ink-faint hover:bg-surface-raised/50"
        }`}
      >
        <Upload
          size={40}
          strokeWidth={1.2}
          className={`mb-3 ${dragOver ? "text-accent" : "text-ink-faint"}`}
        />
        <p className="text-sm font-medium text-ink">
          Drop images here or click to browse
        </p>
        <p className="mt-1 text-xs text-ink-faint">
          JPEG, PNG, WebP, PDF — documents not captured via phone
        </p>
        <input
          ref={inputRef}
          type="file"
          accept="image/*,application/pdf"
          multiple
          className="hidden"
          onChange={(e) => {
            if (e.target.files) addFiles(e.target.files);
            e.target.value = "";
          }}
        />
      </div>

      {/* Upload list */}
      {items.length > 0 && (
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-ink">
              Uploads ({items.filter((i) => i.status === "done").length}/
              {items.length})
            </h3>
            {items.every(
              (i) => i.status === "done" || i.status === "error",
            ) && (
              <button
                onClick={() => setItems([])}
                className="text-xs text-ink-faint hover:text-ink"
              >
                Clear
              </button>
            )}
          </div>
          {items.map((item, i) => (
            <div
              key={i}
              className="flex items-center gap-3 rounded-lg border border-edge bg-surface p-3"
            >
              <FileImage size={18} className="flex-shrink-0 text-ink-faint" />
              <div className="flex-1 min-w-0">
                <p className="truncate text-sm text-ink">{item.name}</p>
                {item.status === "converting" && (
                  <p className="mt-0.5 text-xs text-accent">Converting PDF...</p>
                )}
                {item.status === "uploading" && (
                  <div className="mt-1.5 h-1 rounded-full bg-surface-raised">
                    <div
                      className="h-full rounded-full bg-accent transition-all"
                      style={{ width: `${item.progress}%` }}
                    />
                  </div>
                )}
                {item.error && (
                  <p className="mt-0.5 text-xs text-danger">{item.error}</p>
                )}
              </div>
              {item.status === "done" && (
                <CheckCircle2 size={16} className="text-success" />
              )}
              {item.status === "error" && (
                <AlertTriangle size={16} className="text-danger" />
              )}
              {(item.status === "done" || item.status === "error") && (
                <button
                  onClick={() =>
                    setItems((prev) => prev.filter((_, j) => j !== i))
                  }
                  className="p-0.5 text-ink-faint hover:text-ink"
                >
                  <X size={14} />
                </button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
