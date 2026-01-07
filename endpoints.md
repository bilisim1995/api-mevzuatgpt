# API Endpoints Dokümantasyonu

## Base URL
```
http://localhost:8080/api
```

---

## 1. Health Check

### Endpoint
```
GET /api/health
```

### Request
```
GET /api/health
```

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK) - Sunucu ve MongoDB Bağlantısı Başarılı**
```json
{
  "success": true,
  "server": {
    "status": "running",
    "message": "Sunucu çalışıyor"
  },
  "mongodb": {
    "status": "connected",
    "message": "MongoDB bağlantısı başarılı"
  },
  "message": "Sunucu ve MongoDB bağlantısı başarılı"
}
```

**Success (200 OK) - MongoDB Bağlantısı Başarısız**
```json
{
  "success": false,
  "server": {
    "status": "running",
    "message": "Sunucu çalışıyor"
  },
  "mongodb": {
    "status": "disconnected",
    "message": "MongoDB bağlantı hatası: ..."
  },
  "message": "Sunucu çalışıyor ancak MongoDB bağlantısı başarısız"
}
```

---

## 2. Kurumlar Listesi

### Endpoint
```
GET /api/v1/institutions
```

### Request
```
GET /api/v1/institutions
```

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "kurum_id": "68bbf6df8ef4e8023c19641d",
      "kurum_adi": "Sosyal Güvenlik Kurumu",
      "kurum_logo": "https://cdn.mevzuatgpt.org/portal/kurum_Sosyal%20G%C3%BCvenlik%20Kurumu_b98c7ed9-4bfc-42af-8cbf-550bd9e1885f.svg",
      "kurum_aciklama": "sgk.gov.tr",
      "detsis": "22620739"
    },
    {
      "kurum_id": "another-kurum-id",
      "kurum_adi": "Kurum Adı 2",
      "kurum_logo": "https://example.com/logo2.png",
      "kurum_aciklama": "Başka bir kurum açıklaması",
      "detsis": "67890"
    }
  ],
  "count": 2,
  "message": "İşlem başarılı"
}
```

**Error (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": null,
  "message": "Kurumlar listesi alınamadı"
}
```

---

## 3. Belgeler Listesi

### Endpoint
```
GET /api/v1/documents
```

### Request
```
GET /api/v1/documents?kurum_id={id}&limit=12&offset=0&sort_by=olusturulma_tarihi&sort_order=desc&belge_turu=Genelge&etiketler=2024
```

**Query Parameters:**
- `kurum_id` (opsiyonel): Kurum ID'si ile filtreleme
- `limit` (opsiyonel, varsayılan: 10000): Maksimum kayıt sayısı
- `offset` (opsiyonel, varsayılan: 0): Sayfalama için atlanacak kayıt sayısı
- `sort_by` (opsiyonel, varsayılan: olusturulma_tarihi): Sıralama alanı
- `sort_order` (opsiyonel, varsayılan: desc): Sıralama yönü (asc/desc)
- `belge_turu` (opsiyonel): Belge türüne göre filtreleme (tam eşleşme)
- `etiketler` (opsiyonel): Etiketlere göre filtreleme (virgülle ayrılmış string içinde arama, case-insensitive)

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "url_slug": "belge-url-slug",
      "pdf_adi": "Belge Başlığı",
      "aciklama": "Belge açıklaması",
      "belge_yayin_tarihi": "2024-01-15",
      "belge_turu": "Genelge",
      "belge_durumu": "Yürürlükte",
      "etiketler": "etiket1, etiket2, etiket3",
      "anahtar_kelimeler": "anahtar1, anahtar2",
      "pdf_url": "https://example.com/document.pdf"
    },
    {
      "url_slug": "belge-url-slug-2",
      "pdf_adi": "Başka Bir Belge",
      "aciklama": "Başka bir belge açıklaması",
      "belge_yayin_tarihi": "2024-01-10",
      "belge_turu": "Yönetmelik",
      "belge_durumu": "Değiştirilmiş",
      "etiketler": "etiket4, etiket5",
      "anahtar_kelimeler": "anahtar3",
      "pdf_url": "https://example.com/document2.pdf"
    }
  ],
  "count": 150,
  "message": "İşlem başarılı"
}
```

**Error (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": null,
  "message": "Belgeler alınamadı"
}
```

