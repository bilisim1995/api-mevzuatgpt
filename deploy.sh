#!/bin/bash

# ============================================
# API MevzuatGPT - Deployment Script
# ============================================
# Bu script uygulamayı derler ve production'a deploy eder
# Git işlemleri YAPILMAZ - sadece build ve deploy

set -e  # Hata durumunda scripti durdur

# Renkli output için
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Yapılandırma
PROJECT_DIR="/opt/api-mevzuatgpt"
SERVICE_NAME="api-mevzuatgpt"
BINARY_PATH="target/release/api-mevzuatgpt"
HEALTH_CHECK_URL="http://localhost:8080/api/health"
MAX_HEALTH_CHECK_ATTEMPTS=30
HEALTH_CHECK_INTERVAL=2

# Log fonksiyonları
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Başlık
echo "============================================"
echo "   API MevzuatGPT Deployment Script"
echo "============================================"
echo ""

# 1. Cargo kontrolü ve PATH yükleme
log_info "Cargo kontrolü yapılıyor..."

# Cargo'yu bulmaya çalış
if ! command -v cargo &> /dev/null; then
    log_warning "Cargo bulunamadı, Rust PATH yükleniyor..."
    
    # Standart Rust kurulum yollarını dene
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
        log_success "Cargo PATH yüklendi: $HOME/.cargo/env"
    elif [ -f "/root/.cargo/env" ]; then
        source "/root/.cargo/env"
        log_success "Cargo PATH yüklendi: /root/.cargo/env"
    else
        log_error "Cargo bulunamadı!"
        log_error "Rust'ın kurulu olduğundan emin olun:"
        log_error "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Tekrar kontrol et
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo PATH yüklendikten sonra da bulunamadı!"
        exit 1
    fi
fi

CARGO_VERSION=$(cargo --version)
log_success "Cargo bulundu: $CARGO_VERSION"
echo ""

# 2. Dizin kontrolü
log_info "Proje dizini kontrol ediliyor..."
if [ ! -d "$PROJECT_DIR" ]; then
    log_error "Proje dizini bulunamadı: $PROJECT_DIR"
    exit 1
fi

cd "$PROJECT_DIR"
log_success "Proje dizinine geçildi: $PROJECT_DIR"
echo ""

# 3. .env dosyası kontrolü
log_info ".env dosyası kontrol ediliyor..."
if [ ! -f ".env" ]; then
    log_error ".env dosyası bulunamadı!"
    exit 1
fi
log_success ".env dosyası mevcut"
echo ""

# 4. Uygulamayı derle
log_info "Uygulama derleniyor (release mode)..."
echo "Bu işlem birkaç dakika sürebilir..."
echo ""

if cargo build --release; then
    log_success "Derleme başarılı!"
else
    log_error "Derleme başarısız oldu!"
    exit 1
fi
echo ""

# 5. Binary boyutunu göster
if [ -f "$BINARY_PATH" ]; then
    BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    log_success "Binary boyutu: $BINARY_SIZE"
fi
echo ""

# 6. Systemd servisini yeniden başlat
log_info "Systemd servisi yeniden başlatılıyor..."

if systemctl restart "$SERVICE_NAME"; then
    log_success "Servis başarıyla yeniden başlatıldı"
else
    log_error "Servis yeniden başlatılamadı!"
    exit 1
fi
echo ""

# 7. Servis durumunu kontrol et
log_info "Servis durumu kontrol ediliyor..."
sleep 2  # Servisin başlaması için kısa bir bekleme

if systemctl is-active --quiet "$SERVICE_NAME"; then
    log_success "Servis aktif durumda"
else
    log_error "Servis çalışmıyor!"
    log_info "Servis logları:"
    journalctl -u "$SERVICE_NAME" -n 20 --no-pager
    exit 1
fi
echo ""

# 8. Health check
log_info "Health check yapılıyor..."
HEALTH_CHECK_SUCCESS=false

for i in $(seq 1 $MAX_HEALTH_CHECK_ATTEMPTS); do
    HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$HEALTH_CHECK_URL" 2>/dev/null || echo "000")
    
    if [ "$HTTP_CODE" == "200" ]; then
        log_success "Health check başarılı! (HTTP $HTTP_CODE)"
        HEALTH_CHECK_SUCCESS=true
        break
    else
        if [ $i -eq $MAX_HEALTH_CHECK_ATTEMPTS ]; then
            log_error "Health check başarısız! (HTTP $HTTP_CODE)"
            log_info "Servis logları:"
            journalctl -u "$SERVICE_NAME" -n 20 --no-pager
        else
            echo -n "."
            sleep $HEALTH_CHECK_INTERVAL
        fi
    fi
done
echo ""

if [ "$HEALTH_CHECK_SUCCESS" = false ]; then
    log_error "API yanıt vermiyor!"
    log_info "Servis loglarını kontrol edin: journalctl -u $SERVICE_NAME -n 50"
    exit 1
fi
echo ""

# 9. Deployment özeti
echo "============================================"
echo "   DEPLOYMENT BAŞARILI! ✓"
echo "============================================"
echo ""
log_info "Özet:"
echo "  - Derleme: Başarılı"
echo "  - Binary boyutu: $BINARY_SIZE"
echo "  - Servis durumu: Aktif"
echo "  - Health check: OK"
echo ""
log_info "Faydalı komutlar:"
echo "  - Logları izle: journalctl -u $SERVICE_NAME -f"
echo "  - Servis durumu: systemctl status $SERVICE_NAME"
echo "  - Son loglar: tail -f /var/log/api-mevzuatgpt/app.log"
echo ""
log_success "Deployment tamamlandı! 🚀"

