import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { formatDistanceToNow, format } from "date-fns";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

export function formatRelativeTime(isoString: string): string {
  try {
    return formatDistanceToNow(new Date(isoString), { addSuffix: true });
  } catch {
    return isoString;
  }
}

export function formatDateTime(isoString: string): string {
  try {
    return format(new Date(isoString), "MMM d, yyyy 'at' h:mm a");
  } catch {
    return isoString;
  }
}

export function formatTimestamp(microseconds: number): string {
  // Handle both microseconds and milliseconds
  const ms = microseconds > 1e12 ? microseconds / 1000 : microseconds;
  try {
    return format(new Date(ms), "MMM d, yyyy");
  } catch {
    return String(microseconds);
  }
}

export function truncateUrl(url: string, maxLength = 60): string {
  if (url.length <= maxLength) return url;
  try {
    const u = new URL(url);
    return u.hostname + (u.pathname.length > 1 ? u.pathname.slice(0, 30) + "…" : "");
  } catch {
    return url.slice(0, maxLength) + "…";
  }
}

export function getDomainFromUrl(url: string): string {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

export function pluralize(count: number, singular: string, plural?: string): string {
  return count === 1 ? `${count} ${singular}` : `${count} ${plural ?? singular + "s"}`;
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/** Group an array by a key function */
export function groupBy<T>(arr: T[], key: (item: T) => string): Record<string, T[]> {
  return arr.reduce(
    (acc, item) => {
      const k = key(item);
      (acc[k] ??= []).push(item);
      return acc;
    },
    {} as Record<string, T[]>
  );
}
