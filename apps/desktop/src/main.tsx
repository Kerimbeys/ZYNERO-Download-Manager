import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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

type DownloadStatus = "active" | "queued" | "completed" | "failed";

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
};

type NavItem = {
  label: string;
  icon: typeof Download;
  count?: number;
};

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

function DownloadCard({ item }: { item: DownloadItem }) {
  const progress = item.size > 0 ? Math.round((item.downloaded / item.size) * 100) : 0;
  const isPaused = item.status === "queued";
  return (
    <article className="download-card">
      <div className="download-card__header">
        <div className="file-mark"><Archive size={18} /></div>
        <div className="download-card__identity">
          <strong>{item.name}</strong>
          <span>{item.url}</span>
        </div>
        <button className="icon-button" type="button" aria-label="More download actions"><MoreHorizontal size={19} /></button>
      </div>
      <div className="download-card__progress-row"><span>{progress}%</span><span>{formatBytes(item.downloaded)} of {formatBytes(item.size)}</span></div>
      <div className="progress-track"><span style={{ width: `${progress}%` }} /></div>
      <div className="download-card__footer">
        <span className="status-pill"><span className={`status-dot status-dot--${item.status}`} />{isPaused ? "Queued" : "Downloading"}</span>
        <span>{formatSpeed(item.speed)}</span><span>ETA {formatEta(item.etaSeconds)}</span><span>{item.connections} connections</span>
        <button className="icon-button icon-button--small" type="button" aria-label={isPaused ? "Resume download" : "Pause download"}>{isPaused ? <Play size={15} /> : <Pause size={15} />}</button>
      </div>
    </article>
  );
}

function AddDownloadModal({ onClose, onSubmit }: { onClose: () => void; onSubmit: (url: string, destination: string) => Promise<void> }) {
  const [url, setUrl] = useState("");
  const [destination, setDestination] = useState("Downloads");
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
        <div className="select-field"><FolderOpen size={17} /><select id="destination" value={destination} onChange={(event) => setDestination(event.target.value)}><option>Downloads</option><option>Desktop</option><option>Documents</option></select><ChevronDown size={16} /></div>
        <div className="modal__note"><ShieldCheck size={16} /><span>URL validation and file safety checks will run in the Rust engine.</span></div>
        <div className="modal__actions"><button className="button button--ghost" type="button" onClick={onClose} disabled={isSubmitting}>Cancel</button><button className="button button--primary" type="button" onClick={() => void submit()} disabled={isSubmitting}><ArrowDownToLine size={16} /> {isSubmitting ? "Validating…" : "Start download"}</button></div>
      </section>
    </div>
  );
}

function App() {
  const [activeNav, setActiveNav] = useState("Downloads");
  const [theme, setTheme] = useState<"midnight" | "graphite" | "dawn">("midnight");
  const [isModalOpen, setModalOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const [toast, setToast] = useState("");

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    void invoke<StoredDownload[]>("get_downloads")
      .then((rows) => setDownloads(rows.map((row) => ({
        id: row.id,
        name: row.filename,
        url: row.url,
        size: row.totalBytes ?? 0,
        downloaded: row.downloadedBytes,
        speed: 0,
        etaSeconds: 0,
        status: row.status === "active" ? "active" : row.status === "completed" ? "completed" : row.status === "failed" ? "failed" : "queued",
        connections: 1,
        destination: row.destination,
      }))))
      .catch(() => setToast("Could not load downloads from SQLite."));
  }, []);

  const filteredDownloads = useMemo(() => downloads.filter((item) => item.name.toLowerCase().includes(search.toLowerCase())), [downloads, search]);
  const handleSubmit = async (url: string, destination: string) => {
    if (!("__TAURI_INTERNALS__" in window)) {
      throw new Error("Open ZYNERO desktop to start a real download.");
    }
    const result = await invoke<{ id: string; url: string; filename: string; destination: string; status: DownloadStatus; totalBytes: number | null }>("add_download", {
      request: { url, destination },
    });
    setDownloads((current) => [{ id: result.id, name: result.filename, url: result.url, size: result.totalBytes ?? 0, downloaded: 0, speed: 0, etaSeconds: 0, status: result.status, connections: 1, destination: result.destination }, ...current]);
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
        <div className="sidebar-bottom"><div className="storage-card"><div className="storage-card__title"><span>Storage</span><span>Ready</span></div><div className="storage-bar"><span /></div><small>Local files only</small></div><button className="nav-item" type="button"><Settings size={17} /><span>Settings</span></button><div className="version">ZYNERO v0.1.0 · MVP</div></div>
      </aside>
      <section className="content">
        <header className="topbar"><div><div className="section-kicker">DOWNLOAD MANAGER <span className="live-indicator"><span /> LOCAL ENGINE</span></div><h1>{activeNav}</h1><p className="page-subtitle">Keep every transfer organized and moving.</p></div><button className="button button--primary" type="button" onClick={() => setModalOpen(true)}><Plus size={17} /> Add download</button></header>
        <section className="stats-grid"><StatCard label="Total downloads" value={`${downloads.length}`} detail="Across your workspace" icon={Download} accent="#74c9ff" /><StatCard label="Active speed" value="—" detail="Waiting for Rust engine" icon={Gauge} accent="#9ee7bd" /><StatCard label="Completed" value="0" detail="Nothing finished yet" icon={Check} accent="#c8b6ff" /><StatCard label="Scheduled" value="0" detail="No scheduled transfers" icon={Clock3} accent="#f4c46e" /></section>
        <div className="toolbar"><div className="search-field"><Search size={16} /><input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search downloads" aria-label="Search downloads" /></div><div className="toolbar-actions"><div className="theme-picker" aria-label="Theme"><Palette size={15} /><button className={theme === "midnight" ? "theme-swatch theme-swatch--active" : "theme-swatch"} type="button" onClick={() => setTheme("midnight")} aria-label="Midnight theme" /><button className={theme === "graphite" ? "theme-swatch theme-swatch--active theme-swatch--graphite" : "theme-swatch theme-swatch--graphite"} type="button" onClick={() => setTheme("graphite")} aria-label="Graphite theme" /><button className={theme === "dawn" ? "theme-swatch theme-swatch--active theme-swatch--dawn" : "theme-swatch theme-swatch--dawn"} type="button" onClick={() => setTheme("dawn")} aria-label="Dawn theme" /></div><button className="toolbar-button" type="button"><ListFilter size={16} /> Filter <ChevronDown size={14} /></button></div></div>
        {filteredDownloads.length ? <section className="download-list" aria-label="Downloads">{filteredDownloads.map((item) => <DownloadCard key={item.id} item={item} />)}</section> : <EmptyDownloads onAdd={() => setModalOpen(true)} />}
        <footer className="content-footer"><span><span className="footer-dot" /> Secure local workspace</span><span>Backend status: <strong>Ready for IPC</strong></span></footer>
      </section>
      {toast && <div className="toast"><Check size={16} />{toast}<button type="button" onClick={() => setToast("")} aria-label="Dismiss notification"><X size={14} /></button></div>}
      {isModalOpen && <AddDownloadModal onClose={() => setModalOpen(false)} onSubmit={handleSubmit} />}
    </main>
  );
}

export default App;
