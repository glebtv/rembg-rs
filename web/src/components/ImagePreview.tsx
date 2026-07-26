interface ImagePreviewProps {
  src: string;
  title: string;
}

export function ImagePreview({ src, title }: ImagePreviewProps) {
  return (
    <div className="preview-column">
      <span className="preview-label">
        {title}
      </span>
      <div className="image-stage">
        <img
          src={src}
          alt={title}
          data-testid={title === 'Mask' ? 'mask-image' : undefined}
          className="preview-image"
        />
      </div>
    </div>
  );
}
