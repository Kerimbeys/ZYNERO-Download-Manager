# ZYNERO

> **Download. Faster. Smarter.**

ZYNERO, Windows 10/11 için geliştirilen, gerçek HTTP/HTTPS indirmelerini kullanan, gizlilik ve güvenliği önceleyen masaüstü download manager uygulamasıdır. Ürün; React/Tauri arayüzü, Rust tabanlı indirme motoru ve SQLite kalıcılığı üzerine kuruludur. İlerleme bilgisi frontend'de simüle edilmez; Rust worker'larından gelen gerçek byte sayaçlarıyla gösterilir.

## Proje durumu

ZYNERO şu anda **MVP 0.1 release-candidate geliştirme aşamasındadır**. Çalışan dikey dilim; URL doğrulama, metadata inceleme, streaming indirme, geçici dosya, gerçek progress/speed/ETA, pause/resume/cancel, SQLite state persistence, retry ve queue auto-start akışlarını kapsar. Windows NSIS/MSI paketleri daha önce üretilmiş ve smoke test edilmiştir. D03/D04 segment merge ve Range fallback sağlamlık çalışmaları ayrı kanıtla tamamlanmaktadır.

| Katman | Teknoloji | Durum |
|---|---|---|
| Desktop shell | Tauri 2.x | Windows üzerinde çalışır |
| Frontend | React 19, TypeScript, Vite | Dashboard ve lifecycle eylemleri bağlı |
| Styling | Windows 11 esintili özel UI | Midnight, Graphite ve Dawn temaları |
| Core engine | Rust, Tokio, Reqwest | Streaming, retry, pause/resume ve segment worker'ları |
| Persistence | SQLite, rusqlite, migrations | Downloads, segments, queues, settings ve recovery |
| Browser | WebExtension + Native Messaging | MVP sonrası kapsam |

## Temel ilkeler

ZYNERO sahte progress bar, mock download veya frontend-only persistence kullanmaz. Büyük dosyalar streaming I/O ile diske yazılır; segmentler ayrı geçici dosyalarda tutulur, byte-range sırasına göre birleştirilir ve final çıktı uzunluk bütünlüğüyle kontrol edilir. Range desteklemeyen veya hatalı yanıt veren sunucular güvenli biçimde tek bağlantılı worker'a düşürülür.

Güvenlik kapsamında URL scheme/host/credential kontrolü, filename sanitization, destination root kısıtlaması, minimal Tauri capability izinleri, typed IPC ve hassas bilgilerin loglanmaması temel gereksinimlerdir.

## Repository yapısı

```text
apps/desktop/
  src/                 React dashboard, stores, IPC client
  src-tauri/
    src/               Rust commands, download engine, database, scheduler, security
    migrations/        SQLite migration dosyaları
    capabilities/      Tauri permission policy
apps/extension/        WebExtension kapsamı
packages/shared/       Ortak TypeScript domain ve IPC sözleşmeleri
packages/i18n/         Locale altyapısı
 docs/                  Mimari, güvenlik ve karar belgeleri
scripts/               Local test ve release yardımcıları
tests/                 Cross-layer test fixtures
TODO.md                Yetkili görev ve kanıt defteri
```

## Geliştirme

Node.js, pnpm, Rust stable ve Windows MSVC Build Tools kurulu bir ortam gerekir. İlk kurulum ve temel doğrulama:

```bash
pnpm install
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
```

Tauri geliştirme ve paketleme Windows ortamında çalıştırılmalıdır:

```bash
pnpm --dir apps/desktop tauri info
pnpm --dir apps/desktop tauri dev
pnpm --dir apps/desktop tauri build
```

İlgili Rust testleri:

```bash
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml download::tests -- --nocapture
```

Vite preview'sinde Tauri IPC mevcut olmayabileceği için desktop-only çağrılar korumalı şekilde ele alınmalıdır. Gerçek ürün doğrulaması Tauri runtime ve local HTTP test server ile yapılır.

## Belgeler ve görev defteri

Mimari sınırlar için [`ARCHITECTURE.md`](./ARCHITECTURE.md), güvenlik politikası için [`SECURITY.md`](./SECURITY.md), katkı ve doğrulama akışı için [`CONTRIBUTING.md`](./CONTRIBUTING.md), sürüm geçmişi için [`CHANGELOG.md`](./CHANGELOG.md) ve görev kanıtları için [`TODO.md`](./TODO.md) dosyasına bakın.

## Lisans

Lisans kararı ürün ve dağıtım modeli netleştirildikten sonra eklenecektir. Bu karar verilene kadar repository, açık bir lisans iddiasında bulunmaz.
