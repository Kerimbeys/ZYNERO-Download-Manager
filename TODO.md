# ZYNERO TODO

**Durum işaretleri:** `[ ]` Bekliyor · `[-]` Devam ediyor · `[x]` Tamamlandı · `[!]` Engellendi

**Güncelleme kuralı:** Her görev, doğrulama komutu veya test sonucu ile birlikte işaretlenecek. Bir görev yalnızca Definition of Done koşulları sağlandığında `[x]` yapılacak.

## A. Başlangıç ve mimari

- [x] A01 — Repository çalışma alanını oluştur; `apps/desktop`, `apps/extension`, `packages/shared`, `packages/i18n`, `docs`, `scripts` ve `tests` dizinlerini ekle. Temel pnpm workspace ve React/Vite uygulama iskeleti oluşturuldu.
- [x] A02 — Tauri 2.x + React + TypeScript + Vite masaüstü iskeletini kur ve Windows hedefini doğrula. Rust 1.97.1, Tauri CLI 2.11.4, MSVC Build Tools ve WebView2 hazır; Tauri dev/release derleme doğrulandı.
- [x] A03 — Rust modül sınırlarını oluştur: `commands`, `download`, `database`, `scheduler`, `browser`, `security`, `utils`. Modül iskeletleri ve `lib.rs` bağlantıları eklendi.
- [ ] A04 — Ortak domain tiplerini tanımla: `DownloadStatus`, `Download`, `Segment`, progress payload ve IPC hata modeli.
- [ ] A05 — Güvenli Tauri capability/permission politikasını ve minimal IPC yüzeyini tanımla.
- [ ] A06 — `README.md`, `ARCHITECTURE.md`, `SECURITY.md`, `CONTRIBUTING.md` ve `CHANGELOG.md` başlangıç belgelerini yaz.
- [-] A07 — Format, lint, typecheck, Rust check/test ve build komutlarını çalışır hâle getir. Frontend typecheck/build ve `cargo check` çalışıyor; format/lint/test komutları sonraki adım.

## B. Kalıcı veri katmanı

- [x] B01 — SQLite bağlantı ve uygulama veri dizini yönetimini ekle. `DatabaseState` uygulama data directory altında `zynero.sqlite3` açıyor ve startup'ta yönetiliyor.
- [-] B02 — `downloads`, `segments`, `queues`, `settings` migrations oluştur. İlk `downloads` migration'ı ve status/created_at indexleri eklendi; segment/queue/settings şemaları sonraki migration'lara bırakıldı.
- [x] B03 — Repository katmanını yaz; frontend memory yerine gerçek CRUD kullan. `insert_download`, `list_downloads` ve `find_download` repository metotları eklendi; Dashboard `get_downloads` ile SQLite'dan besleniyor.
- [ ] B04 — Download state transition kurallarını tanımla ve unit testlerini yaz.
- [ ] B05 — Startup recovery için yarım kalan indirmeleri güvenli biçimde yükle.

## C. İlk gerçek indirme dilimi — en yüksek öncelik

- [x] C01 — URL doğrulama, protocol kontrolü ve güvenli filename çıkarımı. `add_download` command HTTP/HTTPS, host, embedded credential ve güvenli filename kontrollerini yapıyor.
- [x] C02 — HEAD/ilk GET metadata tespiti: content length, content type ve Range capability. `inspect_url` HEAD + `Range: bytes=0-0` fallback ile Content-Length/Content-Range/Content-Type/Accept-Ranges okuyor.
- [ ] C03 — Tek bağlantılı streaming HTTP/HTTPS download worker'ını yaz; büyük dosyayı RAM'e alma.
- [ ] C04 — Güvenli geçici dosya ve hedef path oluşturma; traversal ve overwrite kontrollerini ekle.
- [ ] C05 — Gerçek progress, speed ve ETA hesaplamasını hareketli ortalama ile ekle.
- [ ] C06 — Tauri command/event akışını bağla: add, get, pause, resume, cancel, delete, progress.
- [ ] C07 — Gerçek pause: request cancellation, offset persist ve SQLite state update.
- [ ] C08 — Gerçek resume: persisted offset'ten devam et; destek yoksa açık hata/fallback davranışı uygula.
- [ ] C09 — Retry/backoff, timeout, HTTP hata sınıfları ve insan okunabilir hata mesajlarını ekle.
- [ ] C10 — Local HTTP test server ile HTTP, pause/resume, restart recovery ve failure integration testlerini yaz.
- [ ] C11 — İlk dikey milestone'u uçtan uca doğrula ve `feat(core): add resumable downloads` commit'ini oluştur.

