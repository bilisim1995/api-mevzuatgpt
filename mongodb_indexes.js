// MongoDB Index Önerileri
// Bu dosyayı MongoDB shell'de veya MongoDB Compass'ta çalıştırarak index'leri oluşturabilirsiniz

// ============================================
// metadata koleksiyonu için index'ler
// ============================================

// url_slug için unique index (document detail endpoint için kritik)
db.metadata.createIndex(
    { "url_slug": 1 },
    { unique: true, name: "idx_url_slug_unique" }
);

// kurum_id için index (filtreleme ve join işlemleri için)
db.metadata.createIndex(
    { "kurum_id": 1 },
    { name: "idx_kurum_id" }
);

// olusturulma_tarihi için index (sıralama için)
db.metadata.createIndex(
    { "olusturulma_tarihi": -1 },
    { name: "idx_olusturulma_tarihi_desc" }
);

// belge_turu için index (filtreleme için)
db.metadata.createIndex(
    { "belge_turu": 1 },
    { name: "idx_belge_turu" }
);

// Text search için compound index (pdf_adi ve aciklama için)
db.metadata.createIndex(
    { "pdf_adi": "text", "aciklama": "text" },
    { name: "idx_text_search" }
);

// Compound index: kurum_id + olusturulma_tarihi (sık kullanılan kombinasyon)
db.metadata.createIndex(
    { "kurum_id": 1, "olusturulma_tarihi": -1 },
    { name: "idx_kurum_tarih" }
);

// ============================================
// content koleksiyonu için index'ler
// ============================================

// metadata_id için index (document detail endpoint için kritik)
db.content.createIndex(
    { "metadata_id": 1 },
    { name: "idx_content_metadata_id" }
);

// ============================================
// kurumlar koleksiyonu için index'ler
// ============================================

// _id zaten otomatik index'lenmiş, ancak emin olmak için:
// (MongoDB'de _id otomatik olarak index'lenir, bu yüzden gerekli değil)

// kurum_adi için index (arama için, opsiyonel)
db.kurumlar.createIndex(
    { "kurum_adi": 1 },
    { name: "idx_kurum_adi" }
);

// ============================================
// kurum_duyuru koleksiyonu için index'ler
// ============================================

// kurum_id için index
db.kurum_duyuru.createIndex(
    { "kurum_id": 1 },
    { name: "idx_duyuru_kurum_id" }
);

// ============================================
// links koleksiyonu için index'ler
// ============================================

// kurum_id için index
db.links.createIndex(
    { "kurum_id": 1 },
    { name: "idx_links_kurum_id" }
);

// ============================================
// Index Kullanımını Kontrol Etme
// ============================================

// Tüm index'leri görmek için:
// db.metadata.getIndexes()
// db.content.getIndexes()
// db.kurumlar.getIndexes()

// Index kullanımını analiz etmek için:
// db.metadata.find({ "url_slug": "test" }).explain("executionStats")

// ============================================
// Performans Notları
// ============================================

// 1. url_slug unique index'i document detail endpoint'i için kritik
// 2. kurum_id index'leri join işlemlerini hızlandırır
// 3. olusturulma_tarihi index'i sıralama işlemlerini hızlandırır
// 4. Text search index'i arama performansını artırır
// 5. Compound index'ler sık kullanılan sorgu kombinasyonlarını optimize eder

