# ZYNERO Contributing Guide

## Çalışma ilkesi

ZYNERO değişiklikleri, gerçek kullanıcı akışını koruyan küçük ve doğrulanabilir vertical slice'lar olarak geliştirilir. Her değişiklik UI → typed Tauri invoke → Rust validation → SQLite veya worker → typed result/event yolunu açıkça korumalıdır. Sahte progress, mock download, frontend-only persistence ve kanıtsız tamamlanma kabul edilmez.

## Başlamadan önce

Repository'yi Windows MSVC ve WebView2 hazır bir ortamda kurun:

```bash
pnpm install
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
```

İlk incelemede `TODO.md`, `PROJECT_ANALYSIS.md`, `ARCHITECTURE.md`, `SECURITY.md` ve ilgili Rust/TypeScript dosyalarını okuyun. Yeni bir görev seçerken TODO'daki ilk açık ve önkoşulları sağlanmış görevi tercih edin.

## Kod sınırları

Frontend doğrudan filesystem, network veya SQLite kullanmaz. Yeni bir veri alanı boundary'ler arasında taşınıyorsa `packages/shared`, Rust serde domain modeli, repository mapping, IPC response ve React view model birlikte güncellenir. Persistent veri için numaralı SQLite migration eklenir ve startup migration sırasına kaydedilir.

Yeni Tauri plugin veya capability eklemeden önce ihtiyaç duyulan kesin permission'ı belirleyin. `core:default` veya wildcard window izni eklemeyin. URL, host, filename, path, request size ve state transition validation'ı Rust tarafında yapılmalıdır.

## Doğrulama matrisi

Küçük değişikliklerde ilgili testleri, teslim öncesinde aşağıdaki komutları çalıştırın:

```bash
pnpm install
pnpm --dir apps/desktop typecheck
pnpm --dir apps/desktop build
pnpm --dir apps/desktop test
cargo fmt --manifest-path apps/desktop/src-tauri/Cargo.toml -- --check
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml --lib -- --nocapture
pnpm --dir apps/desktop tauri info
```

İndirme değişiklikleri için ayrıca local HTTP server ile retry, 404, Range 206, pause/resume, restart recovery, malformed Content-Range ve Range unsupported fallback senaryolarını doğrulayın. Windows Tauri smoke testinde responsive `ZYNERO` penceresinin açıldığını kaydedin.

## TODO kanıt kuralı

`TODO.md` yetkili görev ledger'idir. `[x]` yalnızca Definition of Done, ilgili test ve kullanıcı akışı kanıtlandığında kullanılabilir. Kod yazılmış fakat doğrulama bekliyorsa `[-]`, gerçek blokaj varsa `[!]` kullanın. Her oturum sonunda tarih, görev, durum, dosyalar, komutlar, çıktı ve kalan riski evidence tablosuna ekleyin.

## Commit ve pull request

Temiz çalışma ağacını doğruladıktan sonra bir cohesive commit oluşturun. Commit mesajları Conventional Commits biçiminde olmalıdır:

```text
feat(download): add ordered segment merge
fix(security): reject unsafe destination paths
test(download): cover range fallback
```

Pull request açıklaması kullanıcıya görünen davranışı, backend sözleşmesini, migration etkisini, çalıştırılan komutları, installer/artifact yollarını ve kalan riskleri belirtmelidir. Test çalıştırılmadıysa bunu gizlemeyin.

## Review kontrolü

Review sırasında şu sorular cevaplanmalıdır: İndirme gerçekten ağdan mı geliyor? Progress gerçek byte sayacından mı üretiliyor? Pause/resume sonrası state ve temporary dosya güvenli mi? Hatalı Range yanıtı bozulmuş merge'e yol açabilir mi? Yeni IPC veya capability gereksiz yetki açıyor mu? UI başarısız ve loading durumlarını gösteriyor mu? Windows'ta temiz bir smoke yolu var mı?

## Lisans ve davranış

Lisans kararı verilene kadar katkılar repository'nin mevcut lisans durumunu değiştirmez. Kullanıcı verisi, credential, cookie, private URL veya installer secret'ı issue, commit veya log içine koymayın.
