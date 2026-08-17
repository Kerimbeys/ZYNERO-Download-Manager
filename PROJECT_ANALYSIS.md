# ZYNERO Proje Analizi ve Çalışma Planı

**Sürüm hedefi:** MVP 0.1

**Platform:** Windows 10/11; mimari olarak macOS ve Linux'a açık

**Ürün adı:** ZYNERO

**Tagline:** Download. Faster. Smarter.

## 1. Yönetici özeti

ZYNERO, görsel bir demo değil, gerçek HTTP/HTTPS trafiği üzerinden dosya oluşturan, duraklatılabilen, sürdürülebilen ve durumunu SQLite üzerinde koruyan bir masaüstü indirme yöneticisi olarak tasarlanmalıdır. Belgelerdeki en önemli karar, geliştirmeye tüm özellikleri aynı anda eklemek yerine ilk olarak uçtan uca çalışan dikey bir dilim oluşturmaktır.

İlk kanıtlanabilir başarı ölçütü şu akıştır: kullanıcı URL girer, Rust katmanı URL'yi doğrular, gerçek HTTP/HTTPS isteği başlatır, dosyayı akışlı biçimde diske yazar, gerçek ilerleme bilgisini yayınlar, kullanıcı indirmeyi duraklatır ve sürdürür, durum SQLite'a yazılır ve dosya tamamlanır. Bu akış çalışmadan gelişmiş UI, tarayıcı eklentisi veya ticari özelliklere geçilmemelidir.

## 2. Kapsamın yapılandırılması

| Katman | MVP 0.1 kapsamı | Sonraki sürüm |
|---|---|---|
| İndirme çekirdeği | HTTP/HTTPS, tek bağlantı, çoklu bağlantı, akışlı I/O, ilerleme, hız, ETA, duraklatma, sürdürme, iptal, yeniden deneme | Daha gelişmiş sunucu uyumluluğu ve akıllı optimizasyonlar |
| Kalıcılık | SQLite, migrations, downloads/segments/queues/settings, yeniden başlatma sonrası kurtarma | Daha kapsamlı geçmiş ve veri bakım araçları |
| Kuyruk | Kuyruk oluşturma, öncelik, eşzamanlı indirme sınırı, otomatik başlatma | Akıllı ve koşullu kuyruklar |
| Masaüstü UI | React/TypeScript/Vite, Tauri IPC, dashboard, liste/kartlar, ayarlar, açık/koyu/sistem teması | Daha gelişmiş etkileşim ve kişiselleştirme |
| Sistem entegrasyonu | Windows bildirimleri, dosya/klasör açma, güvenli dosya işlemleri | Kurulum, kod imzalama ve üretim güncelleme sistemi |
| Tarayıcı | Mimari hazırlık; eklenti MVP sonrasında | Chrome, Edge, Firefox ve Native Messaging |
| Uluslararasılaştırma | İngilizce kaynak dil, anahtar tabanlı UI | Türkçe ve diğer 10 dilin tamamlanması |
| Üretim hazırlığı | Loglama, güvenlik temeli, test altyapısı, dokümantasyon | Güvenlik denetimi, installer, signed update, lisans/Pro |

## 3. Önerilen mimari

Frontend yalnızca sunum, kullanıcı etkileşimi, çeviri ve frontend durumundan sorumlu olmalıdır. Ağ, dosya sistemi, indirme durumu, kalıcılık, zamanlama, hash doğrulama ve güvenlik açısından hassas işlemler Rust tarafında tutulmalıdır.

```text
apps/desktop/src                 React UI, stores, pages, IPC client
apps/desktop/src-tauri/src       Rust commands, core, engine, DB, security
apps/extension                   WebExtension; MVP çekirdeğinden sonra
packages/shared                  Ortak tipler, durumlar, IPC payload şemaları
packages/i18n                    Translation keys and locale resources
docs                             Architecture, security, decisions
scripts                           Local test server, build and release helpers
tests                             Cross-layer integration fixtures
```

Rust modülleri `commands`, `download`, `database`, `scheduler`, `browser`, `security` ve `utils` olarak ayrılmalıdır. `main.rs` yalnızca uygulama başlatma, plugin/state kurulumu ve command registration seviyesinde kalmalıdır.

## 4. Temel bağımlılıklar

