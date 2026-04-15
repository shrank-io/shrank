import { Upload } from "lucide-react";
import DropZone from "../components/Upload/DropZone";

export default function UploadPage() {
  return (
    <div className="mx-auto max-w-2xl p-6">
      <div className="mb-6 flex items-center gap-2">
        <Upload size={18} className="text-ink-faint" />
        <h1 className="text-xl font-semibold text-ink">Upload documents</h1>
      </div>
      <p className="mb-6 text-sm text-ink-muted">
        Drag and drop scanned documents here. The inference sidecar will
        extract metadata, text, and build relationships automatically.
      </p>
      <DropZone />
    </div>
  );
}