## D. Çoklu bağlantı ve performans

- [ ] D01 — Segment hesaplama ve 1–32 bağlantı sınırlarını ekle.
- [ ] D02 — Range destekli sunucular için concurrent segment worker'ları yaz.
- [ ] D03 — Segment dosyası yazma, merge ve bütünlük kontrollerini güvenli hâle getir.
- [ ] D04 — Range unsupported fallback ve hatalı Range response senaryolarını test et.
- [ ] D05 — Global speed limiter ekle; `0 = unlimited` davranışını doğrula.
- [ ] D06 — 1 GB+ test dosyalarında RAM, CPU, hız ve UI responsiveness ölçümü yap.

## E. React masaüstü arayüzü

- [x] E01 — Windows 11 esintili responsive shell ve sidebar oluştur. Responsive sidebar, workspace navigasyonu, storage kartı ve temel uygulama shell'i eklendi.
- [x] E02 — Dashboard'u yalnızca gerçek Rust/SQLite verileriyle besle. Uygulama açılışında `get_downloads` IPC çağrısı ile persisted kayıtlar yükleniyor; mock download kullanılmıyor.
- [x] E03 — Download card: progress, bytes, speed, ETA, connection count, status ve eylemler. Backend tipine hazır DownloadCard bileşeni ve durum görselleri eklendi.
- [x] E04 — Add Download penceresini gerçek `add_download` command'ına bağla. Frontend `invoke` çağrısı, Rust request/response modeli ve hata gösterimi eklendi.
- [ ] E05 — Active, Completed, Queued, Scheduled, Failed ve History görünümlerini ekle.
- [ ] E06 — Pause/resume/cancel/delete/open file/open folder eylemlerini IPC'ye bağla.
- [-] E07 — Light/dark/system tema desteğini ekle; aşırı gradient/glass kullanımından kaçın. Midnight, Graphite ve Dawn token varyantları ile tema seçici eklendi; system preference entegrasyonu sonraki adım.
- [ ] E08 — UI testleri: add, pause, resume, delete ve settings.

## F. Kuyruk, bildirim ve ayarlar

- [ ] F01 — Queue CRUD, priority, automatic start ve max concurrent downloads.
- [ ] F02 — Scheduler: başlangıç/duruş zamanı ve WAITING/QUEUED geçişleri.
- [ ] F03 — Windows completion/failure/queue notification'larını optional yap.
- [ ] F04 — General, Downloads, Connections, Notifications, Appearance, Privacy ve Advanced ayarlarını bağla.
- [ ] F05 — Dosya kategorileri ve extension mapping'i ekle.

## G. Güvenlik ve ürünleştirme

- [ ] G01 — HTTPS doğrulaması, safe URL parsing ve secrets redaction denetimi.
- [ ] G02 — Tauri permissions, IPC input validation ve filesystem root kısıtlarını gözden geçir.
- [ ] G03 — DEBUG/INFO/WARN/ERROR lokal loglama; token/cookie/auth header loglamama.
- [ ] G04 — SHA-256 hash verification ekle.
- [ ] G05 — Installer, Windows code signing ve signed update altyapısını planla/uygula.
- [ ] G06 — Startup <2 saniye, idle RAM <150 MB ve büyük dosya performans hedeflerini ölç.

## H. Tarayıcı ve uluslararasılaştırma

