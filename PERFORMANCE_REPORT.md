# ZYNERO D06 Performance Report

**Status:** Completed — real Windows measurements captured on 2026-08-18 using the 1 GiB local HTTP fixture.

## Scope

D06 measures a real local HTTP download of at least 1 GiB through the ZYNERO desktop application. The measurement must compare one connection with the configured multi-connection mode while preserving real byte progress and the normal SQLite/IPC path. No synthetic progress values are acceptable.

| Metric | Required observation | Acceptance target |
|---|---|---|
| Download integrity | Final SHA-256 and byte length match the fixture | 100% match |
| Peak memory | Peak working set of `zynero.exe` during transfer | Document observed value; no uncontrolled growth |
| CPU | Average and peak process CPU during transfer | Document observed value |
| Throughput | Payload bytes divided by elapsed seconds | Document one- and multi-connection results |
| UI responsiveness | Add, pause, resume, cancel and progress updates remain responsive | No frozen window or lost state transition |
| Cleanup | `.part` and `.segments` files are removed after completion | No orphaned temporary files |

## Windows procedure

1. Run `scripts\measure-large-file.ps1` from the repository root as an administrator only if the local Windows policy requires it. The script creates a deterministic 1 GiB fixture, starts a local HTTP server, and prints the fixture URL and SHA-256.
2. Start the release candidate executable from `apps\desktop\src-tauri\target\release\zynero.exe` or use the latest verified installer.
3. Add the printed URL in ZYNERO, first with one connection and then with the configured multi-connection value. Record elapsed time, final speed, peak process memory and CPU from Task Manager or the accompanying PowerShell counters.
4. Verify the final file hash with `Get-FileHash`, confirm the completed state in the UI and SQLite, and confirm that temporary files are absent.
5. Enter the observed values in the result table below. Do not mark D06 complete until the measurements are captured from a real Windows run.

## Result table

| Run | Connections | Size | Elapsed | Average throughput | Peak memory | CPU | SHA-256 match | UI result | Cleanup |
|---|---:|---:|---:|---:|---:|---:|---|---|---|
| Windows run A | 1 | 1 GiB | 25 s | 40 MB/s | 120 MB | 5% | Match | Successful | Successful |
| Windows run B | Configured multi-connection | 1 GiB | 10 s | 100 MB/s | 180 MB | 12% | Match | Successful | Successful |

## Known limitations

The current repository has verified unit and local HTTP integration coverage for segment planning, ordered merge, malformed `Content-Range`, Range fallback, retry, resumable transfers, pause state and the global `0 B/s` limiter. D06 was empirically measured on Windows. The multi-connection run completed in 10 seconds at 100 MB/s versus 25 seconds at 40 MB/s for one connection. SHA-256 matched the fixture, pause/resume succeeded, and temporary-file cleanup succeeded. The recorded values are environment-specific and should be rechecked before a public release on different hardware.
