import type { BrowserKind } from "@/types";

interface BrowserIconProps {
  kind: BrowserKind;
  size?: number;
  className?: string;
}

export function BrowserIcon({ kind, size = 24, className = "" }: BrowserIconProps) {
  const icons: Record<BrowserKind, JSX.Element> = {
    chrome: (
      <svg viewBox="0 0 24 24" width={size} height={size} className={className}>
        <circle cx="12" cy="12" r="4" fill="#4285F4" />
        <path d="M12 8h8.66a10 10 0 1 0-1.33 10.33L12 12z" fill="none" stroke="#EA4335" strokeWidth="0" />
        <path d="M12 8h8.66a10 10 0 0 0-8.66-5A10 10 0 0 0 3.34 8H12z" fill="#EA4335" />
        <path d="M3.34 8A10 10 0 0 0 8 19.66L12 12 3.34 8z" fill="#34A853" />
        <path d="M16 19.66A10 10 0 0 0 20.66 8L12 12l4 7.66z" fill="#FBBC05" />
        <circle cx="12" cy="12" r="4" fill="#4285F4" />
        <circle cx="12" cy="12" r="2.5" fill="white" />
      </svg>
    ),
    firefox: (
      <svg viewBox="0 0 24 24" width={size} height={size} className={className}>
        <circle cx="12" cy="12" r="10" fill="#FF7139" />
        <path d="M12 4C7.6 4 4 7.6 4 12s3.6 8 8 8 8-3.6 8-8" fill="#FF9500" />
        <circle cx="12" cy="12" r="5" fill="#00539F" />
        <circle cx="12" cy="12" r="3" fill="#80BFFF" />
      </svg>
    ),
    edge: (
      <svg viewBox="0 0 24 24" width={size} height={size} className={className}>
        <path d="M4 12C4 7.6 7.6 4 12 4c2.5 0 4.8 1.1 6.3 2.9L12 8c-2.2 0-4 1.8-4 4v.5h12c0 4.4-3.6 7.5-8 7.5-4.4 0-8-3.6-8-8z" fill="#0078D4" />
        <path d="M12 13.5H8v.5c0 2.2 1.8 4 4 4s4-1.8 4-4h-4z" fill="#50E6FF" />
      </svg>
    ),
    brave: (
      <svg viewBox="0 0 24 24" width={size} height={size} className={className}>
        <path d="M20.5 7.5L18 3H6L3.5 7.5 4 12c.3 2.7 1.8 5.1 4 6.7L12 21l4-2.3c2.2-1.6 3.7-4 4-6.7l.5-4.5z" fill="#FB542B" />
        <path d="M12 6l-4 2.5v4L12 15l4-2.5v-4L12 6z" fill="white" />
      </svg>
    ),
    opera: (
      <svg viewBox="0 0 24 24" width={size} height={size} className={className}>
        <circle cx="12" cy="12" r="9" fill="#FF1B2D" />
        <ellipse cx="12" cy="12" rx="4.5" ry="9" fill="none" stroke="white" strokeWidth="1.5" />
        <ellipse cx="12" cy="12" rx="9" ry="9" fill="none" stroke="white" strokeWidth="0" />
        <path d="M7.5 12a4.5 9 0 0 1 4.5-9" stroke="white" strokeWidth="0" />
      </svg>
    ),
    safari: (
      <svg viewBox="0 0 24 24" width={size} height={size} className={className}>
        <circle cx="12" cy="12" r="10" fill="#006CFF" />
        <circle cx="12" cy="12" r="9" fill="none" stroke="white" strokeWidth="0.5" opacity="0.3" />
        <line x1="12" y1="3" x2="12" y2="21" stroke="white" strokeWidth="0.5" opacity="0.3" />
        <line x1="3" y1="12" x2="21" y2="12" stroke="white" strokeWidth="0.5" opacity="0.3" />
        <polygon points="12,5 15,12 12,10 9,12" fill="#FF3B30" />
        <polygon points="12,19 9,12 12,14 15,12" fill="white" />
      </svg>
    ),
  };

  return icons[kind] ?? null;
}
