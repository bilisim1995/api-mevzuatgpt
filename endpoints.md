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
GET /api/kurumlar
```

### Request
```
GET /api/kurumlar
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

