use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Kurum {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub kurum_adi: String,
    pub kurum_logo: String,
    pub aciklama: String,
    pub olusturulma_tarihi: String,
    pub detsis: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KurumResponse {
    pub id: String,
    pub kurum_adi: String,
    pub kurum_logo: String,
    pub aciklama: String,
    pub olusturulma_tarihi: String,
    pub detsis: String,
}

impl From<Kurum> for KurumResponse {
    fn from(kurum: Kurum) -> Self {
        Self {
            id: kurum.id.to_hex(),
            kurum_adi: kurum.kurum_adi,
            kurum_logo: kurum.kurum_logo,
            aciklama: kurum.aciklama,
            olusturulma_tarihi: kurum.olusturulma_tarihi,
            detsis: kurum.detsis,
        }
    }
}

