# ZYNERO 0.1.0 — Windows Release Candidate

ZYNERO 0.1.0, Windows x64 için Tauri tabanlı masaüstü download manager release paketidir.

## Paket ve SHA-256

| Dosya | Tür | Boyut | SHA-256 |
|---|---|---:|---|
| `ZYNERO_0.1.0_x64-setup.exe` | Windows NSIS installer | 3,738,602 bytes | `045EF0B811C6C75D378C01A9A4C2FEADE636BA9AC2BAB58CB401ED406070A60E` |
| `ZYNERO_0.1.0_x64_en-US.msi` | Windows MSI installer | 5,246,976 bytes | `9F7FCE96AB247BB5EF2CEBA00CB7D9767DD75D3474A2DD20C3E6E1F90E90995B` |
| `zynero.exe` | Raw Windows executable | 13,767,168 bytes | `4DE7995A3DF2EBC6FBDFA8CD3E5A0D9F9E2676EB6B80D157B21F7AE7DBF8D267` |

Checksum yeniden üretmek için PowerShell üzerinde şu komut kullanılabilir:

```powershell
Get-FileHash .\ZYNERO_0.1.0_x64-setup.exe -Algorithm SHA256
Get-FileHash .\ZYNERO_0.1.0_x64_en-US.msi -Algorithm SHA256
```

## Kurulum

`ZYNERO_0.1.0_x64-setup.exe` dosyasını çalıştırın ve Windows kurulum adımlarını izleyin. Kurulumdan sonra Başlat menüsünden ZYNERO’yu açabilirsiniz. Uygulama ilk çalıştırmada kendi SQLite veri klasörünü oluşturur. MSI paketi kurumsal dağıtım senaryolarında kullanılabilir.

Kurulumdan önce artifact checksum değerlerini karşılaştırın; eşleşmeyen dosyaları çalıştırmayın. Kurulum sonrasında Add Download, pause/resume, Notifications Settings ve kategori tabanlı `Auto` destination akışlarını kontrol edin.

## Bu sürümde doğrulananlar

Gerçek HTTP/HTTPS metadata inspection, queued download persistence, streaming download, pause/resume/cancel komutları, Range destekli concurrent segment worker temeli, kategori mapping, Windows bildirimleri, SHA-256 doğrulama komutu, React/Tauri dashboard ve minimal Tauri capability izinleri doğrulanmıştır. Release `zynero.exe` Windows üzerinde `ZYNERO` başlığıyla açılmış ve process smoke testinden geçmiştir.

E08 DOM testlerinde Add Download ile `Auto` destination isteği, pause/resume/delete lifecycle çağrıları ve Notifications Settings persistence akışı 3/3 başarılıdır. Rust lib test paketi redaction ve hash helper testleri dahil 19/19 başarılıdır.

## Bilinen sınırlamalar

Gerçek Tauri WebView üzerinde temiz Windows makine smoke testi, kod imzalama sertifikasıyla imzalama, signed updater manifesti ve otomatik güncelleme uçtan uca doğrulaması bu pakette tamamlanmış değildir. Kod imzalama sertifikası ve signed auto-update altyapısı etkin değildir; bu nedenle Windows SmartScreen uyarısı görülebilir.

Segment pause/cancel recovery ve Range unsupported fallback için mevcut test kapsamı vardır; temiz makine kurulum, kaldırma, upgrade ve rollback senaryoları ayrıca release gate olarak çalıştırılmalıdır.

## Sonraki release gate'leri

Kod imzalama sertifikası güvenli CI secret olarak eklenmeli, NSIS/MSI artifact'ları imzalanmalı ve imza doğrulaması CI'da kontrol edilmelidir. Ardından updater public key ve imzalı manifest ile ayrı bir v0.1.1 test kanalı oluşturulmalı; Windows 10/11 temiz makinelerinde kurulum, kaldırma, upgrade ve rollback sonuçları kayda alınmalıdır.
