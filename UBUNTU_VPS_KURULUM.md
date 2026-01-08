# Ubuntu VPS Kurulum Rehberi - API MevzuatGPT

Bu rehber, Rust tabanlı API MevzuatGPT uygulamasının Ubuntu VPS üzerinde kurulumu ve yapılandırılması için adım adım talimatları içerir.

## Ön Gereksinimler

- Ubuntu 20.04 LTS veya daha yeni bir sürüm
- Root veya sudo yetkisine sahip bir kullanıcı
- Çalışan bir MongoDB veritabanı (local veya remote)
- En az 2GB RAM
- En az 10GB disk alanı

---

## 1. Rust Kurulumu

### 1.1. Rustup ile Rust Kurulumu

```bash
# Rust'ı kurun
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Kurulum sırasında '1' seçeneğini seçin (default installation)
```

### 1.2. Ortam Değişkenlerini Yükleyin

```bash
source $HOME/.cargo/env
```

### 1.3. Rust Kurulumunu Doğrulayın

```bash
rustc --version
cargo --version
```

### 1.4. Gerekli Sistem Paketlerini Kurun

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git
```

---

## 2. Proje Dosyalarının Sunucuya Aktarılması

### Yöntem 1: Git ile Klonlama (Önerilen)

```bash
# Uygulamanın kurulacağı dizine gidin
cd /opt

# Projeyi klonlayın
sudo git clone https://github.com/KULLANICI_ADINIZ/api-mevzuatgpt.git

# Dizin sahipliğini ayarlayın
sudo chown -R $USER:$USER /opt/api-mevzuatgpt

# Proje dizinine gidin
cd /opt/api-mevzuatgpt
```

### Yöntem 2: SCP ile Dosya Aktarımı

Yerel bilgisayarınızdan:

```bash
# Proje dizinine gidin (yerel bilgisayarınızda)
cd /path/to/local/api-mevzuatgpt

# Dosyaları VPS'e aktarın
scp -r * kullanici@VPS_IP:/opt/api-mevzuatgpt/
```

VPS'te:

```bash
# Dizin sahipliğini ayarlayın
sudo chown -R $USER:$USER /opt/api-mevzuatgpt
```

---

## 3. Ortam Değişkenlerinin Yapılandırılması

### 3.1. .env Dosyası Oluşturun

```bash
cd /opt/api-mevzuatgpt
nano .env
```

### 3.2. Aşağıdaki İçeriği Ekleyin

```env
# Server Yapılandırması
PORT=8080
HOST=0.0.0.0

# MongoDB Bağlantısı
MONGODB_URI=mongodb://localhost:27017
# VEYA Remote MongoDB için:
# MONGODB_URI=mongodb://username:password@remote-server:27017

MONGODB_DB_NAME=mevzuatgpt

# Logging
RUST_LOG=info

# Production ortamı için:
# RUST_LOG=warn
```

### 3.3. Dosya İzinlerini Ayarlayın

```bash
chmod 600 .env
```

**ÖNEMLİ:** `.env` dosyasını asla git'e commit etmeyin!

---

## 4. MongoDB İndexlerinin Oluşturulması

### 4.1. MongoDB Shell'e Bağlanın

```bash
# Local MongoDB için:
mongosh

# Remote MongoDB için:
mongosh "mongodb://username:password@remote-server:27017"
```

### 4.2. Veritabanını Seçin

```javascript
use mevzuatgpt
```

### 4.3. İndexleri Oluşturun

Projedeki `mongodb_indexes.js` dosyasını kullanın:

```bash
# mongosh ile dosyayı çalıştırın
mongosh "mongodb://localhost:27017/mevzuatgpt" < mongodb_indexes.js

# VEYA manuel olarak her index'i oluşturun (mongosh içinde):
```

```javascript
// metadata koleksiyonu için indexler
db.metadata.createIndex(
    { "url_slug": 1 },
    { unique: true, name: "idx_url_slug_unique" }
);

db.metadata.createIndex(
    { "kurum_id": 1 },
    { name: "idx_kurum_id" }
);

db.metadata.createIndex(
    { "olusturulma_tarihi": -1 },
    { name: "idx_olusturulma_tarihi_desc" }
);

db.metadata.createIndex(
    { "belge_turu": 1 },
    { name: "idx_belge_turu" }
);

db.metadata.createIndex(
    { "pdf_adi": "text", "aciklama": "text" },
    { name: "idx_text_search" }
);

db.metadata.createIndex(
    { "kurum_id": 1, "olusturulma_tarihi": -1 },
    { name: "idx_kurum_tarih" }
);

