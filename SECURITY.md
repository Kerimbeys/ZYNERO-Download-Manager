# ZYNERO Security Model

## Güvenlik hedefi

ZYNERO, indirme URL'sini, hedef dosya yolunu ve kullanıcı makinesindeki dosya işlemlerini güvenli sınırlar içinde tutmayı amaçlar. Frontend güvenlik sınırı değildir; güvenlik kontrolleri Rust command ve worker katmanında tekrar doğrulanır.

## URL ve ağ güvenliği

Yalnızca `http` ve `https` scheme'leri kabul edilir. Embedded username/password, boş veya çözümlenemeyen host, malformed URL ve desteklenmeyen scheme reddedilir. HTTP içeriği güvenilir kabul edilmez; status, Content-Length, Content-Range ve Range davranışı açıkça doğrulanmadan segmented merge yapılmaz.

HTTPS sertifika doğrulaması Reqwest/TLS varsayılanlarıyla korunur. Kullanıcı tarafından verilen authorization, cookie veya benzeri secret bilgiler URL içine gömülü olmamalıdır. Bu tür içerikler hata mesajlarına ve loglara yazılmaz.

## Dosya yolu güvenliği

Filename URL'den çıkarılırken path separator, traversal bileşenleri, control character ve platforma aykırı adlar sanitize edilir. Destination yalnızca uygulamanın izin verdiği kullanıcı klasörleri içinde çözülür. Frontend'in ham bir arbitrary path'i Rust doğrulamasını atlayamaz. Existing file davranışı overwrite yerine güvenli unique-name veya açık ürün politikasına bağlıdır.

Geçici dosyalar `.zynero.part` veya segment-specific temporary path olarak tutulur. Final dosya yalnızca tamamlanmış ve bütünlük kontrolleri geçmiş iş akışında görünür hâle getirilir. Başarısız veya iptal edilmiş işlerde temporary parçalar temizlenir; cleanup başarısızlığı sessizce başarı olarak raporlanmaz.

## WebView içerik güvenliği
Tauri WebView için `tauri.conf.json` içinde dar bir Content Security Policy etkinleştirildi. Varsayılan kaynaklar yalnızca uygulama paketini, IPC bağlantısı için `ipc:` ve `http://ipc.localhost` kanallarını; transfer metadata ve asset erişimi için gerekli HTTPS bağlantılarını kapsar. `script-src` uygulama paketine, `style-src` ise React/Vite stil üretimi nedeniyle self ve inline stillere sınırlandırılmıştır. CSP uzaktaki script çalıştırmayı ve geniş kaynak yüklemeyi varsayılan olarak engeller.

## Tauri capability politikası

`apps/desktop/src-tauri/capabilities/default.json` yalnızca `main` penceresine uygulanır. Capability listesi `core:event:default` ve UI tarafından kullanılan notification izinleriyle sınırlıdır. `core:default`, shell, dialog, updater veya geniş filesystem izinleri varsayılan olarak açılmaz.

Custom application command'ları Rust `invoke_handler` üzerinden register edilir. Bu command'lar özel bir Tauri capability izni yerine kendi input validation katmanlarını kullanır. Yeni bir plugin eklenirse yalnızca ihtiyaç duyulan `allow-*` izinleri ve dar window scope'u eklenir.

## IPC ve state güvenliği

Her IPC request typed serde modeline dönüştürülür, validation'dan geçer ve kullanıcıya okunabilir fakat secret içermeyen hata döner. Lifecycle state geçişleri database repository içinde doğrulanır. UI state source of truth değildir; yeniden başlatma sonrası SQLite recovery uygulanır.

Progress event'leri yalnızca gerçek worker sayaçlarını taşır. Frontend'de sahte progress, mock download veya client-side completion kullanılmaz.

## Log ve gizlilik

Production logları URL credential'ı, cookie, authorization header, access token, tam kullanıcı yolu veya hassas dosya içeriği içeremez. Debug logları dahi secret redaction uygulamalıdır. Download error mesajları status code, güvenli sınıf ve genel açıklama içermeli; response body doğrudan kullanıcıya veya loga aktarılmamalıdır.

## Hash doğrulama ve release

Hash verification yalnızca SQLite'da persist edilmiş ve Rust tarafından doğrulanmış download path üzerinde çalışmalıdır. Frontend'den gelen arbitrary verification path'i kabul edilmez. Release installer'ları SHA-256 ile kaydedilir. Code signing, signed update ve temiz makine testi doğrulanmadıkça tamamlanmış sayılmaz ve release notlarında açıkça belirtilir.

## Tehdit tablosu

| Tehdit | Koruma | Kalan çalışma |
|---|---|---|
| Path traversal | Filename sanitize ve destination root validation | G02 audit kapsamı genişletilecek |
| Hatalı Range yanıtı | `206`/Content-Range/length doğrulaması ve single-stream fallback | D04 Windows evidence tamamlanacak |
| Secret loglama | Genel hata sınıfları ve redaction kuralı | G03 merkezi logger çalışması |
| Bozuk tamamlanmış dosya | Segment merge integrity ve hash verification | G04 hash komutu/UX tamamlanacak |
| Geniş IPC yetkisi | Minimal capability ve typed commands; CSP ile WebView kaynak sınırı | G02 final audit |
| Yetkisiz update/installer | Checksum ve release kayıtları | G05 signing/update planı |

Güvenlik açığı bildirmek için public issue açmadan önce maintainers'a özel bir GitHub security advisory veya repository sahibi tarafından belirtilen özel kanal kullanılmalıdır. Hassas exploit ayrıntıları public issue içine koyulmamalıdır.
