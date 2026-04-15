import { useParams } from "react-router-dom";
import DocumentDetailView from "../components/Documents/DocumentDetail";

export default function DocumentPage() {
  const { id } = useParams<{ id: string }>();

  if (!id) return null;

  return (
    <div className="h-full">
      <DocumentDetailView id={id} />
    </div>
  );
}
