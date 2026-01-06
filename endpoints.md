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

**Success (200 OK)**
```json
{
  "status": "ok",
  "message": "API çalışıyor"
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
      "id": "68bbf6df8ef4e8023c19641d",
      "kurumAdi": "Sosyal Güvenlik Kurumu",
      "kurumLogo": "https://cdn.mevzuatgpt.org/portal/kurum_Sosyal%20G%C3%BCvenlik%20Kurumu_b98c7ed9-4bfc-42af-8cbf-550bd9e1885f.svg",
      "aciklama": "sgk.gov.tr",
      "olusturulmaTarihi": "2025-09-06T08:54:55.403880",
      "detsis": "22620739"
    }
  ],
  "message": null
}
```

**Error (500 Internal Server Error)**
```json
{
  "success": false,
  "data": null,
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

