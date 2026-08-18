# ZYNERO 0.1.0 — Windows Release Candidate

ZYNERO 0.1.0, Windows x64 için Tauri tabanlı masaüstü download manager release paketidir.

## Paket

| Dosya | Tür | SHA-256 |
|---|---|---|
| `ZYNERO_0.1.0_x64-setup.exe` | Windows NSIS installer | `ACE8C2B5951847F8F1BE8DD4C3F4D1F0F137D58BAA974CEB60178F8CF4AA8E63` |
| `ZYNERO_0.1.0_x64_en-US.msi` | Windows MSI installer | `506E642DB832F275BF8FDF8F3FA1F572F39F75A0BDDC201BDF44028101385D9F` |

## Kurulum

`ZYNERO_0.1.0_x64-setup.exe` dosyasını çalıştırın ve Windows kurulum adımlarını izleyin. Kurulumdan sonra Başlat menüsünden ZYNERO’yu açabilirsiniz. Uygulama ilk çalıştırmada kendi SQLite veri klasörünü oluşturur.

## Bu sürümde doğrulananlar

Gerçek HTTP/HTTPS metadata inspection, queued download persistence, streaming download, pause/resume/cancel komutları, Range destekli concurrent segment worker temeli, kategori mapping, Windows bildirimleri, SHA-256 doğrulama komutu, React/Tauri dashboard ve minimal Tauri capability izinleri doğrulanmıştır. Release `zynero.exe` Windows üzerinde `ZYNERO` başlığıyla açılmış ve process smoke testinden geçmiştir.

## Bilinen sınırlamalar

Windows test binary çalıştırması için daha fazla MSVC entegrasyon çalışması gerekmektedir. Segment pause/cancel recovery ve Range unsupported fallback için integration test kapsamı release sonrasında genişletilmelidir. Kod imzalama sertifikası ve signed auto-update altyapısı bu pakette henüz etkin değildir; bu nedenle Windows SmartScreen uyarısı görülebilir.
