import { TransformWrapper, TransformComponent } from "react-zoom-pan-pinch";
import { ZoomIn, ZoomOut, RotateCcw } from "lucide-react";

export default function ImageViewer({ src, alt }: { src: string; alt: string }) {
  return (
    <TransformWrapper
      initialScale={1}
      minScale={0.5}
      maxScale={5}
      wheel={{ step: 0.1 }}
    >
      {({ zoomIn, zoomOut, resetTransform }) => (
        <div className="relative h-full w-full overflow-hidden rounded-lg bg-black/30">
          <div className="absolute top-3 right-3 z-10 flex gap-1">
            {[
              { icon: ZoomIn, action: () => zoomIn() },
              { icon: ZoomOut, action: () => zoomOut() },
              { icon: RotateCcw, action: () => resetTransform() },
            ].map(({ icon: Icon, action }, i) => (
              <button
                key={i}
                onClick={action}
                className="rounded-md bg-surface/80 p-1.5 text-ink-muted backdrop-blur transition-colors hover:bg-surface-raised hover:text-ink"
              >
                <Icon size={16} />
              </button>
            ))}
          </div>
          <TransformComponent
            wrapperStyle={{ width: "100%", height: "100%" }}
            contentStyle={{
              width: "100%",
              height: "100%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <img
              src={src}
              alt={alt}
              className="max-h-full max-w-full object-contain"
            />
          </TransformComponent>
        </div>
      )}
    </TransformWrapper>
  );
}
