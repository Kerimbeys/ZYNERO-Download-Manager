# ZYNERO

> **Download. Faster. Smarter.**

ZYNERO, Windows 10/11 için geliştirilen, gerçek HTTP/HTTPS indirmelerini destekleyen, güvenlik ve gizliliği önceliklendiren yeni nesil masaüstü indirme yöneticisidir. Proje, görsel bir mockup değil; Rust tabanlı gerçek bir indirme motoru, kalıcı SQLite durumu ve React/Tauri arayüzü üzerine kurulacaktır.

## Proje durumu

Proje şu anda **MVP geliştirme aşamasındadır**. İlk dikey hedef; URL doğrulama, gerçek dosya oluşturma, gerçek ilerleme, pause/resume ve tamamlanma akışını uçtan uca çalıştırmaktır.

| Katman | Teknoloji | Durum |
|---|---|---|
| Desktop shell | Tauri 2.x | Rust toolchain kurulumu bekleniyor |
| Frontend | React, TypeScript, Vite | Başlangıç iskeleti hazır |
| Styling | Tailwind CSS / erişilebilir UI | Planlandı |
| Core | Rust, Tokio, Reqwest | Planlandı |
| Persistence | SQLite ve migrations | Planlandı |
| Browser | WebExtension API, Native Messaging | MVP sonrası |

## Geliştirme ilkeleri

ZYNERO sahte progress bar, mock download veya `setInterval` tabanlı simülasyon kullanmayacaktır. Büyük dosyalar streaming I/O ile işlenecek, frontend yalnızca gerçek Rust motoru durumunu gösterecek ve kritik özellikler local HTTP test sunucusu üzerinden doğrulanacaktır.

Güvenlik kapsamında path traversal koruması, filename sanitization, HTTPS sertifika doğrulaması, minimal Tauri izinleri, güvenli IPC ve hassas bilgilerin loglanmaması temel gereksinimlerdir.

## Repository yapısı

```text
apps/
  desktop/       React/Vite frontend ve Tauri/Rust backend
  extension/     Chrome, Edge ve Firefox WebExtension
packages/
  shared/        Ortak domain tipleri ve IPC şemaları
  i18n/          Translation key'leri ve locale kaynakları
docs/            Mimari, güvenlik ve karar kayıtları
scripts/         Geliştirme ve test yardımcıları
tests/           Entegrasyon testleri
```

## Başlangıç

Node.js ve pnpm kurulu bir ortamda:

```bash
pnpm install
pnpm --filter @zynero/desktop typecheck
pnpm --filter @zynero/desktop build
```

Tauri geliştirme ve Windows paketleme için Rust toolchain ile Tauri CLI ayrıca kurulmalıdır. Rust backend tamamlanana kadar mevcut frontend, yalnızca uygulama kabuğunu doğrulamak amacıyla kullanılmaktadır; üretim indirme işlevi henüz eklenmemiştir.

## Belgeler

Ayrıntılı kapsam, riskler ve geliştirme sırası için [`PROJECT_ANALYSIS.md`](./PROJECT_ANALYSIS.md) dosyasına; görev durumları için [`TODO.md`](./TODO.md) dosyasına bakın.

## Lisans

Lisans kararı ürün ve dağıtım modeli netleştirildikten sonra eklenecektir.
