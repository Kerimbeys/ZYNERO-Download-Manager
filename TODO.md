# ZYNERO TODO

**Durum işaretleri:** `[ ]` Bekliyor · `[-]` Devam ediyor · `[x]` Tamamlandı · `[!]` Engellendi

**Güncelleme kuralı:** Her görev, doğrulama komutu veya test sonucu ile birlikte işaretlenecek. Bir görev yalnızca Definition of Done koşulları sağlandığında `[x]` yapılacak.

## A. Başlangıç ve mimari

- [x] A01 — Repository çalışma alanını oluştur; `apps/desktop`, `apps/extension`, `packages/shared`, `packages/i18n`, `docs`, `scripts` ve `tests` dizinlerini ekle. Temel pnpm workspace ve React/Vite uygulama iskeleti oluşturuldu.
- [x] A02 — Tauri 2.x + React + TypeScript + Vite masaüstü iskeletini kur ve Windows hedefini doğrula. Rust 1.97.1, Tauri CLI 2.11.4, MSVC Build Tools ve WebView2 hazır; Tauri dev/release derleme doğrulandı.
- [x] A03 — Rust modül sınırlarını oluştur: `commands`, `download`, `database`, `scheduler`, `browser`, `security`, `utils`. Modül iskeletleri ve `lib.rs` bağlantıları eklendi.
- [x] A04 — Ortak domain tiplerini tanımla: `DownloadStatus`, `Download`, `Segment`, progress payload ve IPC hata modeli. `packages/shared/src/index.ts` TypeScript IPC sözleşmesi ve `apps/desktop/src-tauri/src/domain.rs` serde uyumlu Rust modelleri eklendi; desktop workspace bağımlılığı bağlandı.
- [x] A05 — Güvenli Tauri capability/permission politikasını ve minimal IPC yüzeyini tanımla. `core:default` kaldırıldı; yalnızca `core:event:default` ve notification için `is-permission-granted`, `request-permission`, `show` izinleri bırakıldı. Custom IPC command'lar yalnızca Rust handler üzerinden erişilebilir.
- [ ] A06 — `README.md`, `ARCHITECTURE.md`, `SECURITY.md`, `CONTRIBUTING.md` ve `CHANGELOG.md` başlangıç belgelerini yaz.
- [-] A07 — Format, lint, typecheck, Rust check/test ve build komutlarını çalışır hâle getir. Frontend typecheck/build ve `cargo check` çalışıyor; format/lint/test komutları sonraki adım.

## B. Kalıcı veri katmanı

- [x] B01 — SQLite bağlantı ve uygulama veri dizini yönetimini ekle. `DatabaseState` uygulama data directory altında `zynero.sqlite3` açıyor ve startup'ta yönetiliyor.
- [x] B02 — `downloads`, `segments`, `queues`, `settings` migrations oluştur. `0001`, `0002` ve `0003` migration'ları ile tüm temel tablolar, runtime alanları, indexler ve varsayılan ayarlar eklendi.
- [x] B03 — Repository katmanını yaz; frontend memory yerine gerçek CRUD kullan. `insert_download`, `list_downloads` ve `find_download` repository metotları eklendi; Dashboard `get_downloads` ile SQLite'dan besleniyor.
- [x] B04 — Download state transition kurallarını tanımla ve unit testlerini yaz. Geçerli lifecycle geçişleri ve hatalı geçiş reddi test edildi.
- [x] B05 — Startup recovery için yarım kalan indirmeleri güvenli biçimde yükle. Startup'ta `active` kayıtlar paused state'e alınarak offset'ten resume'e hazır bırakılıyor.

## C. İlk gerçek indirme dilimi — en yüksek öncelik