---

## 3.1. Belgeler Filtre Listesi

### Endpoint
```
GET /api/v1/documents/filters
```

### Request
```
GET /api/v1/documents/filters?kurum_id={id}
```

**Query Parameters:**
- `kurum_id` (opsiyonel): Kurum ID'si ile filtreleme (belirtilirse sadece o kuruma ait unique değerler döner)

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": {
    "belge_turu": [
      "Genelge",
      "Yönetmelik",
      "Tebliğ",
      "Kanun",
      "Karar",
      "Yönerge",
      "Belirtilmemiş"
    ],
    "etiketler": [
      "2024",
      "2025",
      "duyuru",
      "güncelleme",
      "yeni",
      "önemli"
    ]
  },
  "message": "Filtre listeleri başarıyla alındı",
  "error": null
}
```

**Error (500 Internal Server Error)**
```json
{
  "success": false,
  "data": {
    "belge_turu": [],
    "etiketler": []
  },
  "message": null,
  "error": "Belge türü listesi alınamadı"
}
```

**Notlar:**
- `belge_turu` listesi alfabetik olarak sıralanır
- `etiketler` listesi virgülle ayrılmış string'lerden parse edilir ve unique değerler alfabetik olarak sıralanır
- Boş `belge_turu` değerleri "Belirtilmemiş" olarak gösterilir
- `kurum_id` belirtilirse sadece o kuruma ait belgelerden unique değerler çıkarılır

---

## 4. Duyurular Listesi

### Endpoint
```
GET /api/v1/announcements
```

### Request
```
GET /api/v1/announcements?kurum_id={id}
```

**Query Parameters:**
- `kurum_id` (opsiyonel): Kurum ID'si ile filtreleme

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "baslik": "Duyuru Başlığı 1",
      "link": "https://example.com/duyuru/1",
      "tarih": "2024-01-15"
    },
    {
      "baslik": "Duyuru Başlığı 2",
      "link": "https://example.com/duyuru/2",
      "tarih": "2024-01-10"
    }
  ],
  "count": 2,
  "message": "İşlem başarılı"
}
```

**Boş Liste (200 OK)**
```json
{
  "success": true,
  "data": [],
  "count": 0,
  "message": "İşlem başarılı"
}
```

**Error (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": null,
  "message": "Duyurular alınamadı"
}
```

---

## 5. Linkler Listesi

### Endpoint
```
GET /api/v1/links
```

### Request
```
GET /api/v1/links?kurum_id=507f1f77bcf86cd799439020
```

**Query Parameters:**
- `kurum_id` (zorunlu): Kurum ID'si (MongoDB ObjectID hex formatında)

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "id": "507f1f77bcf86cd799439011",
      "baslik": "E-Devlet Girişi",
      "aciklama": "Kurumun e-devlet üzerinden hizmetlerine erişim",
      "url": "https://www.turkiye.gov.tr/kurum-hizmetleri",
      "kurum_id": "507f1f77bcf86cd799439020",
      "created_at": "2024-01-15T10:30:00Z"
    },
    {
      "id": "507f1f77bcf86cd799439012",
      "baslik": "Online Başvuru Sistemi",
      "aciklama": "Dijital başvuru ve takip sistemi",
      "url": "https://basvuru.kurum.gov.tr",
      "kurum_id": "507f1f77bcf86cd799439020",
      "created_at": "2024-01-20T14:15:00Z"
    },
    {
      "id": "507f1f77bcf86cd799439013",
      "baslik": "Duyurular Sayfası",
      "aciklama": "Kurum duyuru ve haberleri",
      "url": "https://www.kurum.gov.tr/duyurular",
      "kurum_id": "507f1f77bcf86cd799439020",
      "created_at": "2024-02-01T09:00:00Z"
    }
  ],
  "count": 3,
  "message": "Kurum linkleri başarıyla çekildi",
  "error": null
}
```

**Boş Liste (200 OK)**
```json
{
  "success": true,
  "data": [],
  "count": 0,
  "message": "Kurum linkleri başarıyla çekildi",
  "error": null
}
```

**Error - kurum_id Parametresi Eksik (400 Bad Request)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "kurum_id parameter is required"
}
```

**Error - Geçersiz kurum_id Formatı (400 Bad Request)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Invalid kurum_id format"
}
```