| Alan | Bağımlılık/tercih | Kullanım amacı |
|---|---|---|
| Desktop shell | Tauri 2.x | Güvenli masaüstü kabuğu ve IPC |
| UI | React, TypeScript, Vite | Arayüz ve geliştirme altyapısı |
| Stil | Tailwind CSS, erişilebilir component sistemi, Lucide | Tutarlı ve erişilebilir UI |
| Rust async | Tokio | UI'yi bloklamayan workers ve cancellation |
| HTTP | Reqwest | HTTP/HTTPS, HEAD, Range ve streaming |
| DB | SQLite + SQLx benzeri Rust katmanı | Kalıcı durum ve migrations |
| Veri | Serde | IPC ve DB veri modelleri |
| Frontend state | Zustand veya eşdeğeri | IPC'den gelen gerçek durumun görünümü |
| Extension | WebExtension API + Native Messaging | Güvenli tarayıcı bağlantısı |

Bağımlılık sürümleri proje başlatılırken güncel ve birbiriyle uyumlu şekilde sabitlenmeli; gereksiz paket eklenmemelidir.

## 5. Öncelikli riskler ve karşılıkları

| Risk | Etki | Önleyici yaklaşım |
|---|---|---|
| Range desteklemeyen veya hatalı davranan sunucular | Çoklu bağlantı ve resume bozulabilir | Önce capability detection; destek yoksa tek bağlantıya güvenli fallback |
| Pause/resume sırasında dosya bozulması | Veri kaybı | Segment offset'lerini ve state'i atomik biçimde persist etmek; local HTTP test sunucusu kullanmak |
| Büyük dosyada RAM tüketimi | Performans ve kararlılık sorunu | Streaming I/O, bounded buffer ve rastgele dosya yazma |
| Eşzamanlı worker yarışları | Yanlış progress veya corrupt dosya | Merkezi state reducer, cancellation token ve segment başına açık sorumluluk |
| Uygulama kapanması/uyku | Yetim veya kurtarılamayan indirme | Shutdown hook, persisted offsets ve startup recovery |
| Güvenli olmayan dosya yolu | Path traversal ve veri üzerine yazma | Filename sanitization, izin verilen root doğrulaması ve overwrite politikası |
| IPC yüzeyinin genişlemesi | Yetki yükselmesi ve kötüye kullanım | Minimal Tauri permissions, typed commands ve frontend'e ham filesystem yetkisi vermeme |
| Sahte ilerleme veya mock veri | Ürün tanımının ihlali | UI yalnızca Rust event/store verisini göstermeli; mock production akışı olmamalı |
| Windows entegrasyon farkları | Kurulum ve bildirim sorunları | Windows üzerinde gerçek smoke test; cross-platform soyutlamaları erken korumak |
| NEXUS/ZYNERO adlandırma farkı | Paket, klasör ve ürün kimliği tutarsızlığı | Canonical product name olarak ZYNERO kullanmak; mevcut klasör adını yalnızca repository path kabul etmek |

## 6. Geliştirme sırası

Geliştirme, her adımın test edilebilir ve geri alınabilir olduğu küçük commit'lere bölünmelidir. İlk üç dikey kilometre taşı şöyledir:

1. **İskelet:** Tauri + React + TypeScript + Rust çalışma alanı, güvenli IPC iskeleti, ortak tipler ve temel dokümantasyon.
2. **Kalıcı çekirdek:** SQLite migrations, download/segment modelleri, state transition doğrulaması ve repository katmanı.
3. **İlk gerçek dikey dilim:** URL doğrulama, tek bağlantılı streaming download, gerçek progress events, pause/resume, persist ve local HTTP integration test.

Bu dilim doğrulanmadan çoklu bağlantı, kuyruk, ayarlar ve görsel cilaya geçilmemelidir. Sonrasında çoklu bağlantı, retry/recovery, kuyruk, UI sayfaları, bildirim, ayarlar, i18n, tarayıcı ve release hardening sırasıyla eklenmelidir.

## 7. Definition of Done

Bir görev yalnızca ekranı çizildiği için tamamlanmış sayılmayacaktır. Görev; gerçek backend davranışıyla uçtan uca çalışmalı, gerekli state'i kalıcılaştırmalı, hata durumlarını ele almalı, UI thread'ini bloklamamalı ve kritik mantık için test içermelidir. İndirme özellikleri ayrıca uygulama yeniden başlatıldıktan sonra mümkün olan durumlarda kurtarılmalıdır.

## 8. İlk başlatılacak görev

İlk görev **repository ve Tauri çalışma alanı iskeletinin oluşturulmasıdır**. Çıktı, boş veya sahte işlev içeren bir mockup değil; sonraki SQLite ve download engine çalışmalarını taşıyacak derlenebilir, modüler ve güvenli bir temel olmalıdır. İskelet hazırlandıktan sonra derleme/lint/test komutları çalıştırılacak ve ilk anlamlı commit oluşturulacaktır.