- [x] C01 — URL doğrulama, protocol kontrolü ve güvenli filename çıkarımı. `add_download` command HTTP/HTTPS, host, embedded credential ve güvenli filename kontrollerini yapıyor.
- [x] C02 — HEAD/ilk GET metadata tespiti: content length, content type ve Range capability. `inspect_url` HEAD + `Range: bytes=0-0` fallback ile Content-Length/Content-Range/Content-Type/Accept-Ranges okuyor.
- [x] C03 — Tek bağlantılı streaming HTTP/HTTPS download worker'ını yaz; büyük dosyayı RAM'e alma. Reqwest byte stream ve async temp file worker eklendi.
- [x] C04 — Güvenli geçici dosya ve hedef path oluşturma; traversal ve overwrite kontrollerini ekle. `.zynero.part`, Downloads/Desktop/Documents root çözümleme ve çakışmada unique filename eklendi.
- [x] C05 — Gerçek progress, speed ve ETA hesaplamasını hareketli ortalama ile ekle. Persisted downloaded bytes, speed B/s ve ETA snapshot'ları 250 ms aralıkla SQLite'a yazılıyor; Dashboard 1 saniyede yenileniyor.
- [x] C06 — Tauri command/event akışını bağla: add, get, pause, resume, cancel, delete, progress. Lifecycle command'ları, `download-progress` Tauri event'i ve Dashboard listener/polling bağlı.
- [x] C07 — Gerçek pause: request cancellation, offset persist ve SQLite state update. Worker control flag ile stream'i durduruyor; temp file boyutu ve paused state kalıcı.
- [x] C08 — Gerçek resume: persisted offset'ten devam et; destek yoksa açık hata/fallback davranışı uygula. Range destekliyse offset header kullanılıyor; 206 dönmezse sıfırdan güvenli fallback yapılıyor.
- [x] C09 — Retry/backoff, timeout, HTTP hata sınıfları ve insan okunabilir hata mesajlarını ekle. 3 denemeli exponential backoff, connect/request timeout ve transient HTTP status handling eklendi.
- [-] C10 — Local HTTP test server ile HTTP, pause/resume, restart recovery ve failure integration testlerini yaz. Test binary derleniyor; Windows test çalıştırması `STATUS_ENTRYPOINT_NOT_FOUND` ile duruyor. Test binary'sini Tauri host bağımlılıklarından ayırmak veya MSVC runtime/DLL yükleme düzenini düzeltmek gerekiyor.
- [ ] C11 — İlk dikey milestone'u uçtan uca doğrula ve `feat(core): add resumable downloads` commit'ini oluştur.

## D. Çoklu bağlantı ve performans

- [-] D01 — Segment hesaplama ve 1–32 bağlantı sınırlarını ekle. 1–32 bounded contiguous segment planner ve sınır testleri eklendi; worker concurrency entegrasyonu sonraki adım.
- [-] D02 — Range destekli sunucular için concurrent segment worker'ları yaz. Gerçek bounded segment worker, Range 206 doğrulaması, paralel temp segment dosyaları, progress aggregation ve ordered merge eklendi; pause/cancel recovery ve integration testleri bekliyor.
- [ ] D03 — Segment dosyası yazma, merge ve bütünlük kontrollerini güvenli hâle getir.
- [ ] D04 — Range unsupported fallback ve hatalı Range response senaryolarını test et.
- [ ] D05 — Global speed limiter ekle; `0 = unlimited` davranışını doğrula.
- [ ] D06 — 1 GB+ test dosyalarında RAM, CPU, hız ve UI responsiveness ölçümü yap.

## E. React masaüstü arayüzü

- [x] E01 — Windows 11 esintili responsive shell ve sidebar oluştur. Responsive sidebar, workspace navigasyonu, storage kartı ve temel uygulama shell'i eklendi.
- [x] E02 — Dashboard'u yalnızca gerçek Rust/SQLite verileriyle besle. Uygulama açılışında `get_downloads` IPC çağrısı ile persisted kayıtlar yükleniyor; mock download kullanılmıyor.
- [x] E03 — Download card: progress, bytes, speed, ETA, connection count, status ve eylemler. Backend tipine hazır DownloadCard bileşeni ve durum görselleri eklendi.
- [x] E04 — Add Download penceresini gerçek `add_download` command'ına bağla. Frontend `invoke` çağrısı, Rust request/response modeli ve hata gösterimi eklendi.
- [x] E05 — Active, Completed, Queued, Scheduled, Failed ve History görünümlerini ekle. Navigation artık gerçek status filtreleri ve arama eşleşmesi uyguluyor; Scheduled görünümü scheduler verisi bekliyor.
- [x] E06 — Pause/resume/cancel/delete/open file/open folder eylemlerini IPC'ye bağla. Tüm temel download lifecycle ve Windows Explorer file/folder command'ları frontend'e bağlandı.
- [x] E07 — Light/dark/system tema desteğini ekle; aşırı gradient/glass kullanımından kaçın. Midnight/Graphite/Dawn token varyantları, system preference başlangıcı ve localStorage kalıcılığı eklendi.
- [ ] E08 — UI testleri: add, pause, resume, delete ve settings.

