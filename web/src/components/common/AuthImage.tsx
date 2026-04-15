import { useAuthImage } from "../../api/hooks";

export default function AuthImage({
  src,
  alt,
  className,
}: {
  src: string;
  alt: string;
  className?: string;
}) {
  const { data: objectUrl, isLoading } = useAuthImage(src);

  if (isLoading || !objectUrl) {
    return <div className={`skeleton ${className ?? ""}`} />;
  }

  return <img src={objectUrl} alt={alt} className={className} loading="lazy" />;
}
