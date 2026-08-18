# ZYNERO Architecture

## Amaç

ZYNERO'nun mimarisi, Windows üzerinde güvenli ve gerçek dosya indirmelerini UI'dan ayrıştırır. React yalnızca kullanıcı etkileşimini ve görüntülemeyi yönetir; ağ, dosya sistemi, lifecycle state, segment worker'ları ve SQLite işlemleri Rust tarafında kalır.

## Katmanlar

| Katman | Sorumluluk | Sınır |
|---|---|---|
| React/Vite | Dashboard, filtreler, temalar, form ve IPC sonucu görünümü | Ham filesystem, HTTP ve SQLite erişimi yok |
| Tauri commands | Typed invoke sınırı ve kullanıcıya dönük hata modeli | Her komut Rust validation'dan geçer |
| Download engine | Metadata, streaming, retry, pause/resume, segment ve merge | Geçici dosya ve destination policy ile çalışır |
| DatabaseState | SQLite bağlantısı, migration, CRUD, recovery ve state transition | UI memory state'i source of truth değildir |
| Scheduler | Queue auto-start ve schedule window değerlendirmesi | İndirme worker'ının güvenlik kurallarını değiştirmez |
| Security/utils | URL, path, filename, kategori ve hata yardımcıları | Validation ortak kuralları tekrar edilmez |

## Başlatma akışı

Tauri `run` fonksiyonu yalnızca command handler'larını, notification plugin'ini, database state'ini, download manager'ı ve scheduler'ı kurar. Startup sırasında SQLite migrations çalışır ve yarım kalmış `active` kayıtlar recovery kuralıyla `paused` durumuna alınır. Scheduler, ayarları ve queue window'larını okuyarak kapasite sınırı dahilinde queued kayıtları başlatır.

```text
Tauri startup
    |
    +--> DatabaseState::open --> migrations --> recover_incomplete
    |
    +--> DownloadManager::new --> app handle / control registry
    |
    +--> invoke_handler --> typed commands
    |
    +--> scheduler loop --> queue gating --> download workers
```

## Gerçek indirme akışı

1. Kullanıcı Add Download formuna URL ve hedef bilgisi girer.
2. `add_download` URL scheme, host, embedded credential, filename ve destination root doğrulamasından geçirir.
3. `inspect_url` HEAD ve gerektiğinde body-free Range isteğiyle content length, content type, Content-Range ve Accept-Ranges bilgisini toplar.
4. Download kaydı SQLite'a `queued` olarak yazılır.
5. Worker gerçek HTTP byte stream'ini `.zynero.part` veya segment geçici dosyasına yazar.
6. Her snapshot'ta downloaded bytes, speed, ETA ve state SQLite'a yazılır; UI typed event/polling ile güncellenir.
7. Pause/cancel control flag ile worker durur ve offset/state kalıcılaştırılır. Resume persisted offset ve sunucu yanıtına göre devam eder veya güvenli fallback uygular.
8. Segment akışında yalnızca doğrulanmış `206` ve beklenen `Content-Range` kabul edilir. Segmentler byte-range sırasına göre birleştirilir; eksik, bozuk veya uzunluğu uyuşmayan parça final çıktı üretmeden reddedilir.
9. Başarıda temporary files temizlenir, final path doğrulanır ve SQLite state `completed` olur.

## IPC sözleşmesi

Komut listesi `apps/desktop/src-tauri/src/lib.rs` içinde açıkça register edilir. Ortak serializable tipler `packages/shared` ve Rust `domain` modelleri arasında korunur. Frontend `@tauri-apps/api/core` üzerinden `invoke` kullanır; custom command'lar doğrudan filesystem veya shell izni vermez.

Temel lifecycle komutları `add_download`, `get_downloads`, `pause_download`, `resume_download`, `cancel_download`, `delete_download`, `open_download_file`, `open_download_folder`, `verify_download_hash`, queue ve settings komutlarıdır.

## Kalıcılık modeli

SQLite; downloads, segments, queues, settings ve runtime progress alanlarını taşır. Migration'lar startup'ta sıralı ve idempotent uygulanır. Persisted state, yeniden başlatma sonrası worker recovery için kullanılır. React state yalnızca ekrandaki projection'dır ve yeniden açılışta database'den tekrar yüklenir.

## Test sınırları

Rust unit/integration testleri gerçek local HTTP server, Range yanıtı, retry, pause/resume, 404 failure, malformed Content-Range ve fallback senaryolarını kullanır. Tauri WebView bootstrap'ı Rust test koşulundan ayrıştırılmıştır; böylece Windows test binary ürün davranışını test ederken UI runtime'a gereksiz bağımlı kalmaz.

## Gelecek genişlemeler

D01-D06 kapsamında segment recovery, global speed limiter ve 1 GB+ ölçümleri; E/F kapsamında UI testleri, notification ayarları, settings IPC ve category routing; H/I kapsamında i18n, WebExtension, CI, threat model ve release hardening eklenir. Bu genişlemeler mevcut UI → typed IPC → Rust validation → SQLite/worker → typed result sınırını bozmamalıdır.
