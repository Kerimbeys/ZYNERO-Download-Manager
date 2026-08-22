# ZYNERO v0.1.1 — Windows x64 Release

ZYNERO v0.1.1, Tauri 2 tabanlı Windows x64 download manager sürümüdür. Bu sürüm özellikle YouTube `watch` gibi doğrudan medya dosyası olmayan URL'lerin yaklaşık 1 KB HTML dosyası olarak kaydedilmesini engelleyen response doğrulamasını içerir.

## Paket ve SHA-256

| Dosya | Tür | Boyut | SHA-256 |
|---|---|---:|---|
| `ZYNERO_0.1.1_x64-setup.exe` | Windows NSIS installer | 3,745,986 bytes | `268403A5C1F11DEB07109EF81A9CC355215E1226890CBF720A57529D942E92EE` |
| `ZYNERO_0.1.1_x64_en-US.msi` | Windows MSI installer | 5,251,072 bytes | `5ECF9BEE89E8413F6548ABB94203013C9D60F9C8A511A69213F1B4BC3F58C81B` |
| `zynero.exe` | Raw Windows executable | 13,768,704 bytes | `2F3C48C3A9A93CF6DD40358E6DA783A4DEF4250BEFB654E802B894B732904711` |

Checksum yeniden üretmek için PowerShell üzerinde şu komutları kullanın:

```powershell
Get-FileHash .\ZYNERO_0.1.1_x64-setup.exe -Algorithm SHA256
Get-FileHash .\ZYNERO_0.1.1_x64_en-US.msi -Algorithm SHA256
Get-FileHash .\zynero.exe -Algorithm SHA256
```

## Bu sürümdeki değişiklikler

YouTube/player/consent sayfası gibi `text/html` veya `application/xhtml+xml` response'ları metadata ve worker katmanlarında reddedilir. Kullanıcıya doğrudan medya URL'si kullanması gerektiğini bildiren hata akışı korunur; ZYNERO, genel bir YouTube extractor değildir.

SHA-256 hesaplama ortak Rust helper'ına taşınmış, completed download kartına success/mismatch doğrulama görünümü eklenmiş ve hassas URL/query/header değerlerinin lokal log formatter'ında maskelenmesi etkinleştirilmiştir. Add Download, pause/resume/delete, Notifications Settings ve hash success/mismatch DOM testleri başarıyla çalıştırılmıştır.

## Doğrulama sonuçları

| Kontrol | Sonuç |
|---|---|
| Frontend UI testleri | 1 dosya, 5 test başarılı |
| Frontend typecheck | Başarılı |
| Frontend production build | Başarılı |
| Rust `cargo check` | Başarılı |
| Rust lib testleri | 21 test başarılı |
| Rust format check | Başarılı |
| Windows Tauri bundle | NSIS ve MSI başarılı |

## Kurulum ve test

`ZYNERO_0.1.1_x64-setup.exe` dosyasını çalıştırıp Windows kurulum adımlarını izleyin. Kurulumdan önce artifact checksum değerlerini karşılaştırın; eşleşmeyen dosyaları çalıştırmayın. Kurulum sonrasında Add Download, Auto destination, pause/resume, Notifications Settings ve tamamlanmış dosya SHA-256 verification akışlarını kontrol edin.

YouTube için yalnızca watch URL'si eklemek desteklenen bir medya dosyası indirme sözleşmesi değildir. ZYNERO bu adresi HTML olarak kaydetmek yerine reddeder. Doğrudan medya stream URL'leri için Content-Type ve Range gereksinimleri sağlayıcıya göre değişebilir.

## Bilinen sınırlamalar

Kod imzalama sertifikası, signed updater manifesti ve otomatik güncelleme uçtan uca doğrulaması henüz etkin değildir; bu nedenle Windows SmartScreen uyarısı görülebilir. Temiz Windows 10/11 makinelerinde install, uninstall, upgrade ve rollback smoke testleri ayrıca çalıştırılmalıdır.

Bu release GitHub üzerinde checksum doğrulamasıyla paylaşılır; imza doğrulaması eklenene kadar kullanıcılar checksum değerlerini karşılaştırmalıdır.

## Kaynak derleme

```powershell
pnpm --dir apps\desktop test -- --maxWorkers=1 --minWorkers=1
pnpm --dir apps\desktop typecheck
pnpm --dir apps\desktop build
cargo fmt --manifest-path apps\desktop\src-tauri\Cargo.toml -- --check
cargo check --manifest-path apps\desktop\src-tauri\Cargo.toml
cargo test --manifest-path apps\desktop\src-tauri\Cargo.toml --lib -- --nocapture
pnpm --dir apps\desktop tauri build
```

**Not:** Bu sürüm, geliştirme ortamında ve bağlı Windows build makinesinde doğrulanmıştır. Kod imzalama ve temiz makine release gate'leri tamamlanmadan kurumsal dağıtım için ek doğrulama gereklidir.

## References

[1]: https://tauri.app/ Tauri Documentation
[2]: https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases GitHub Releases Documentation

*Hazırlayan: Manus AI*

##### Sources

- [Tauri Documentation](https://tauri.app/)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)

---

## v0.1.0 referansı

Önceki release notes dosyası `RELEASE_NOTES_v0.1.0.md` olarak korunmuştur.
