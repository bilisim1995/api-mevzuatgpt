# API MevzuatGPT - Rust Backend

Rust ile geliştirilmiş MongoDB tabanlı RESTful API backend.

## Kurulum

1. Rust yüklü olduğundan emin olun:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Bağımlılıkları yükleyin:
```bash
cargo build
```

3. `.env` dosyası oluşturun:
```env
PORT=8080
HOST=0.0.0.0
MONGODB_URI=mongodb://localhost:27017
MONGODB_DB_NAME=mevzuatgpt
RUST_LOG=info
```

## Çalıştırma

```bash
cargo run
```

Server `http://localhost:8080` adresinde çalışacaktır.

## Endpoint'ler

- `GET /api/health` - Sağlık kontrolü

## Proje Yapısı

```
src/
├── main.rs           # Ana server yapılandırması
├── config/           # Yapılandırma modülleri
│   └── mod.rs        # MongoDB bağlantısı ve config
├── handlers/         # Endpoint handler'ları
│   ├── mod.rs        # Handler modül tanımları
│   └── health.rs     # Health check handler
├── models/           # Veri modelleri
│   └── mod.rs        # Model tanımları
├── routes/           # Route yapılandırması
│   └── mod.rs        # Route tanımları
└── utils/            # Yardımcı fonksiyonlar
    └── mod.rs        # Utility fonksiyonları
```

## Yeni Endpoint Ekleme

1. `src/handlers/` altında yeni bir handler modülü oluşturun
2. Handler'ı `src/handlers/mod.rs` içinde export edin
3. Route'u `src/routes/mod.rs` içinde tanımlayın