**Error - Linkler Çekilemedi (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Failed to fetch links"
}
```

**Error - Veri Decode Hatası (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Failed to decode links"
}
```

---

## 6. Son Yüklenen Mevzuatlar

### Endpoint
```
GET /api/v1/regulations/recent
```

### Request
```
GET /api/v1/regulations/recent?limit=50
```

**Query Parameters:**
- `limit` (opsiyonel, varsayılan: 50): Maksimum kayıt sayısı (maksimum 1000)

**Not:** Kayıtlar her zaman `olusturulma_tarihi` alanına göre azalan (desc) sırada döner (en yeni önce).

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "pdf_adi": "2016-21 - Kısa Vadeli Sigorta Kolları Uygulamaları",
      "kurum_adi": "Sosyal Güvenlik Kurumu",
      "aciklama": "T C Sosyal Güvenli k Kurumu Emeklilik Hizmetleri Genel Müdürlüğü 2016 21 SAYILI GENELGE...",
      "olusturulma_tarihi": "2025-09-06 12:50:09",
      "belge_turu": "Genelge",
      "url_slug": "2016-21-kisa-vadeli-sigorta-kollari-uygulamalari"
    }
  ],
  "count": 50,
  "message": "İşlem başarılı",
  "error": null
}
```

**Error (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Mevzuatlar alınamadı"
}
```

---

## 7. İstatistikler

### Endpoint
```
GET /api/v1/statistics
```

### Request
```
GET /api/v1/statistics
```

**Query Parameters:** Yok

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": {
    "total_kurumlar": 150,
    "total_belgeler": 12500,
    "belge_turu_istatistik": [
      {
        "belge_turu": "Kanun",
        "count": 3500
      },
      {
        "belge_turu": "Yönetmelik",
        "count": 2800
      },
      {
        "belge_turu": "Tebliğ",
        "count": 2100
      },
      {
        "belge_turu": "Genelge",
        "count": 1800
      },
      {
        "belge_turu": "Karar",
        "count": 1200
      },
      {
        "belge_turu": "Yönerge",
        "count": 800
      },
      {
        "belge_turu": "Belirtilmemiş",
        "count": 300
      }
    ]
  },
  "message": "Statistics fetched successfully",
  "error": null
}
```

**Error - Belgeler Sayılamadı (500 Internal Server Error)**
```json
{
  "success": false,
  "data": null,
  "message": null,
  "error": "Failed to count documents"
}
```

**Error - Belge Türü İstatistikleri Alınamadı (500 Internal Server Error)**
```json
{
  "success": false,
  "data": null,
  "message": null,
  "error": "Failed to aggregate document types"
}
```

---

## 8. Kurum Duyuruları (Web Scraping)

### Endpoint
```
GET /api/v1/kurum-duyuru
```

### Request
```
GET /api/v1/kurum-duyuru?kurum_id=507f1f77bcf86cd799439020
```

**Query Parameters:**
- `kurum_id` (zorunlu): Kurum ID'si (string formatında)

**Headers:** Yok

**Body:** Yok

### Endpoint Açıklaması

Belirtilen kurumun web sitesinden duyuruları çeker. Kurumun `kurum_duyuru` koleksiyonundaki `duyuru_linki` kullanılarak web scraping yapılır.

**Desteklenen Kurumlar:**
- Yargıtay (`yargitay.gov.tr`)
- SGK (`sgk.gov.tr`)
- İşkur (`iskur.gov.tr`)
- Diğer: Varsayılan olarak Yargıtay scraper'ı kullanılır

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "baslik": "2024 Yılı Yargıtay Genel Kurul Kararları",
      "link": "https://www.yargitay.gov.tr/item/12345/duyuru-detay",
      "tarih": "15.01.2024"
    },
    {
      "baslik": "Yargıtay Daire Başkanları Toplantısı Duyurusu",
      "link": "https://www.yargitay.gov.tr/item/12346/duyuru-detay",
      "tarih": "14.01.2024"
    },
    {
      "baslik": "Yeni İçtihat Kararları Yayınlandı",
      "link": "https://www.yargitay.gov.tr/item/12347/duyuru-detay",
      "tarih": "13.01.2024"
    },
    {
      "baslik": "Yargıtay Personel Alım İlanı",
      "link": "https://www.yargitay.gov.tr/item/12348/duyuru-detay",
      "tarih": "12.01.2024"
    },
    {
      "baslik": "Yargıtay Yıllık Faaliyet Raporu",
      "link": "https://www.yargitay.gov.tr/item/12349/duyuru-detay",
      "tarih": "10.01.2024"
    }
  ],
  "count": 5,
  "message": "Kurum duyuruları başarıyla çekildi",
  "error": null
}
```

