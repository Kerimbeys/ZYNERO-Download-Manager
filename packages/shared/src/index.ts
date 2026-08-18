/**
 * ZYNERO shared domain contract.
 * Keep this file transport-oriented: values are JSON serializable across Tauri IPC.
 */

export const downloadStatuses = [
  "queued",
  "active",
  "paused",
  "completed",
  "failed",
  "cancelled",
] as const;

export type DownloadStatus = (typeof downloadStatuses)[number];

export type Download = {
  id: string;
  url: string;
  filename: string;
  destination: string;
  status: DownloadStatus;
  totalBytes: number | null;
  downloadedBytes: number;
  contentType: string | null;
  supportsRange: boolean;
  tempPath: string | null;
  finalPath: string | null;
  errorMessage: string | null;
  speedBps: number;
  etaSeconds: number;
  category: string;
  createdAt: string;
  updatedAt: string;
};

export type SegmentStatus = "pending" | "active" | "paused" | "completed" | "failed" | "cancelled";

export type Segment = {
  id: string;
  downloadId: string;
  segmentIndex: number;
  startByte: number;
  endByte: number;
  downloadedBytes: number;
  status: SegmentStatus;
  tempPath: string | null;
  errorMessage: string | null;
};

export type DownloadProgressPayload = {
  download: Download;
  emittedAt: string;
};

export type IpcErrorCode =
  | "validation"
  | "not_found"
  | "network"
  | "filesystem"
  | "database"
  | "permission"
  | "cancelled"
  | "unknown";

export type IpcError = {
  code: IpcErrorCode;
  message: string;
  retryable: boolean;
  details?: Record<string, string | number | boolean | null>;
};

export type IpcResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: IpcError };
