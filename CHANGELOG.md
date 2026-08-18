# Changelog

Bu dosya ZYNERO'nun kullanıcıya görünen ve release açısından önemli değişikliklerini kaydeder. Henüz yayınlanmamış çalışmalar, doğrulama kanıtı oluşana kadar `Unreleased` bölümünde tutulur.

## [Unreleased]

### In progress

- D01/D02 kapsamında bounded concurrent segment worker'ları, ayrı temporary segment dosyaları, aggregate progress ve ordered merge akışı geliştiriliyor.
- D03/D04 kapsamında Content-Range, segment uzunluğu, merged output bütünlüğü ve Range unsupported single-stream fallback testleri ekleniyor; Windows test kanıtı bekleyen maddeler TODO.md'de partial tutuluyor.
- A07 kalite komutları, E08 UI testleri, F03-F05 bildirim/ayar/kategori akışları ve G01-G06 güvenlik hardening çalışmaları sıraya alındı.

### Completed in current development line

- A04 shared TypeScript/Rust domain modelleri ve typed IPC sözleşmesi.
- A05 dar kapsamlı Tauri capability policy; gereksiz `core:default` izinleri kaldırıldı.
- SQLite downloads, segments, queues ve settings migrations ile repository CRUD ve startup recovery.
- Gerçek streaming HTTP worker, retry/backoff, progress/speed/ETA, pause/resume/cancel ve queue auto-start.
- React dashboard'un gerçek SQLite/Tauri verileriyle beslenmesi, tema desteği ve lifecycle eylemleri.

## [0.1.0] - Release candidate

### Added

- Windows Tauri 2.x desktop shell ve React/Vite dashboard.
- HTTP/HTTPS URL validation, metadata inspection ve güvenli filename/destination kontrolleri.
- `.zynero.part` temporary dosya akışı ve restart sonrası recovery.
- SQLite state persistence, migrations, download queue ve schedule window desteği.
- Windows NSIS ve MSI installer paketleri.
- Local HTTP testleri: retry, resumable Range 206, pause state, 404 failure ve temp path davranışları.

### Security

- Minimal default capability: yalnızca main window, core event ve kullanılan notification izinleri.
- Path traversal, embedded credential ve unsafe scheme reddi.
- Hassas URL credential'larının hata/log çıktısına taşınmaması için hata sınıflandırması.

### Known limitations

- Windows code signing ve signed update henüz tamamlanmış değildir.
- GitHub Release asset yüklemesi önceki API permission engeli nedeniyle ayrıca doğrulanmalıdır.
- Çoklu bağlantı pause/cancel recovery, global speed limiter, UI component testleri, i18n ve browser extension kapsamı tamamlanmamıştır.

[Unreleased]: https://github.com/Kerimbeys/ZYNERO-Download-Manager/compare/main...HEAD