- [ ] H01 — Translation key sistemini ilk günden etkinleştir; hardcoded UI metinlerini kaldır.
- [ ] H02 — İngilizce kaynak locale ve Türkçe locale'i ekle.
- [ ] H03 — Alman, Fransız, İspanyol, Portekiz, İtalyan, Rus, Çin, Japon, Kore ve Arapça locale hazırlığını tamamla.
- [ ] H04 — WebExtension temel yapısını Chrome/Edge/Firefox için oluştur.
- [ ] H05 — Native Messaging host kaydı ve güvenli URL/filename/referrer aktarımı.
- [ ] H06 — Browser interception deneysel özelliğini uçtan uca test et.

## I. Release hazırlığı

- [ ] I01 — Full unit/integration/UI test matrisi ve CI çalıştırması.
- [ ] I02 — Security review ve threat model dokümanını tamamla.
- [ ] I03 — Changelog, migration policy ve release checklist oluştur.
- [ ] I04 — ZYNERO marka, domain, GitHub adı ve paket çakışmalarını lansman öncesi araştır.
- [ ] I05 — v0.1 release candidate üret; Windows 10/11 temiz makine smoke test'i yap.
- [ ] I06 — v0.2/v0.3/v1.0 kapsamını ayrı milestone'lara böl.

## J. Repository ve paylaşım

- [x] J01 — `Kerimbeys/ZYNERO-Download-Manager` GitHub repository'sini doğrula, yerel `main` branch'ini bağla ve ilk commit'leri pushla.

## K. Yeniden kullanılabilir beceri

- [x] K01 — ZYNERO Tauri/Rust/SQLite geliştirme sürecini reusable skill'e dönüştür; `/home/ubuntu/skills/zynero-download-manager-development/SKILL.md` oluştur ve doğrula.

## Günlük ilerleme kaydı

| Tarih | Görev | Sonuç | Doğrulama |
|---|---|---|---|
| 2026-08-17 | Proje belgeleri analizi | Tamamlandı | `PROJECT_ANALYSIS.md` oluşturuldu |
| 2026-08-17 | TODO çalışma planı | Tamamlandı | Bu dosya oluşturuldu |
| 2026-08-17 | A01 repository iskeleti | Tamamlandı | `pnpm --filter @zynero/desktop typecheck` ve `build` başarılı; Rust/cargo/rustup eksikliği A02'yi engelliyor |
| 2026-08-17 | J01 GitHub repository | Tamamlandı | `main` branch pushlandı; GitHub üzerinde 2 commit ve README doğrulandı |
| 2026-08-17 | A02/A03 Tauri katmanı | Tamamlandı | Rust 1.97.1, Tauri CLI 2.11.4, MSVC ve WebView2 kuruldu; `cargo check`, frontend build ve Windows EXE/MSI/NSIS üretimi başarılı |
| 2026-08-17 | E01/E03 frontend bileşenleri | Tamamlandı | Responsive Dashboard shell, sidebar, stats, search/filter toolbar, empty state, DownloadCard ve Add Download modalı; typecheck/build başarılı |
| 2026-08-17 | C01/E04 IPC entegrasyonu | Tamamlandı | Rust `add_download` command, güvenli URL/filename validation, frontend `invoke` bağlantısı; `cargo check`, typecheck ve build başarılı |
| 2026-08-17 | E07 tema özelleştirmesi | Devam ediyor | Midnight, Graphite ve Dawn token varyantları ile seçici eklendi; system theme ve kalıcı ayar bekliyor |
| 2026-08-17 | B01-B03/C02/E02 data layer | Tamamlandı | SQLite startup/migration, repository CRUD, HEAD/Range metadata ve Dashboard `get_downloads` entegrasyonu; cargo check/typecheck/build başarılı |
| 2026-08-17 | K01 reusable skill | Tamamlandı | `/home/ubuntu/skills/zynero-download-manager-development/SKILL.md`; `quick_validate.py` başarılı |