**SGK Örneği**
```json
{
  "success": true,
  "data": [
    {
      "baslik": "SGK Prim Ödemeleri Hakkında Duyuru",
      "link": "https://www.sgk.gov.tr/Duyuru/Detay/12345",
      "tarih": "23.09.2025"
    },
    {
      "baslik": "Emeklilik Başvuru Süreçleri Güncellendi",
      "link": "https://www.sgk.gov.tr/Duyuru/Detay/12346",
      "tarih": "20.09.2025"
    }
  ],
  "count": 2,
  "message": "Kurum duyuruları başarıyla çekildi",
  "error": null
}
```

**İşkur Örneği**
```json
{
  "success": true,
  "data": [
    {
      "baslik": "Yeni İş İmkanları Duyurusu",
      "link": "https://www.iskur.gov.tr/duyurular/12345",
      "tarih": "11.08.2025"
    },
    {
      "baslik": "Mesleki Eğitim Programları",
      "link": "https://www.iskur.gov.tr/duyurular/12346",
      "tarih": "27.07.2025"
    }
  ],
  "count": 2,
  "message": "Kurum duyuruları başarıyla çekildi",
  "error": null
}
```

**Boş Liste (200 OK)**
```json
{
  "success": true,
  "data": [],
  "count": 0,
  "message": "Kurum duyuruları başarıyla çekildi",
  "error": null
}
```

**Error - kurum_id Parametresi Eksik (400 Bad Request)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "kurum_id parameter is required"
}
```

**Error - Kurum Duyuru Linki Bulunamadı (404 Not Found)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Kurum duyuru linki bulunamadı"
}
```

**Error - Duyuru Linki Tanımlanmamış (404 Not Found)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Kurum için duyuru linki tanımlanmamış"
}
```

**Error - Web Scraping Hatası (500 Internal Server Error)**
```json
{
  "success": false,
  "data": [],
  "count": 0,
  "message": null,
  "error": "Duyuru sayfası çekilemedi"
}
```

### Özellikler

1. **Zorunlu parametre**: `kurum_id` parametresi zorunludur
2. **Web scraping**: Kurumun web sitesinden gerçek zamanlı duyurular çekilir
3. **Limit**: En fazla 5 duyuru döndürülür
4. **Otomatik domain algılama**: URL'deki domain'e göre uygun scraper seçilir
5. **HTML temizleme**: Başlıklar HTML tag'lerinden ve entity'lerden temizlenir
6. **Tarih çıkarımı**: HTML içinden tarih bilgisi otomatik çıkarılır
7. **Link normalizasyonu**: Relative linkler mutlak URL'ye dönüştürülür
8. **Navigasyon filtresi**: Menü linkleri otomatik filtrelenir
9. **Tekrar önleme**: Aynı link birden fazla kez döndürülmez

### Tarih Formatları

Endpoint farklı tarih formatlarını destekler:

- **Yargıtay**: `DD.MM.YYYY` veya `DD/MM/YYYY`
- **SGK**: `DD Ay YYYY` (örn: "23 Eylül 2025") → `DD.MM.YYYY` formatına dönüştürülür
- **İşkur**: `DD Ay Kısaltma YYYY` (örn: "11 Ağu 2025") → `DD.MM.YYYY` formatına dönüştürülür
- **Varsayılan**: Tarih bulunamazsa bugünün tarihi kullanılır

### Notlar

- `kurum_id` string formatında olmalıdır
- Duyurular her istekte web sitesinden çekilir (cache yok)
- Timeout: 30 saniye
- HTTP client timeout: 15 saniye
- Başlıklar minimum 10-15 karakter olmalıdır
- Navigasyon linkleri (ana sayfa, menü vb.) otomatik filtrelenir
- Türkçe karakterler doğru şekilde decode edilir

---

