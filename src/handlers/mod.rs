// Handler modülleri burada tanımlanacak
// Her endpoint için ayrı modül oluşturulacak

pub mod health;
pub mod kurum;
pub mod document;
pub mod announcement;
pub mod link;
pub mod regulation;
pub mod regulation_yargitay;
pub mod statistics;
pub mod kurum_duyuru;
pub mod search;
pub mod search_yargitay;
pub mod documents_yargitay;
pub mod sitemap;

// Yeni handler'lar eklendikçe buraya ekleyin

