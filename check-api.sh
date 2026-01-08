#!/bin/bash

# ============================================
# API MevzuatGPT - Health Check Script
# ============================================
# Bu script API'nin durumunu kontrol eder
# Log yazmaz, sadece ekrana yazdırır

# Renkli output için
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Yapılandırma
SERVICE_NAME="api-mevzuatgpt"
HEALTH_CHECK_URL="http://localhost:8080/api/health"

# Timestamp
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

echo "============================================"
echo "   API Health Check - $TIMESTAMP"
echo "============================================"
echo ""

# 1. Servis durumunu kontrol et
echo -n "Servis durumu kontrol ediliyor... "

if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo -e "${GREEN}✓ Çalışıyor${NC}"
else
    echo -e "${RED}✗ ÇALIŞMIYOR!${NC}"
    echo ""
    echo -e "${RED}[HATA]${NC} Systemd servisi aktif değil!"
    echo "Detay için: systemctl status $SERVICE_NAME"
    exit 1
fi

# 2. Health check endpoint kontrolü
echo -n "Health check endpoint kontrol ediliyor... "

HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$HEALTH_CHECK_URL" 2>/dev/null)

if [ "$HTTP_CODE" == "200" ]; then
    echo -e "${GREEN}✓ OK (HTTP $HTTP_CODE)${NC}"
    echo ""
    echo -e "${GREEN}[BAŞARILI]${NC} API sağlıklı çalışıyor! ✓"
else
    echo -e "${RED}✗ BAŞARISIZ (HTTP $HTTP_CODE)${NC}"
    echo ""
    echo -e "${RED}[HATA]${NC} Health check endpoint yanıt vermiyor!"
    echo "  - Beklenen: HTTP 200"
    echo "  - Alınan: HTTP $HTTP_CODE"
    echo "  - URL: $HEALTH_CHECK_URL"
    echo ""
    echo "Olası nedenler:"
    echo "  1. Servis başlatılırken hata oluştu"
    echo "  2. MongoDB bağlantısı kurulamıyor"
    echo "  3. Port çakışması var"
    echo ""
    echo "Kontrol için:"
    echo "  - Loglar: journalctl -u $SERVICE_NAME -n 50"
    echo "  - Servis: systemctl status $SERVICE_NAME"
    exit 1
fi

echo "============================================"