// content koleksiyonu için indexler
db.content.createIndex(
    { "metadata_id": 1 },
    { name: "idx_content_metadata_id" }
);

// kurumlar koleksiyonu için indexler
db.kurumlar.createIndex(
    { "kurum_adi": 1 },
    { name: "idx_kurum_adi" }
);

// kurum_duyuru koleksiyonu için indexler
db.kurum_duyuru.createIndex(
    { "kurum_id": 1 },
    { name: "idx_duyuru_kurum_id" }
);

// links koleksiyonu için indexler
db.links.createIndex(
    { "kurum_id": 1 },
    { name: "idx_links_kurum_id" }
);
```

### 4.4. İndexleri Doğrulayın

```javascript
// Tüm indexleri görüntüleyin
db.metadata.getIndexes()
db.content.getIndexes()
db.kurumlar.getIndexes()
db.kurum_duyuru.getIndexes()
db.links.getIndexes()
```

---

## 5. Uygulamanın Derlenmesi

### 5.1. Release Modunda Derleyin

```bash
cd /opt/api-mevzuatgpt

# Release modunda derleyin (optimizasyonlu)
cargo build --release
```

Bu işlem ilk seferde 5-15 dakika sürebilir (VPS kaynaklarına bağlı olarak).

### 5.2. Derlenen Binary'yi Kontrol Edin

```bash
ls -lh target/release/api-mevzuatgpt
```

### 5.3. Test Çalıştırması (Opsiyonel)

```bash
# Uygulamayı test etmek için:
./target/release/api-mevzuatgpt

# Başka bir terminal'de test edin:
curl http://localhost:8080/api/health
```

Ctrl+C ile durdurun.

---

## 6. Systemd Servisi Oluşturma (Production)

### 6.1. Systemd Service Dosyası Oluşturun

```bash
sudo nano /etc/systemd/system/api-mevzuatgpt.service
```

### 6.2. Aşağıdaki İçeriği Ekleyin

```ini
[Unit]
Description=API MevzuatGPT - Rust Backend Service
After=network.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/api-mevzuatgpt
Environment="RUST_LOG=info"
EnvironmentFile=/opt/api-mevzuatgpt/.env
ExecStart=/opt/api-mevzuatgpt/target/release/api-mevzuatgpt
Restart=always
RestartSec=5
StandardOutput=append:/var/log/api-mevzuatgpt/app.log
StandardError=append:/var/log/api-mevzuatgpt/error.log

# Güvenlik ayarları
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/api-mevzuatgpt
ReadWritePaths=/var/log/api-mevzuatgpt

[Install]
WantedBy=multi-user.target
```

### 6.3. Log Dizini Oluşturun

```bash
sudo mkdir -p /var/log/api-mevzuatgpt
sudo chown www-data:www-data /var/log/api-mevzuatgpt
```

### 6.4. Dizin İzinlerini Ayarlayın

```bash
sudo chown -R www-data:www-data /opt/api-mevzuatgpt
```

### 6.5. Servisi Etkinleştirin ve Başlatın

```bash
# Systemd'yi yeniden yükleyin
sudo systemctl daemon-reload

# Servisi etkinleştirin (boot'ta otomatik başlasın)
sudo systemctl enable api-mevzuatgpt

# Servisi başlatın
sudo systemctl start api-mevzuatgpt

# Servisi durumunu kontrol edin
sudo systemctl status api-mevzuatgpt
```

### 6.6. Servis Komutları

```bash
# Servisi başlatma
sudo systemctl start api-mevzuatgpt

# Servisi durdurma
sudo systemctl stop api-mevzuatgpt

# Servisi yeniden başlatma
sudo systemctl restart api-mevzuatgpt

# Servisi yeniden yükleme (kod güncellemesi sonrası)
sudo systemctl reload api-mevzuatgpt

# Servis durumunu görüntüleme
sudo systemctl status api-mevzuatgpt

# Servis loglarını görüntüleme
sudo journalctl -u api-mevzuatgpt -f
```

---

## 7. Güvenlik ve Firewall Ayarları

### 7.1. UFW Firewall Kurulumu

```bash
# UFW'yi kurun (genellikle varsayılan olarak yüklüdür)
sudo apt install ufw

# SSH portunu açın (kilitleme olmasın!)
sudo ufw allow 22/tcp

# API portunu açın (sadece gerekirse)
# Eğer Nginx kullanıyorsanız bu gerekli değil
sudo ufw allow 8080/tcp

# Firewall'u etkinleştirin
sudo ufw enable