## F. Kuyruk, bildirim ve ayarlar

- [x] F01 — Queue CRUD, priority, automatic start ve max concurrent downloads. Queue/settings SQLite CRUD, max concurrent kapasite hesabı, `start_queued_downloads` IPC ve 5 saniyelik auto-start runner eklendi.
- [x] F02 — Scheduler: başlangıç/duruş zamanı ve WAITING/QUEUED geçişleri. Queue schedule migration, RFC3339 evaluator, `evaluate_queue_schedule` IPC ve auto-start runner artık queue auto_start/start_at/stop_at penceresine göre gating yapıyor.
- [ ] F03 — Windows completion/failure/queue notification'larını optional yap.
- [-] F04 — General, Downloads, Connections, Notifications, Appearance, Privacy ve Advanced ayarlarını bağla. Downloads max concurrent/auto-start ayarları, Appearance tema seçenekleri ve `set_setting` IPC paneli eklendi; diğer ayar bölümleri sonraki adım.
- [-] F05 — Dosya kategorileri ve extension mapping'i ekle. Rust `get_file_category` IPC ve URL filename tabanlı archive/audio/video/images/documents/applications/other mapping eklendi; kategori bazlı UI/klasör routing sonraki adım.

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
| 2026-08-17 | C03-C05 download worker | Tamamlandı | Async reqwest streaming, `.zynero.part` temp dosyası, unique final path, Range resume, persisted progress/speed/ETA; cargo check başarılı |
| 2026-08-17 | E06 lifecycle IPC | Tamamlandı | pause/resume/cancel/delete/open file/open folder IPC, Tauri progress event listener ve polling fallback; cargo check/typecheck/build başarılı |
| 2026-08-17 | B02/B04/B05 data resilience | Tamamlandı | `0003_queues_segments_settings.sql`, lifecycle transition guard, startup recovery ve 2/2 Rust unit test başarılı |
| 2026-08-17 | E05/E07 frontend completion | Tamamlandı | Status navigation filtreleri, gerçek aktif/completed stats, system-aware theme başlangıcı ve localStorage tema kalıcılığı; typecheck/build başarılı |
| 2026-08-17 | C10 test hazırlığı | Devam ediyor | Local TCP HTTP retry ve temp path testleri eklendi; test binary derleniyor ancak Windows'ta `STATUS_ENTRYPOINT_NOT_FOUND` nedeniyle çalıştırma engelli |
| 2026-08-17 | F01 queue/settings backend | Devam ediyor | QueueRecord/settings repository CRUD, validation ve `get_queues/save_queue/get_setting/set_setting` Tauri commands eklendi; cargo check/typecheck/build başarılı |
| 2026-08-17 | F02 scheduler foundation | Devam ediyor | `0004_queue_schedule.sql`, RFC3339 schedule window evaluator ve `evaluate_queue_schedule` IPC eklendi; cargo check/typecheck/build başarılı; otomatik queue runner bekliyor |
| 2026-08-17 | F04 settings foundation | Devam ediyor | Settings paneli, max concurrent/auto-start `set_setting` IPC ve tema seçenekleri eklendi; typecheck/build başarılı |
| 2026-08-17 | D01 segment foundation | Devam ediyor | 1–32 bağlantı sınırları, contiguous byte-range planner ve boundary testleri eklendi; cargo check başarılı; worker entegrasyonu bekliyor |
| 2026-08-17 | F01/F02 queue runner | Tamamlandı | `start_queued_downloads` IPC, max-concurrent kapasite hesabı, 5 saniyelik auto-start runner ve queue schedule window gating; cargo check başarılı |
| 2026-08-17 | F05 category foundation | Devam ediyor | `get_file_category` IPC ve extension mapping eklendi; cargo check başarılı; kategori UI ve destination routing bekliyor |
| 2026-08-17 | K01 reusable skill | Tamamlandı | `/home/ubuntu/skills/zynero-download-manager-development/SKILL.md`; `quick_validate.py` başarılı |
| 2026-08-17 | D02 segment worker foundation | Devam ediyor | Range destekli indirmeler için bounded concurrent workers, per-segment temp files, aggregate progress ve ordered merge eklendi; `cargo check`, `cargo fmt`, frontend typecheck/build başarılı; pause/cancel integration testleri bekliyor |

