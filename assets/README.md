# NEXUS Sort — görsel varlıklar

Bu klasör, mobil oyunun isteğe bağlı PNG varlıklarını barındırır. Kod bu dosyaları başlangıçta `load_texture()` ile yüklemeyi dener; dosya henüz yoksa oyun mevcut çizimlerle çalışmaya devam eder.

Dosyaları şeffaf arka planlı PNG olarak şu isimlerle ekleyin:

- `menu_bg.png` — ana menü, tam ekran arka plan
- `map_bg.png` — macera haritası arka planı
- `board_bg.png` — oyun/tahta arka planı
- `gem_red.png`, `gem_blue.png`, `gem_green.png`, `gem_purple.png`, `gem_yellow.png` — ayrı kristaller
- `button_green.png` — yeşil, altın kenarlıklı buton
- `panel_blue.png` — metin/panel arka planı
- `level_complete.png` — bölüm tamamlandı paneli
- `power_hammer.png`, `power_bomb.png`, `power_rainbow.png`, `power_shuffle.png` — güçlendirici ikonları

Önerilen ölçüler: arka planlar 1080×1920 veya 9:16 oranında; kristal ve güçlendirici ikonları kare (256×256 veya 512×512); buton/panel görselleri yatay PNG. Arka planlar ekrana kırpılarak sığdırılır; ikonlar mevcut alanlarına ölçeklenir.

Not: Bu commit dosya yükleme altyapısını ve isim sözleşmesini ekler; önceki görsel üretim çıktıları bu depoya binary olarak henüz eklenmedi.