# Durumu kontrol edin
sudo ufw status
```

### 7.2. Port Yapılandırması

**Önemli Notlar:**

- **8080 portunun internete açık olması gerekiyorsa:** `sudo ufw allow 8080/tcp`
- **Eğer Nginx kullanıyorsanız:** 8080 portunu kapatın, sadece Nginx'in erişmesine izin verin
- **Sadece belirli IP'lerden erişim için:**
  ```bash
  sudo ufw allow from GÜVENLI_IP_ADRESI to any port 8080
  ```

### 7.3. Fail2ban Kurulumu (Opsiyonel ama Önerilen)

```bash
# Fail2ban'i kurun
sudo apt install fail2ban

# Fail2ban'i başlatın
sudo systemctl enable fail2ban
sudo systemctl start fail2ban
```

### 7.4. Otomatik Güvenlik Güncellemeleri

```bash
# Unattended-upgrades'i kurun
sudo apt install unattended-upgrades

# Etkinleştirin
sudo dpkg-reconfigure -plow unattended-upgrades
```

---

## 8. Log Yönetimi ve İzleme

### 8.1. Log Dosyalarını Görüntüleme

```bash
# Uygulama logları
sudo tail -f /var/log/api-mevzuatgpt/app.log

# Hata logları
sudo tail -f /var/log/api-mevzuatgpt/error.log

# Systemd journalctl ile
sudo journalctl -u api-mevzuatgpt -f

# Son 100 satır
sudo journalctl -u api-mevzuatgpt -n 100

# Bugünkü loglar
sudo journalctl -u api-mevzuatgpt --since today
```

### 8.2. Logrotate Yapılandırması

Logların otomatik olarak arşivlenmesi için:

```bash
sudo nano /etc/logrotate.d/api-mevzuatgpt
```

İçeriği:

```
/var/log/api-mevzuatgpt/*.log {
    daily
    rotate 14
    compress
    delaycompress
    missingok
    notifempty
    create 0640 www-data www-data
    sharedscripts
    postrotate
        systemctl reload api-mevzuatgpt > /dev/null 2>&1 || true
    endscript
}
```

### 8.3. Basit İzleme Scripti

```bash
nano ~/check-api.sh
```

İçeriği:

```bash
#!/bin/bash

# API'nin çalışıp çalışmadığını kontrol et
HEALTH_CHECK=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/api/health)

if [ "$HEALTH_CHECK" != "200" ]; then
    echo "[$(date)] API yanıt vermiyor! HTTP Status: $HEALTH_CHECK" >> /var/log/api-mevzuatgpt/monitor.log
    # Servisi yeniden başlat
    sudo systemctl restart api-mevzuatgpt
else
    echo "[$(date)] API çalışıyor - OK" >> /var/log/api-mevzuatgpt/monitor.log
fi
```

Çalıştırılabilir yapın:

```bash
chmod +x ~/check-api.sh
```

Cron ile her 5 dakikada bir çalıştırın:

```bash
crontab -e
```

Ekleyin:

```
*/5 * * * * /home/KULLANICI_ADINIZ/check-api.sh
```

### 8.4. Kaynak Kullanımını İzleme

```bash
# CPU ve Memory kullanımı
top -p $(pgrep -f api-mevzuatgpt)

# VEYA daha detaylı
htop -p $(pgrep -f api-mevzuatgpt)

# Servis kaynak kullanımı
systemctl status api-mevzuatgpt

# Detaylı sistem metrikleri
journalctl -u api-mevzuatgpt --since "1 hour ago" | grep -i "memory\|cpu"
```

---

## 9. Kod Güncellemeleri ve Deployment

### 9.1. Güncelleme İşlemi

```bash
cd /opt/api-mevzuatgpt

# Kodu çekin (git kullanıyorsanız)
sudo -u www-data git pull

# Yeniden derleyin
sudo -u www-data cargo build --release

# Servisi yeniden başlatın
sudo systemctl restart api-mevzuatgpt

# Durumu kontrol edin
sudo systemctl status api-mevzuatgpt
```

### 9.2. Rollback (Geri Alma)

```bash
cd /opt/api-mevzuatgpt

# Önceki commit'e dönün
sudo -u www-data git log --oneline  # Commit hash'i bulun
sudo -u www-data git checkout COMMIT_HASH

# Yeniden derleyin
sudo -u www-data cargo build --release

# Servisi yeniden başlatın
sudo systemctl restart api-mevzuatgpt
```

### 9.3. Zero-Downtime Deployment (İleri Seviye)

```bash
# Yeni binary'yi farklı bir isimle derleyin
cargo build --release
mv target/release/api-mevzuatgpt target/release/api-mevzuatgpt-new