## Devam oturumu — sonraki dikey dilimler

- [ ] C11 — İlk resumable download milestone'unu uçtan uca doğrula ve cohesive commit oluştur.
- [ ] D02 — Range destekli sunucular için concurrent segment worker'larını gerçek worker akışına bağla.
- [ ] D03 — Segment dosyası yazma, merge ve bütünlük kontrollerini güvenli hâle getir.
- [ ] D04 — Range unsupported fallback ve hatalı Range response senaryolarını test et.
- [ ] E08 — Add, pause, resume, delete ve settings için UI testlerini çalıştır.
- [ ] F03 — Windows completion/failure/queue notification'larını optional ayarlarla tamamla.
- [ ] F04 — General, Downloads, Connections, Notifications, Appearance, Privacy ve Advanced ayarlarının kalan IPC akışlarını tamamla.
- [ ] F05 — Kategori filtreleme sonrası kategori bazlı klasör routing ve UI görünürlüğünü tamamla.

## UI önizleme doğrulaması

- [x] Windows Tauri masaüstü UI'sını güncel kaynaklarla çalıştır.
- [x] Dashboard ekranının güncel görüntüsünü yakala veya çalıştırma engelini kaydet. `createRoot(...).render(<App />)` mount eksikliği giderildi; gerçek ZYNERO dashboard görüntüsü Windows üzerinde yakalandı.
- [x] Kullanıcıya masaüstü UI önizlemesini ve doğrulama sonucunu göster.
- [x] UI-BOOT — Tauri WebView boş ekran nedenini teşhis et ve düzelt. Eksik React root mount çağrısı eklendi; `pnpm --dir apps/desktop typecheck`, production build ve Tauri dev runtime başarılı.

## Devam oturumu — sonraki açık görev

- [x] Sıradaki açık görevi mevcut TODO sırasına göre belirle ve kapsamını doğrula. Seçilen görev: A04.
- [x] Seçilen görev için gerçek backend/IPC/UI dikey dilimini uygula. Shared TypeScript sözleşmesi, Rust serde modelleri ve frontend DownloadStatus bağlantısı eklendi.
- [x] Windows Rust, frontend typecheck/build ve ilgili testleri çalıştır. `pnpm install`, desktop typecheck, `cargo fmt --check`, `cargo check` ve production build başarılı; yalnızca mevcut/uygulanmamış domain kullanımına ait dead-code uyarıları kaldı.
- [x] Doğrulama sonrası bu oturumun görev durumunu ve commit kapsamını güncelle. A04 değişiklikleri cohesive commit için hazırlandı.

## A05 → Windows release teslim planı

- [x] A05 güvenlik kapsamını mevcut Tauri capability ve IPC command listesiyle karşılaştır.
- [x] Minimal capability/permission politikasını uygula ve gereksiz izinleri kaldır.
- [x] Kritik ürün akışlarının mevcut gerçek backend bağlantısını doğrula; mock veya sahte ilerleme eklenmedi.
- [x] Windows smoke testini çalıştır; release `zynero.exe` gerçek pencereyi `ZYNERO` başlığıyla açtı ve process yanıt veriyor.
- [x] Windows installer/`.exe` paketini oluştur. NSIS ve MSI çıktıları üretildi.
- [-] Release notlarını, checksum bilgisini ve kullanım talimatlarını ekle. `RELEASE_NOTES_v0.1.0.md` hazırlandı ve NSIS installer SHA-256 doğrulandı; GitHub release asset yüklemesi sonraki teslim adımı.
