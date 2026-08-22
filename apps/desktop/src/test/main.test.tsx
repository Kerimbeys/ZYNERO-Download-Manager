import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { App } from "../main";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(() => Promise.resolve(() => undefined)) }));
vi.mock("@tauri-apps/plugin-notification", () => ({ isPermissionGranted: vi.fn(() => Promise.resolve(false)), requestPermission: vi.fn(() => Promise.resolve("denied")), sendNotification: vi.fn() }));
const invokeMock = vi.mocked(invoke);
const row = { id: "download-1", url: "https://example.com/archive.zip", filename: "archive.zip", destination: "Downloads", status: "active", totalBytes: 1000, downloadedBytes: 250, contentType: "application/zip", supportsRange: true, tempPath: null, finalPath: null, errorMessage: null, speedBps: 100, category: "archives", etaSeconds: 8 };
const completedRow = { ...row, id: "download-2", filename: "verified.zip", status: "completed", downloadedBytes: 1000, finalPath: "C:/Downloads/verified.zip" };

beforeEach(() => {
  Object.defineProperty(window, "__TAURI_INTERNALS__", { value: {}, configurable: true });
  invokeMock.mockImplementation(async (command, args) => {
    const input = args as { key?: string } | undefined;
    if (command === "get_downloads") return [row, completedRow];
    if (command === "get_setting") return input?.key === "notifications_enabled" ? "false" : null;
    if (command === "add_download") return { ...row, id: "download-3", status: "queued", filename: "new.zip", downloadedBytes: 0 };
    if (command === "verify_download_hash") return true;
    return null;
  });
});

describe("ZYNERO desktop critical UI flows", () => {
  it("queues an Add Download request with Auto destination", async () => {
    render(<App />);
    await screen.findByText("archive.zip");
    fireEvent.click(screen.getByRole("button", { name: /Add download/i }));
    fireEvent.change(screen.getByLabelText("Download URL"), { target: { value: "https://example.com/new.zip" } });
    fireEvent.click(screen.getByRole("button", { name: /Start download/i }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("add_download", { request: { url: "https://example.com/new.zip", destination: "Auto" } }));
  });

  it("calls pause, resume and delete through IPC", async () => {
    render(<App />);
    await screen.findByText("archive.zip");
    fireEvent.click(screen.getByRole("button", { name: "Pause download" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("pause_download", { id: "download-1" }));
    fireEvent.click(screen.getByRole("button", { name: "Resume download" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("resume_download", { id: "download-1" }));
    fireEvent.click(screen.getAllByRole("button", { name: "Delete download" })[0]);
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("delete_download", { id: "download-1" }));
  });

  it("verifies a completed download and shows the success state", async () => {
    render(<App />);
    await screen.findByText("verified.zip");
    fireEvent.click(screen.getByRole("button", { name: "Verify SHA-256" }));
    fireEvent.change(screen.getByLabelText("Expected SHA-256"), { target: { value: "a".repeat(64) } });
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("verify_download_hash", { id: "download-2", expectedSha256: "a".repeat(64) }));
    expect(await screen.findByRole("status")).toHaveTextContent("SHA-256 verified.");
  });

  it("shows a visible mismatch state when hash verification returns false", async () => {
    invokeMock.mockImplementation(async (command, args) => {
      if (command === "get_downloads") return [row, completedRow];
      if (command === "verify_download_hash") return false;
      const input = args as { key?: string } | undefined;
      if (command === "get_setting") return input?.key === "notifications_enabled" ? "false" : null;
      return null;
    });
    render(<App />);
    await screen.findByText("verified.zip");
    fireEvent.click(screen.getByRole("button", { name: "Verify SHA-256" }));
    fireEvent.change(screen.getByLabelText("Expected SHA-256"), { target: { value: "b".repeat(64) } });
    fireEvent.click(screen.getByRole("button", { name: "Verify" }));
    expect(await screen.findByRole("status")).toHaveTextContent("SHA-256 mismatch.");
  });

  it("shows the persisted notification setting and saves it", async () => {
    render(<App />);
    await screen.findByText("archive.zip");
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    const toggle = await screen.findByRole("checkbox", { name: "Enable download notifications" });
    expect(toggle).not.toBeChecked();
    fireEvent.click(toggle);
    fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("set_setting", { key: "notifications_enabled", value: "true" }));
  });
});