# Eski binary'yi yedekleyin
cp target/release/api-mevzuatgpt-old target/release/api-mevzuatgpt-backup

# Yeni binary'yi aktif edin
mv target/release/api-mevzuatgpt-new target/release/api-mevzuatgpt

# Graceful restart
sudo systemctl reload-or-restart api-mevzuatgpt
```

---

## 10. Sorun Giderme

### 10.1. Servis Başlamıyor

```bash
# Detaylı hata mesajlarını görün
sudo journalctl -u api-mevzuatgpt -xe

# Manuel çalıştırarak test edin
cd /opt/api-mevzuatgpt
./target/release/api-mevzuatgpt
```

### 10.2. MongoDB Bağlantı Hatası

```bash
# .env dosyasını kontrol edin
cat /opt/api-mevzuatgpt/.env

# MongoDB'nin çalıştığını doğrulayın
sudo systemctl status mongod  # Local MongoDB için

# Bağlantıyı test edin
mongosh "MONGODB_URI"
```

### 10.3. Port Zaten Kullanımda

```bash
# Hangi process 8080 portunu kullanıyor?
sudo lsof -i :8080

# VEYA
sudo netstat -tulpn | grep 8080

# Process'i öldürün (gerekirse)
sudo kill -9 PID
```

### 10.4. İzin Hataları

```bash
# Dizin sahipliğini düzeltin
sudo chown -R www-data:www-data /opt/api-mevzuatgpt

# Log dizini izinleri
sudo chown -R www-data:www-data /var/log/api-mevzuatgpt
```

### 10.5. Yüksek Memory Kullanımı

```bash
# Rust log seviyesini düşürün
# .env dosyasında:
RUST_LOG=warn  # info yerine

# Servisi yeniden başlatın
sudo systemctl restart api-mevzuatgpt
```

---

## 11. Performans İyileştirmeleri

### 11.1. Rust Optimizasyonları

`Cargo.toml` dosyasına ekleyin:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

Yeniden derleyin:

```bash
cargo build --release
```

### 11.2. Sistem Limitleri

```bash
# Dosya için limits.conf düzenleyin
sudo nano /etc/security/limits.conf
```

Ekleyin:

```
www-data soft nofile 65536
www-data hard nofile 65536
```

### 11.3. MongoDB Connection Pool

`.env` dosyasında:

```env
MONGODB_URI=mongodb://localhost:27017/?maxPoolSize=20&minPoolSize=5
```

---

## 12. Yedekleme Stratejisi

### 12.1. Uygulama Binary Yedeği

```bash
# Cron ile günlük yedek
crontab -e
```

Ekleyin:

```
0 2 * * * cp /opt/api-mevzuatgpt/target/release/api-mevzuatgpt /opt/backups/api-mevzuatgpt-$(date +\%Y\%m\%d)
```

### 12.2. .env Dosyası Yedeği

```bash
# Güvenli bir yere yedekleyin
sudo cp /opt/api-mevzuatgpt/.env /root/backups/.env.backup
sudo chmod 600 /root/backups/.env.backup
```

---

## Özet Checklist

Kurulum tamamlandıktan sonra kontrol listesi:

- [ ] Rust kuruldu ve çalışıyor
- [ ] Proje dosyaları VPS'e aktarıldı
- [ ] `.env` dosyası oluşturuldu ve yapılandırıldı
- [ ] MongoDB indexleri oluşturuldu
- [ ] Uygulama release modunda derlendi
- [ ] Systemd servisi oluşturuldu ve çalışıyor
- [ ] Firewall yapılandırıldı
- [ ] Log dosyaları kontrol edildi
- [ ] Health check endpoint test edildi (`curl http://localhost:8080/api/health`)
- [ ] Logrotate yapılandırıldı
- [ ] İzleme scripti kuruldu (opsiyonel)

---

## Yararlı Komutlar

```bash
# Servis durumu
sudo systemctl status api-mevzuatgpt

# Logları izle (canlı)
sudo journalctl -u api-mevzuatgpt -f

# Servisi yeniden başlat
sudo systemctl restart api-mevzuatgpt

# API test et
curl http://localhost:8080/api/health

# Kaynak kullanımı
top -p $(pgrep -f api-mevzuatgpt)

# Disk kullanımı
du -sh /opt/api-mevzuatgpt
```

---

## Destek ve İletişim

Sorun yaşarsanız:

1. Logları kontrol edin: `sudo journalctl -u api-mevzuatgpt -xe`
2. Uygulama loglarını inceleyin: `/var/log/api-mevzuatgpt/`
3. GitHub Issues'da rapor edin

---

**Son Güncelleme:** 2026-01-08

