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
GET /api/v1/documents?kurum_id={id}&limit=10000&sort_by=olusturulma_tarihi&sort_order=desc
```

**Query Parameters:**
- `kurum_id` (opsiyonel): Kurum ID'si ile filtreleme
- `limit` (opsiyonel, varsayılan: 10000): Maksimum kayıt sayısı
- `sort_by` (opsiyonel, varsayılan: olusturulma_tarihi): Sıralama alanı
- `sort_order` (opsiyonel, varsayılan: desc): Sıralama yönü (asc/desc)

**Headers:** Yok

**Body:** Yok

### Response

**Success (200 OK)**
```json
{
  "success": true,
  "data": [
    {
      "id": "68bc2e010ba7fbdd9f7d41f7",
      "kurumId": "68bbf6df8ef4e8023c19641d",
      "kurumAdi": "Sosyal Güvenlik Kurumu",
      "kurumLogo": "https://cdn.mevzuatgpt.org/portal/kurum_Sosyal%20G%C3%BCvenlik%20Kurumu_b98c7ed9-4bfc-42af-8cbf-550bd9e1885f.svg",
      "kurumAciklama": "sgk.gov.tr",
      "pdfAdi": "2016-21 - Kısa Vadeli Sigorta Kolları Uygulamaları",
      "etiketler": "2016-21",
      "belgeYayinTarihi": "2016-01-01",
      "belgeDurumu": "Yürürlükte",
      "aciklama": "T C Sosyal Güvenli k Kurumu Emeklilik Hizmetleri Genel Müdürlüğü 2016 21 SAYILI GENELGE...",
      "urlSlug": "2016-21-kisa-vadeli-sigorta-kollari-uygulamalari",
      "belgeTuru": "Genelge",
      "anahtarKelimeler": "sayılı, kazası, göremezlik, meslek, sağlık...",
      "status": "aktif",
      "sayfaSayisi": 111,
      "dosyaBoyutuMb": 1.33,
      "pdfUrl": "https://cdn.mevzuatgpt.org/portal/2016-21%20-%20K%C4%B1sa%20Vadeli%20Sigorta%20Kollar%C4%B1%20Uygulamalar%C4%B1_68bc2e010ba7fbdd9f7d41f7.pdf"
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

