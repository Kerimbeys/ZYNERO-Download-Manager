import { useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import type { DownloadStatus as SharedDownloadStatus } from "@zynero/shared";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
import {
  Archive,
  ArrowDownToLine,
  Check,
  ChevronDown,
  Clock3,
  Download,
  ExternalLink,
  FolderOpen,
  Gauge,
  History,
  ListFilter,
  MoreHorizontal,
  Palette,
  Pause,
  Play,
  Plus,
  RotateCcw,
  Search,
  Settings,
  ShieldCheck,
  Sparkles,
  Trash2,
  X,
  Zap,
} from "lucide-react";
import "./styles.css";

type DownloadStatus = SharedDownloadStatus;

type DownloadItem = {
  id: string;
  name: string;
  url: string;
  size: number;
  downloaded: number;
  speed: number;
  etaSeconds: number;
  status: DownloadStatus;
  connections: number;
  destination: string;
  category: string;
};

const categoryLabels: Record<string, string> = {
  all: "All categories",
  archives: "Archives",
  audio: "Audio",
  video: "Video",
  images: "Images",
  documents: "Documents",
  applications: "Applications",
  other: "Other",
};

type StoredDownload = {
  id: string;
  url: string;
  filename: string;
  destination: string;
  status: string;
  totalBytes: number | null;
  downloadedBytes: number;
  contentType: string | null;
  supportsRange: boolean;
  tempPath: string | null;
  finalPath: string | null;
  errorMessage: string | null;
  speedBps: number;
  category: string;

  etaSeconds: number;
};

type NavItem = {
  label: string;
  icon: typeof Download;
  count?: number;
};

const notifiedDownloads = new Set<string>();

async function notifyIfNeeded(row: StoredDownload, enabled: boolean) {
  if (!enabled || !['completed', 'failed'].includes(row.status) || notifiedDownloads.has(row.id)) return;
  notifiedDownloads.add(row.id);
  try {
    let permission = await isPermissionGranted();
    if (!permission) permission = (await requestPermission()) === 'granted';
    if (!permission) return;
    sendNotification({
      title: row.status === 'completed' ? 'Download complete' : 'Download failed',
      body: row.status === 'completed' ? `${row.filename} is ready.` : `${row.filename}: ${row.errorMessage ?? 'The transfer could not be completed.'}`,
    });
  } catch {
    // Notifications are optional and must never interrupt download state updates.
  }
}

const navItems: NavItem[] = [
  { label: "Downloads", icon: Download },
  { label: "Active", icon: Zap },
  { label: "Completed", icon: Check },
  { label: "Queued", icon: Clock3 },
  { label: "Scheduled", icon: Clock3 },
  { label: "Failed", icon: RotateCcw },
  { label: "History", icon: History },
];

const formatBytes = (bytes: number) => {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** index).toFixed(index === 0 ? 0 : 1)} ${units[index]}`;
};

const formatSpeed = (bytesPerSecond: number) => `${formatBytes(bytesPerSecond)}/s`;

const formatEta = (seconds: number) => {
  if (!seconds) return "—";
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return minutes ? `${minutes}m ${remainingSeconds}s` : `${remainingSeconds}s`;
};

function StatCard({ label, value, detail, icon: Icon, accent }: { label: string; value: string; detail: string; icon: typeof Gauge; accent: string }) {
  return (
    <article className="stat-card">
      <div className="stat-card__topline">
        <span>{label}</span>
        <span className="stat-card__icon" style={{ color: accent }}><Icon size={16} strokeWidth={2.2} /></span>
      </div>
      <strong>{value}</strong>
      <small>{detail}</small>
    </article>
  );
}

function EmptyDownloads({ onAdd }: { onAdd: () => void }) {
  return (
    <section className="empty-state">
      <div className="empty-state__icon"><ArrowDownToLine size={26} /></div>
      <div className="section-kicker">YOUR DOWNLOADS</div>
      <h2>Ready for your first download?</h2>
      <p>Add a URL and ZYNERO will handle the rest with secure, resumable transfers.</p>
      <button className="button button--primary" type="button" onClick={onAdd}>
        <Plus size={17} /> Add your first download
      </button>
      <div className="empty-state__hint"><ShieldCheck size={14} /> Files stay on your device</div>
    </section>
  );
}

function DownloadCard({ item, onAction }: { item: DownloadItem; onAction: (action: "pause" | "resume" | "cancel" | "delete" | "openFile" | "openFolder") => Promise<void> }) {
  const progress = item.size > 0 ? Math.min(100, Math.round((item.downloaded / item.size) * 100)) : 0;
  const isPaused = item.status === "queued" || item.status === "paused";
  const isFinished = item.status === "completed" || item.status === "cancelled";
  const statusLabel = item.status === "paused" ? "Paused" : item.status === "completed" ? "Completed" : item.status === "failed" ? "Failed" : item.status === "cancelled" ? "Cancelled" : item.status === "queued" ? "Queued" : "Downloading";
  return (
    <article className="download-card">
      <div className="download-card__header">
        <div className="file-mark"><Archive size={18} /></div>
        <div className="download-card__identity">
          <strong>{item.name}</strong>
            <span>{item.url}</span>
          <span className="category-badge">{categoryLabels[item.category] ?? categoryLabels.other}</span>
        </div>
        {isFinished && <button className="icon-button" type="button" aria-label="Open download folder" onClick={() => void onAction("openFolder")}><FolderOpen size={17} /></button>}
        <button className="icon-button" type="button" aria-label="Delete download" onClick={() => void onAction("delete")}><Trash2 size={17} /></button>
      </div>
      <div className="download-card__progress-row"><span>{progress}%</span><span>{formatBytes(item.downloaded)} of {formatBytes(item.size)}</span></div>
      <div className="progress-track"><span style={{ width: `${progress}%` }} /></div>
      <div className="download-card__footer">
        <span className="status-pill"><span className={`status-dot status-dot--${item.status}`} />{statusLabel}</span>
        <span>{formatSpeed(item.speed)}</span><span>ETA {formatEta(item.etaSeconds)}</span><span>{item.connections} connections</span>
        {!isFinished && <button className="icon-button icon-button--small" type="button" aria-label={isPaused ? "Resume download" : "Pause download"} onClick={() => void onAction(isPaused ? "resume" : "pause")}>{isPaused ? <Play size={15} /> : <Pause size={15} />}</button>}
        {!isFinished && !isPaused && <button className="icon-button icon-button--small" type="button" aria-label="Cancel download" onClick={() => void onAction("cancel")}><X size={15} /></button>}
      </div>
    </article>
  );
}

function AddDownloadModal({ onClose, onSubmit }: { onClose: () => void; onSubmit: (url: string, destination: string) => Promise<void> }) {
  const [url, setUrl] = useState("");
  const [destination, setDestination] = useState("Auto");
  const [error, setError] = useState("");
  const [isSubmitting, setSubmitting] = useState(false);
  const submit = async () => {
    try {
      const parsed = new URL(url);
      if (!/^https?:$/.test(parsed.protocol)) throw new Error();
      setSubmitting(true);
      await onSubmit(url.trim(), destination);
    } catch (submissionError) {
      setError(submissionError instanceof Error ? submissionError.message : "The download could not be queued.");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <section className="modal" role="dialog" aria-modal="true" aria-labelledby="add-download-title">
        <div className="modal__header"><div><div className="section-kicker">NEW TRANSFER</div><h2 id="add-download-title">Add download</h2></div><button className="icon-button" type="button" onClick={onClose} aria-label="Close dialog"><X size={19} /></button></div>
        <label className="field-label" htmlFor="download-url">Download URL</label>
        <div className={`url-field ${error ? "url-field--error" : ""}`}><ExternalLink size={17} /><input id="download-url" autoFocus value={url} onChange={(event) => { setUrl(event.target.value); setError(""); }} placeholder="https://example.com/file.zip" onKeyDown={(event) => event.key === "Enter" && void submit()} /></div>
        {error && <p className="field-error">{error}</p>}
        <label className="field-label" htmlFor="destination">Save to</label>
        <div className="select-field"><FolderOpen size={17} /><select id="destination" value={destination} onChange={(event) => setDestination(event.target.value)}><option value="Auto">Automatic by category</option><option value="Downloads">Downloads</option><option value="Desktop">Desktop</option><option value="Documents">Documents</option></select><ChevronDown size={16} /></div>
        <div className="modal__note"><ShieldCheck size={16} /><span>URL validation and file safety checks will run in the Rust engine.</span></div>
        <div className="modal__actions"><button className="button button--ghost" type="button" onClick={onClose} disabled={isSubmitting}>Cancel</button><button className="button button--primary" type="button" onClick={() => void submit()} disabled={isSubmitting}><ArrowDownToLine size={16} /> {isSubmitting ? "Validating…" : "Start download"}</button></div>
      </section>
    </div>
  );
}

function SettingsPanel({ theme, onTheme, notificationsEnabled, onNotifications, onClose, onToast }: { theme: "midnight" | "graphite" | "dawn"; onTheme: (theme: "midnight" | "graphite" | "dawn") => void; notificationsEnabled: boolean; onNotifications: (enabled: boolean) => void; onClose: () => void; onToast: (message: string) => void }) {
  const [maxConcurrent, setMaxConcurrent] = useState("3");
  const [maxConnectionsPerDownload, setMaxConnectionsPerDownload] = useState("8");
  const [autoStart, setAutoStart] = useState(true);
  const [startOnLaunch, setStartOnLaunch] = useState(false);
  const [speedLimit, setSpeedLimit] = useState("0");
  const [defaultDestination, setDefaultDestination] = useState("Downloads");
  const [categoryRoutes, setCategoryRoutes] = useState<Record<string, string>>({
    archives: "Downloads", audio: "Downloads", video: "Downloads", images: "Downloads", documents: "Documents", applications: "Downloads", other: "Downloads",
  });
  const [saving, setSaving] = useState(false);
  const routedCategories = ["archives", "audio", "video", "images", "documents", "applications", "other"];

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    const keys = ["max_concurrent_downloads", "max_connections_per_download", "auto_start_downloads", "start_on_launch", "global_speed_limit_bps", "default_destination", ...routedCategories.map((category) => `category_folder_${category}`)];
    void Promise.all(keys.map((key) => invoke<string | null>("get_setting", { key }))).then((values) => {
      const settings = Object.fromEntries(keys.map((key, index) => [key, values[index]]));
      if (settings.max_concurrent_downloads) setMaxConcurrent(settings.max_concurrent_downloads);
      if (settings.max_connections_per_download) setMaxConnectionsPerDownload(settings.max_connections_per_download);
      if (settings.auto_start_downloads) setAutoStart(settings.auto_start_downloads !== "false");
      if (settings.start_on_launch) setStartOnLaunch(settings.start_on_launch === "true");
      if (settings.global_speed_limit_bps) setSpeedLimit(settings.global_speed_limit_bps);
      if (settings.default_destination) setDefaultDestination(settings.default_destination);
      setCategoryRoutes(Object.fromEntries(routedCategories.map((category) => [category, settings[`category_folder_${category}`] ?? (category === "documents" ? "Documents" : "Downloads")] )));
    }).catch(() => undefined);
  }, []);

  const save = async () => {
    const parsed = Number(maxConcurrent);
    const parsedSpeedLimit = Number(speedLimit);
    if (!Number.isInteger(parsed) || parsed < 1 || parsed > 32) {
      onToast("Concurrent downloads must be between 1 and 32.");
      return;
    }
    if (!Number.isSafeInteger(parsedSpeedLimit) || parsedSpeedLimit < 0 || parsedSpeedLimit > 1_000_000_000) {
      onToast("Speed limit must be 0 (unlimited) or between 1 and 1,000,000,000 B/s.");
      return;
    }
    const parsedConnections = Number(maxConnectionsPerDownload);
    if (!Number.isInteger(parsedConnections) || parsedConnections < 1 || parsedConnections > 32) {
      onToast("Connections per download must be between 1 and 32.");
      return;
    }
    setSaving(true);
    try {
      if ("__TAURI_INTERNALS__" in window) {
        const settings = {
          max_concurrent_downloads: String(parsed), max_connections_per_download: String(parsedConnections), auto_start_downloads: String(autoStart), start_on_launch: String(startOnLaunch), global_speed_limit_bps: String(parsedSpeedLimit), notifications_enabled: String(notificationsEnabled), default_destination: defaultDestination,
          ...Object.fromEntries(routedCategories.map((category) => [`category_folder_${category}`, categoryRoutes[category]])),
        };
        for (const [key, value] of Object.entries(settings)) await invoke("set_setting", { key, value });
      }
      onToast("Settings saved.");
      onClose();
    } catch (error) {
      onToast(error instanceof Error ? error.message : "Settings could not be saved.");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal-backdrop" role="presentation" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <section className="modal settings-panel" role="dialog" aria-modal="true" aria-labelledby="settings-title">
        <div className="modal__header"><div><div className="section-kicker">WORKSPACE CONFIGURATION</div><h2 id="settings-title">Settings</h2></div><button className="icon-button" type="button" onClick={onClose} aria-label="Close settings"><X size={19} /></button></div>
        <div className="settings-section"><div className="settings-section__heading"><FolderOpen size={16} /><div><strong>General</strong><small>Choose where new downloads are routed by default.</small></div></div><label className="field-label" htmlFor="default-destination">Default destination</label><select className="settings-input" id="default-destination" value={defaultDestination} onChange={(event) => setDefaultDestination(event.target.value)}><option>Downloads</option><option>Desktop</option><option>Documents</option></select><label className="settings-toggle"><input type="checkbox" checked={startOnLaunch} onChange={(event) => setStartOnLaunch(event.target.checked)} /><span>Start ZYNERO with Windows</span></label></div>
        <div className="settings-section"><div className="settings-section__heading"><Gauge size={16} /><div><strong>Connections</strong><small>Control concurrency without changing the real worker.</small></div></div><label className="field-label" htmlFor="max-connections">Maximum connections per download</label><input className="settings-input" id="max-connections" type="number" min="1" max="32" value={maxConnectionsPerDownload} onChange={(event) => setMaxConnectionsPerDownload(event.target.value)} /><small className="settings-help">Used by segmented downloads when the server supports byte ranges.</small></div>
        <div className="settings-section"><div className="settings-section__heading"><FolderOpen size={16} /><div><strong>Category routing</strong><small>Each file type can use its own Windows known folder.</small></div></div><div className="category-routing-grid">{routedCategories.map((category) => <label className="field-label" key={category}>{categoryLabels[category]}<select className="settings-input" value={categoryRoutes[category]} onChange={(event) => setCategoryRoutes((current) => ({ ...current, [category]: event.target.value }))}><option>Downloads</option><option>Desktop</option><option>Documents</option></select></label>)}</div><small className="settings-help">Choose Auto in Add download to apply these rules.</small></div>
        <div className="settings-section"><div className="settings-section__heading"><ShieldCheck size={16} /><div><strong>Privacy</strong><small>Local-only behavior and optional alerts.</small></div></div><small className="settings-help">ZYNERO stores download metadata locally in SQLite. No download URLs are sent to a remote service.</small></div>
        <div className="settings-section"><div className="settings-section__heading"><Settings size={16} /><div><strong>Advanced</strong><small>Low-level transfer defaults.</small></div></div><small className="settings-help">Settings are validated in Rust and persisted before they affect future queue runs.</small></div>
        <div className="settings-section settings-section--notifications" role="group" aria-labelledby="notifications-heading"><div className="settings-section__heading"><ShieldCheck size={16} /><div><strong id="notifications-heading">Notifications</strong><small>Control completion and failure alerts.</small></div><span className={`settings-status ${notificationsEnabled ? "settings-status--on" : "settings-status--off"}`}>{notificationsEnabled ? "ON" : "OFF"}</span></div><label className="settings-toggle"><input aria-label="Enable download notifications" type="checkbox" checked={notificationsEnabled} onChange={(event) => onNotifications(event.target.checked)} /><span>Notify when downloads complete or fail</span></label><small className="settings-help">This setting is saved with the other workspace settings.</small></div>
        <div className="settings-section"><div className="settings-section__heading"><Settings size={16} /><div><strong>Downloads</strong><small>Control the local transfer engine.</small></div></div><label className="field-label" htmlFor="max-concurrent">Maximum concurrent downloads</label><input className="settings-input" id="max-concurrent" type="number" min="1" max="32" value={maxConcurrent} onChange={(event) => setMaxConcurrent(event.target.value)} /><label className="field-label" htmlFor="speed-limit">Global speed limit (B/s)</label><input className="settings-input" id="speed-limit" type="number" min="0" max="1000000000" value={speedLimit} onChange={(event) => setSpeedLimit(event.target.value)} /><small className="settings-help">Use 0 for unlimited. The limit is shared across all active segment workers.</small><label className="settings-toggle"><input type="checkbox" checked={autoStart} onChange={(event) => setAutoStart(event.target.checked)} /><span>Start queued downloads automatically</span></label></div>
        <div className="settings-section"><div className="settings-section__heading"><Palette size={16} /><div><strong>Appearance</strong><small>Choose a persistent workspace theme.</small></div></div><div className="theme-options">{(["midnight", "graphite", "dawn"] as const).map((option) => <button key={option} type="button" className={`theme-option ${theme === option ? "theme-option--active" : ""}`} onClick={() => onTheme(option)}><span className={`theme-option__swatch theme-option__swatch--${option}`} /><span>{option[0].toUpperCase() + option.slice(1)}</span></button>)}</div></div>
        <div className="modal__actions"><button className="button button--ghost" type="button" onClick={onClose} disabled={saving}>Cancel</button><button className="button button--primary" type="button" onClick={() => void save()} disabled={saving}>{saving ? "Saving…" : "Save settings"}</button></div>
      </section>
    </div>
  );
}

function App() {
  const [activeNav, setActiveNav] = useState("Downloads");
  const [theme, setTheme] = useState<"midnight" | "graphite" | "dawn">(() => {
    const saved = window.localStorage.getItem("zynero-theme");
    if (saved === "midnight" || saved === "graphite" || saved === "dawn") return saved;
    return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "dawn" : "midnight";
  });
  const [isModalOpen, setModalOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("all");
  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const [toast, setToast] = useState("");
  const [isSettingsOpen, setSettingsOpen] = useState(false);
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void invoke<string | null>("get_setting", { key: "notifications_enabled" }).then((value) => setNotificationsEnabled(value !== "false")).catch(() => undefined);
    void invoke<StoredDownload[]>("get_downloads")
      .then((rows) => setDownloads(rows.map((row) => ({ id: row.id, name: row.filename, url: row.url, size: row.totalBytes ?? 0, downloaded: row.downloadedBytes, speed: row.speedBps ?? 0, etaSeconds: row.etaSeconds ?? 0, status: row.status === "active" ? "active" : row.status === "completed" ? "completed" : row.status === "failed" ? "failed" : row.status === "paused" ? "paused" : row.status === "cancelled" ? "cancelled" : "queued", connections: 1, destination: row.destination, category: row.category ?? "other" }))))
      .catch(() => setToast("Could not load downloads from SQLite."));
  }, []);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    const refresh = () => void invoke<StoredDownload[]>("get_downloads").then((rows) => setDownloads(rows.map((row) => ({ id: row.id, name: row.filename, url: row.url, size: row.totalBytes ?? 0, downloaded: row.downloadedBytes, speed: row.speedBps ?? 0, etaSeconds: row.etaSeconds ?? 0, status: row.status === "active" ? "active" : row.status === "completed" ? "completed" : row.status === "failed" ? "failed" : row.status === "paused" ? "paused" : row.status === "cancelled" ? "cancelled" : "queued", connections: 1, destination: row.destination, category: row.category ?? "other" }))));
    const interval = window.setInterval(refresh, 1000);
    return () => window.clearInterval(interval);
  }, []);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    let unlisten: (() => void) | undefined;
    void listen<StoredDownload>("download-progress", (event) => {
      const row = event.payload;
      void notifyIfNeeded(row, notificationsEnabled);
      setDownloads((current) => {
        const next = { id: row.id, name: row.filename, url: row.url, size: row.totalBytes ?? 0, downloaded: row.downloadedBytes, speed: row.speedBps ?? 0, etaSeconds: row.etaSeconds ?? 0, status: row.status === "active" ? "active" : row.status === "completed" ? "completed" : row.status === "failed" ? "failed" : row.status === "paused" ? "paused" : row.status === "cancelled" ? "cancelled" : "queued", connections: 1, destination: row.destination, category: row.category ?? "other" } as DownloadItem;
        return current.some((item) => item.id === row.id) ? current.map((item) => item.id === row.id ? next : item) : [next, ...current];
      });
    }).then((remove) => { unlisten = remove; });
    return () => { unlisten?.(); };
  }, [notificationsEnabled]);

  useEffect(() => { window.localStorage.setItem("zynero-theme", theme); }, [theme]);

  const filteredDownloads = useMemo(() => {
    const query = search.toLowerCase();
    return downloads.filter((item) => {
      const matchesSearch = item.name.toLowerCase().includes(query) || item.url.toLowerCase().includes(query);
      const matchesCategory = categoryFilter === "all" || item.category === categoryFilter;
      const matchesView = activeNav === "Downloads" || activeNav === "History" || (activeNav === "Active" && item.status === "active") || (activeNav === "Completed" && item.status === "completed") || (activeNav === "Queued" && (item.status === "queued" || item.status === "paused")) || (activeNav === "Failed" && item.status === "failed") || activeNav === "Scheduled";
      return matchesSearch && matchesCategory && matchesView;
    });
  }, [activeNav, categoryFilter, downloads, search]);
  const mapStoredDownload = (row: StoredDownload): DownloadItem => ({
    id: row.id, name: row.filename, url: row.url, size: row.totalBytes ?? 0, downloaded: row.downloadedBytes,
    speed: row.speedBps ?? 0, etaSeconds: row.etaSeconds ?? 0,
    status: row.status === "active" ? "active" : row.status === "completed" ? "completed" : row.status === "failed" ? "failed" : row.status === "paused" ? "paused" : row.status === "cancelled" ? "cancelled" : "queued",
    connections: 1, destination: row.destination, category: row.category ?? "other",
  });
  const handleAction = async (id: string, action: "pause" | "resume" | "cancel" | "delete" | "openFile" | "openFolder") => {
    if (!("__TAURI_INTERNALS__" in window)) throw new Error("Open ZYNERO desktop to control downloads.");
    const command = action === "pause" ? "pause_download" : action === "resume" ? "resume_download" : action === "cancel" ? "cancel_download" : action === "delete" ? "delete_download" : action === "openFile" ? "open_download_file" : "open_download_folder";
    await invoke(command, { id });
    if (action === "openFile" || action === "openFolder") { setToast(action === "openFile" ? "Opened downloaded file." : "Opened download folder."); return; }
    if (action === "delete") setDownloads((current) => current.filter((item) => item.id !== id));
    else if (action === "pause") setDownloads((current) => current.map((item) => item.id === id ? { ...item, status: "paused" } : item));
    else if (action === "cancel") setDownloads((current) => current.map((item) => item.id === id ? { ...item, status: "cancelled" } : item));
    else { const rows = await invoke<StoredDownload[]>("get_downloads"); setDownloads(rows.map(mapStoredDownload)); }
  };
  const handleSubmit = async (url: string, destination: string) => {
    if (!("__TAURI_INTERNALS__" in window)) {
      throw new Error("Open ZYNERO desktop to start a real download.");
    }
    const result = await invoke<{ id: string; url: string; filename: string; destination: string; status: DownloadStatus; totalBytes: number | null; category?: string }>("add_download", {
      request: { url, destination },
    });
    setDownloads((current) => [{ id: result.id, name: result.filename, url: result.url, size: result.totalBytes ?? 0, downloaded: 0, speed: 0, etaSeconds: 0, status: result.status, connections: 1, destination: result.destination, category: (result as { category?: string }).category ?? "other" }, ...current]);
    setModalOpen(false);
    setToast("Download queued by the Rust engine.");
    window.setTimeout(() => setToast(""), 4200);
  };

  return (
    <main className={`app-shell theme-${theme}`}>
      <aside className="sidebar">
        <div className="brand-lockup"><div className="brand-mark"><Sparkles size={17} /></div><div><div className="brand">ZYNERO</div><div className="tagline">Download. Faster. Smarter.</div></div></div>
        <div className="workspace-label">WORKSPACE</div>
        <nav className="main-nav" aria-label="Primary navigation">
          {navItems.map(({ label, icon: Icon }) => <button key={label} className={`nav-item ${activeNav === label ? "nav-item--active" : ""}`} type="button" onClick={() => setActiveNav(label)}><Icon size={17} /><span>{label}</span>{label === "Downloads" && <span className="nav-count">{downloads.length}</span>}</button>)}
        </nav>
        <div className="sidebar-bottom"><div className="storage-card"><div className="storage-card__title"><span>Storage</span><span>Ready</span></div><div className="storage-bar"><span /></div><small>Local files only</small></div><button className="nav-item" type="button" onClick={() => setSettingsOpen(true)}><Settings size={17} /><span>Settings</span></button><div className="version">ZYNERO v0.1.0 · MVP</div></div>
      </aside>
      <section className="content">
        <header className="topbar"><div><div className="section-kicker">DOWNLOAD MANAGER <span className="live-indicator"><span /> LOCAL ENGINE</span></div><h1>{activeNav}</h1><p className="page-subtitle">Keep every transfer organized and moving.</p></div><button className="button button--primary" type="button" onClick={() => setModalOpen(true)}><Plus size={17} /> Add download</button></header>
        <section className="stats-grid"><StatCard label="Total downloads" value={`${downloads.length}`} detail="Across your workspace" icon={Download} accent="#74c9ff" /><StatCard label="Active speed" value={formatSpeed(downloads.filter((item) => item.status === "active").reduce((sum, item) => sum + item.speed, 0))} detail="Live from download workers" icon={Gauge} accent="#9ee7bd" /><StatCard label="Completed" value={`${downloads.filter((item) => item.status === "completed").length}`} detail="Finished transfers" icon={Check} accent="#c8b6ff" /><StatCard label="Scheduled" value="0" detail="No scheduled transfers" icon={Clock3} accent="#f4c46e" /></section>
        <div className="toolbar"><div className="search-field"><Search size={16} /><input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search downloads" aria-label="Search downloads" /></div>          <div className="toolbar-actions"><label className="category-filter"><ListFilter size={15} /><select value={categoryFilter} onChange={(event) => setCategoryFilter(event.target.value)} aria-label="Filter by category">{Object.entries(categoryLabels).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</select></label><div className="theme-picker" aria-label="Theme"><Palette size={15} /><button className={theme === "midnight" ? "theme-swatch theme-swatch--active" : "theme-swatch"} type="button" onClick={() => setTheme("midnight")} aria-label="Midnight theme" /><button className={theme === "graphite" ? "theme-swatch theme-swatch--active theme-swatch--graphite" : "theme-swatch theme-swatch--graphite"} type="button" onClick={() => setTheme("graphite")} aria-label="Graphite theme" /><button className={theme === "dawn" ? "theme-swatch theme-swatch--active theme-swatch--dawn" : "theme-swatch theme-swatch--dawn"} type="button" onClick={() => setTheme("dawn")} aria-label="Dawn theme" /></div></div></div>
        {filteredDownloads.length ? <section className="download-list" aria-label="Downloads">{filteredDownloads.map((item) => <DownloadCard key={item.id} item={item} onAction={(action) => handleAction(item.id, action)} />)}</section> : <EmptyDownloads onAdd={() => setModalOpen(true)} />}
        <footer className="content-footer"><span><span className="footer-dot" /> Secure local workspace</span><span>Backend status: <strong>Ready for IPC</strong></span></footer>
      </section>
      {toast && <div className="toast"><Check size={16} />{toast}<button type="button" onClick={() => setToast("")} aria-label="Dismiss notification"><X size={14} /></button></div>}
      {isModalOpen && <AddDownloadModal onClose={() => setModalOpen(false)} onSubmit={handleSubmit} />}
      {isSettingsOpen && <SettingsPanel theme={theme} onTheme={setTheme} notificationsEnabled={notificationsEnabled} onNotifications={setNotificationsEnabled} onClose={() => setSettingsOpen(false)} onToast={(message) => { setToast(message); window.setTimeout(() => setToast(""), 4200); }} />}
    </main>
  );
}

export default App;

const rootElement = document.getElementById("root");
if (!rootElement) throw new Error("ZYNERO root element was not found.");
createRoot(rootElement).render(<App />);
