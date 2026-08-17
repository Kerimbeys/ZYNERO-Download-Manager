import React from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";

function App() {
  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">ZYNERO</div>
        <div className="tagline">Download. Faster. Smarter.</div>
        <nav aria-label="Primary navigation">
          <a className="active" href="#downloads">Downloads</a>
          <a href="#active">Active</a>
          <a href="#completed">Completed</a>
          <a href="#queued">Queued</a>
          <a href="#scheduled">Scheduled</a>
          <a href="#failed">Failed</a>
          <a href="#history">History</a>
        </nav>
        <a className="settings-link" href="#settings">Settings</a>
      </aside>
      <section className="content">
        <header className="topbar">
          <div>
            <p className="eyebrow">DOWNLOAD MANAGER</p>
            <h1>Downloads</h1>
          </div>
          <button type="button" disabled title="Rust IPC will be connected in the next task">
            Add download
          </button>
        </header>
        <section className="empty-state" aria-live="polite">
          <h2>Ready for your first download</h2>
          <p>The interface shell is ready. Download data will come from the Rust engine through secure Tauri events.</p>
        </section>
      </section>
    </main>
  );
}

createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
