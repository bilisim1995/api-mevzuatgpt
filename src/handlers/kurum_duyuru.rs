use actix_web::{web, HttpResponse, http::StatusCode};
use mongodb::{Collection, bson::{doc, Document as MongoDocument}};
use crate::config::AppState;
use crate::models::kurum_duyuru_scraped::{DuyuruItem, KurumDuyuruScrapedResponse};
use regex::{Regex, RegexBuilder};
use std::collections::HashSet;
use url::Url;

#[derive(serde::Deserialize)]
pub struct KurumDuyuruQuery {
    pub kurum_id: String,
}

// HTML'den text temizleme (Go kodundaki cleanHTML fonksiyonuna benzer)
fn clean_html_text(text: &str) -> String {
    // HTML tag'lerini kaldır
    let html_tag_re = Regex::new(r"<[^>]*>").unwrap();
    let mut cleaned = html_tag_re.replace_all(text, "").to_string();
    
    // HTML entity'lerini decode et (html::unescape benzeri)
    cleaned = cleaned
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'");
    
    // Türkçe karakterler için HTML entity decode (&#231; -> ç, &#x131; -> ı gibi)
    // Önce hex formatı (&#x131;)
    let html_entity_hex_re = Regex::new(r"&#x([0-9a-fA-F]+);").unwrap();
    cleaned = html_entity_hex_re.replace_all(&cleaned, |caps: &regex::Captures| {
        if let Ok(num) = u32::from_str_radix(&caps[1], 16) {
            if let Some(ch) = std::char::from_u32(num) {
                return ch.to_string();
            }
        }
        caps[0].to_string()
    }).to_string();
    
    // Sonra decimal formatı (&#231;)
    let html_entity_re = Regex::new(r"&#(\d+);").unwrap();
    cleaned = html_entity_re.replace_all(&cleaned, |caps: &regex::Captures| {
        if let Ok(num) = caps[1].parse::<u32>() {
            if let Some(ch) = std::char::from_u32(num) {
                return ch.to_string();
            }
        }
        caps[0].to_string()
    }).to_string();
    
    // Fazla boşlukları temizle
    let whitespace_re = Regex::new(r"\s+").unwrap();
    cleaned = whitespace_re.replace_all(&cleaned, " ").to_string();
    
    cleaned.trim().to_string()
}

// Navigasyon linki kontrolü (Go kodundaki isNavigationLink fonksiyonuna benzer)
fn is_navigation_link(text: &str) -> bool {
    let nav_keywords = [
        "ana sayfa", "anasayfa", "home", "menü", "menu",
        "hakkımızda", "iletişim", "contact", "about",
        "giriş", "login", "kayıt", "register", "çıkış", "logout",
        "ara", "search", "site haritası", "sitemap",
    ];
    
    let lower_text = text.to_lowercase();
    for keyword in nav_keywords.iter() {
        if lower_text.contains(keyword) {
            return true;
        }
    }
    
    // Çok kısa metinler muhtemelen navigasyon linkidir
    text.trim().len() < 15
}

// Mutlak URL oluştur (Go kodundaki makeAbsoluteURL fonksiyonuna benzer)
fn make_absolute_url(href: &str, base_url: &Url) -> String {
    if href.starts_with("http") {
        return href.to_string();
    }
    
    if href.starts_with("/") {
        if let Ok(joined) = base_url.join(href) {
            return joined.to_string();
        }
    }
    
    if let Ok(joined) = base_url.join(&format!("/{}", href.trim_start_matches('/'))) {
        return joined.to_string();
    }
    
    href.to_string()
}

// Tarih çıkarma (Go kodundaki extractDateFromText fonksiyonuna benzer)
fn extract_date_from_text(text: &str) -> String {
    // Yaygın Türkçe tarih pattern'leri
    let date_patterns = [
        r"\d{1,2}[./]\d{1,2}[./]\d{4}",  // 01.01.2024 veya 01/01/2024
        r"\d{1,2}[./]\d{1,2}[./]\d{2}",  // 01.01.24
        r"\d{4}[.-]\d{1,2}[.-]\d{1,2}",  // 2024-01-01
    ];
    
    for pattern in date_patterns.iter() {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.find(text) {
                return normalize_date(caps.as_str());
            }
        }
    }
    
    String::new()
}

// Başlık etrafındaki metinden tarih çıkar (Go kodundaki extractDateFromHTML fonksiyonuna benzer)
fn extract_date_from_html(html_content: &str, title: &str, search_range: usize) -> String {
    if let Some(title_index) = html_content.find(title) {
        let start = title_index.saturating_sub(search_range);
        let end = (title_index + title.len() + search_range).min(html_content.len());
        let surrounding_text = &html_content[start..end];
        
        let date = extract_date_from_text(surrounding_text);
        if !date.is_empty() {
            return date;
        }
    }
    
    chrono::Utc::now().format("%d.%m.%Y").to_string()
}

// Tarih formatını normalize et
fn normalize_date(date_str: &str) -> String {
    // DD.MM.YYYY formatına çevir
    let date_re = Regex::new(r"(\d{1,2})[./](\d{1,2})[./](\d{2,4})").unwrap();
    if let Some(caps) = date_re.captures(date_str) {
        let day = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let month = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let year = caps.get(3).map(|m| m.as_str()).unwrap_or("");
        
        // Yıl 2 haneli ise 20 ekle
        let full_year = if year.len() == 2 {
            format!("20{}", year)
        } else {
            year.to_string()
        };
        
        return format!("{}.{}.{}", day, month, full_year);
    }
    
    // Ay isimlerini kontrol et
    let month_map = [
        ("Ocak", "01"), ("Şubat", "02"), ("Mart", "03"), ("Nisan", "04"),
        ("Mayıs", "05"), ("Haziran", "06"), ("Temmuz", "07"), ("Ağustos", "08"),
        ("Eylül", "09"), ("Ekim", "10"), ("Kasım", "11"), ("Aralık", "12"),
        ("Ağu", "08"), ("Eyl", "09"), ("Eki", "10"), ("Kas", "11"), ("Ara", "12"),
    ];
    
    for (month_name, month_num) in month_map.iter() {
        if date_str.contains(month_name) {
            let date_re = Regex::new(r"(\d{1,2})\s*").unwrap();
            if let Some(caps) = date_re.captures(date_str) {
                let day = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let year_re = Regex::new(r"(\d{4})").unwrap();
                let year = year_re.captures(date_str)
                    .and_then(|c| c.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| chrono::Utc::now().format("%Y").to_string());
                return format!("{}.{}.{}", day, month_num, year);
            }
        }
    }
    
    // Bulunamazsa bugünün tarihi
    chrono::Utc::now().format("%d.%m.%Y").to_string()
}

// Yargıtay scraper (Go kodundaki extractDuyurularWithRegex fonksiyonuna benzer)
async fn scrape_yargitay(url: &str) -> Result<Vec<DuyuruItem>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client oluşturulamadı: {}", e))?;
    
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Sayfa çekilemedi: {}", e))?;
    
    let html = response.text().await
        .map_err(|e| format!("HTML okunamadı: {}", e))?;
    
    let base_url = Url::parse(url).map_err(|_| "Geçersiz URL".to_string())?;
    
    // Go kodundaki üç aşamalı regex yaklaşımını kullan
    let duyurular = extract_yargitay_duyurular_with_regex(&html, &base_url);
    
    // Limit to 5 results
    Ok(duyurular.into_iter().take(5).collect())
}

// Yargıtay duyurularını regex ile çıkar (Go kodundaki extractDuyurularWithRegex fonksiyonuna benzer)
fn extract_yargitay_duyurular_with_regex(html_content: &str, base_url: &Url) -> Vec<DuyuruItem> {
    let mut duyurular = Vec::new();
    let mut seen_links = HashSet::new();
    
    // Primary regex: Yargıtay item links + keyword-based links with nested HTML support
    // Case insensitive + dot matches newline
    let link_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*(?:/item/\d+/[^"']*|(?:duyuru|haber|news|announcement)[^"']*))["'][^>]*>([\s\S]*?)</a>"#)
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();
    
    for caps in link_pattern.captures_iter(html_content) {
        if caps.len() >= 3 {
            let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let inner_html = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            
            if href.is_empty() {
                continue;
            }
            
            let title = clean_html_text(inner_html);
            let normalized_link = make_absolute_url(href, base_url);
            
            if title.len() > 10 && !seen_links.contains(&normalized_link) && !is_navigation_link(&title) {
                seen_links.insert(normalized_link.clone());
                let tarih = extract_date_from_html(html_content, &title, 500);
                duyurular.push(DuyuruItem {
                    baslik: title,
                    link: normalized_link,
                    tarih,
                });
            }
        }
    }
    
    // Secondary pass: if we have fewer than 5 items, try general item links
    if duyurular.len() < 5 {
        let item_link_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*/item/\d+/[^"']*)["'][^>]*>([\s\S]*?)</a>"#)
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap();
        
        for caps in item_link_pattern.captures_iter(html_content) {
            if duyurular.len() >= 5 {
                break;
            }
            
            if caps.len() >= 3 {
                let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let inner_html = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                
                if href.is_empty() {
                    continue;
                }
                
                let title = clean_html_text(inner_html);
                let normalized_link = make_absolute_url(href, base_url);
                
                if title.len() > 10 && !seen_links.contains(&normalized_link) && !is_navigation_link(&title) {
                    seen_links.insert(normalized_link.clone());
                    let tarih = extract_date_from_html(html_content, &title, 500);
                    duyurular.push(DuyuruItem {
                        baslik: title,
                        link: normalized_link,
                        tarih,
                    });
                }
            }
        }
    }
    
    // Final fallback: general links if still not enough
    if duyurular.len() < 5 {
        let general_link_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*)["'][^>]*>([\s\S]{15,}?)</a>"#)
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap();
        
        for caps in general_link_pattern.captures_iter(html_content) {
            if duyurular.len() >= 5 {
                break;
            }
            
            if caps.len() >= 3 {
                let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let inner_html = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                
                if href.is_empty() || href.contains("#") || href.contains("javascript:") || href == "/" {
                    continue;
                }
                
                let title = clean_html_text(inner_html);
                let normalized_link = make_absolute_url(href, base_url);
                
                if title.len() > 15 && !seen_links.contains(&normalized_link) && !is_navigation_link(&title) {
                    seen_links.insert(normalized_link.clone());
                    let tarih = extract_date_from_html(html_content, &title, 500);
                    duyurular.push(DuyuruItem {
                        baslik: title,
                        link: normalized_link,
                        tarih,
                    });
                }
            }
        }
    }
    
    duyurular
}

// SGK scraper (Go kodundaki extractSGKDuyurularWithRegex fonksiyonuna benzer)
async fn scrape_sgk(url: &str) -> Result<Vec<DuyuruItem>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client oluşturulamadı: {}", e))?;
    
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Sayfa çekilemedi: {}", e))?;
    
    let html = response.text().await
        .map_err(|e| format!("HTML okunamadı: {}", e))?;
    
    let base_url = Url::parse(url).map_err(|_| "Geçersiz URL".to_string())?;
    
    let duyurular = extract_sgk_duyurular_with_regex(&html, &base_url);
    
    Ok(duyurular.into_iter().take(5).collect())
}

// SGK duyurularını regex ile çıkar (Go kodundaki extractSGKDuyurularWithRegex fonksiyonuna benzer)
fn extract_sgk_duyurular_with_regex(html_content: &str, base_url: &Url) -> Vec<DuyuruItem> {
    let mut duyurular = Vec::new();
    let mut seen_links = HashSet::new();
    
    // Primary regex: SGK /Duyuru/Detay/ links
    let link_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*/Duyuru/Detay/[^"']*)["'][^>]*>([\s\S]*?)</a>"#)
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();
    
    for caps in link_pattern.captures_iter(html_content) {
        if caps.len() >= 3 {
            let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let inner_html = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            
            if href.is_empty() {
                continue;
            }
            
            let title = clean_html_text(inner_html);
            let normalized_link = make_absolute_url(href, base_url);
            
            if title.len() > 10 && !seen_links.contains(&normalized_link) {
                seen_links.insert(normalized_link.clone());
                let tarih = extract_sgk_date_from_html(html_content, &title);
                duyurular.push(DuyuruItem {
                    baslik: title,
                    link: normalized_link,
                    tarih,
                });
            }
        }
    }
    
    // Secondary pass: if we have fewer than 5 items, try general duyuru links
    if duyurular.len() < 5 {
        let general_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*(?:duyuru|Duyuru)[^"']*)["'][^>]*>([\s\S]*?)</a>"#)
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap();
        
        for caps in general_pattern.captures_iter(html_content) {
            if duyurular.len() >= 5 {
                break;
            }
            
            if caps.len() >= 3 {
                let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let inner_html = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                
                if href.is_empty() {
                    continue;
                }
                
                let title = clean_html_text(inner_html);
                let normalized_link = make_absolute_url(href, base_url);
                
                if title.len() > 15 && !seen_links.contains(&normalized_link) && !is_navigation_link(&title) {
                    seen_links.insert(normalized_link.clone());
                    let tarih = extract_sgk_date_from_html(html_content, &title);
                    duyurular.push(DuyuruItem {
                        baslik: title,
                        link: normalized_link,
                        tarih,
                    });
                }
            }
        }
    }
    
    duyurular
}

// SGK tarih çıkarma (Go kodundaki extractSGKDateFromHTML fonksiyonuna benzer)
fn extract_sgk_date_from_html(html_content: &str, title: &str) -> String {
    // Başlık etrafındaki 1000 karakter içinde arama
    if let Some(title_index) = html_content.find(title) {
        let start = title_index.saturating_sub(1000);
        let end = (title_index + title.len() + 1000).min(html_content.len());
        let surrounding_text = &html_content[start..end];
        
        let date = extract_sgk_date_from_text(surrounding_text);
        if !date.is_empty() {
            return date;
        }
    }
    
    chrono::Utc::now().format("%d.%m.%Y").to_string()
}

// SGK tarih çıkarma (Go kodundaki extractSGKDateFromText fonksiyonuna benzer)
fn extract_sgk_date_from_text(text: &str) -> String {
    // SGK Türkçe ay isimleri kullanır: "23 Eylül 2025"
    let turkish_months = [
        ("Ocak", "01"), ("Şubat", "02"), ("Mart", "03"), ("Nisan", "04"),
        ("Mayıs", "05"), ("Haziran", "06"), ("Temmuz", "07"), ("Ağustos", "08"),
        ("Eylül", "09"), ("Ekim", "10"), ("Kasım", "11"), ("Aralık", "12"),
    ];
    
    // Pattern: "23 Eylül 2025"
    let pattern = Regex::new(r"(\d{1,2})\s+(Ocak|Şubat|Mart|Nisan|Mayıs|Haziran|Temmuz|Ağustos|Eylül|Ekim|Kasım|Aralık)\s+(\d{4})").unwrap();
    
    if let Some(caps) = pattern.captures(text) {
        if caps.len() >= 4 {
            let day = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let month_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let year = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            
            for (name, num) in turkish_months.iter() {
                if month_name == *name {
                    let day_padded = if day.len() == 1 {
                        format!("0{}", day)
                    } else {
                        day.to_string()
                    };
                    return format!("{}.{}.{}", day_padded, num, year);
                }
            }
        }
    }
    
    // Fallback to standard date patterns
    extract_date_from_text(text)
}

// İşkur scraper (Go kodundaki extractIskurDuyurularWithRegex fonksiyonuna benzer)
async fn scrape_iskur(url: &str) -> Result<Vec<DuyuruItem>, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client oluşturulamadı: {}", e))?;
    
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Sayfa çekilemedi: {}", e))?;
    
    let html = response.text().await
        .map_err(|e| format!("HTML okunamadı: {}", e))?;
    
    let base_url = Url::parse(url).map_err(|_| "Geçersiz URL".to_string())?;
    
    let duyurular = extract_iskur_duyurular_with_regex(&html, &base_url);
    
    Ok(duyurular.into_iter().take(5).collect())
}

// İşkur duyurularını regex ile çıkar (Go kodundaki extractIskurDuyurularWithRegex fonksiyonuna benzer)
fn extract_iskur_duyurular_with_regex(html_content: &str, base_url: &Url) -> Vec<DuyuruItem> {
    let mut duyurular = Vec::new();
    let mut seen_links = HashSet::new();
    
    // Primary regex: İşkur /duyurular/ links with title attribute
    let link_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*/duyurular/[^"']+)["'][^>]*title=["']([^"']+)["']"#)
        .case_insensitive(true)
        .dot_matches_new_line(true)
        .build()
        .unwrap();
    
    for caps in link_pattern.captures_iter(html_content) {
        if caps.len() >= 3 {
            let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let title = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            
            if href.is_empty() {
                continue;
            }
            
            let cleaned_title = clean_html_text(title);
            let normalized_link = make_absolute_url(href, base_url);
            
            if cleaned_title.len() > 10 && !seen_links.contains(&normalized_link) {
                seen_links.insert(normalized_link.clone());
                let tarih = extract_iskur_date_from_html(html_content, &cleaned_title);
                duyurular.push(DuyuruItem {
                    baslik: cleaned_title,
                    link: normalized_link,
                    tarih,
                });
            }
        }
    }
    
    // Secondary pass: if we have fewer than 5 items, try general duyuru links without title attribute
    if duyurular.len() < 5 {
        let general_pattern = RegexBuilder::new(r#"<a[^>]+href=["']([^"']*/duyurular/[^"']*)["'][^>]*>([\s\S]*?)</a>"#)
            .case_insensitive(true)
            .dot_matches_new_line(true)
            .build()
            .unwrap();
        
        for caps in general_pattern.captures_iter(html_content) {
            if duyurular.len() >= 5 {
                break;
            }
            
            if caps.len() >= 3 {
                let href = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let inner_html = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                
                if href.is_empty() {
                    continue;
                }
                
                let title = clean_html_text(inner_html);
                let normalized_link = make_absolute_url(href, base_url);
                
                if title.len() > 15 && !seen_links.contains(&normalized_link) && !is_navigation_link(&title) {
                    seen_links.insert(normalized_link.clone());
                    let tarih = extract_iskur_date_from_html(html_content, &title);
                    duyurular.push(DuyuruItem {
                        baslik: title,
                        link: normalized_link,
                        tarih,
                    });
                }
            }
        }
    }
    
    duyurular
}

// İşkur tarih çıkarma (Go kodundaki extractIskurDateFromHTML fonksiyonuna benzer)
fn extract_iskur_date_from_html(html_content: &str, title: &str) -> String {
    // Başlık etrafındaki 1500 karakter içinde arama
    if let Some(title_index) = html_content.find(title) {
        let start = title_index.saturating_sub(1500);
        let end = (title_index + title.len() + 1500).min(html_content.len());
        let surrounding_text = &html_content[start..end];
        
        let date = extract_iskur_date_from_text(surrounding_text);
        if !date.is_empty() {
            return date;
        }
    }
    
    chrono::Utc::now().format("%d.%m.%Y").to_string()
}

// İşkur tarih çıkarma (Go kodundaki extractIskurDateFromText fonksiyonuna benzer)
fn extract_iskur_date_from_text(text: &str) -> String {
    // İşkur kısaltılmış Türkçe ay isimleri kullanır: "11 Ağu 2025", "27 Haz 2025"
    let turkish_months = [
        ("Oca", "01"), ("Şub", "02"), ("Mar", "03"), ("Nis", "04"), ("May", "05"), ("Haz", "06"),
        ("Tem", "07"), ("Ağu", "08"), ("Eyl", "09"), ("Eki", "10"), ("Kas", "11"), ("Ara", "12"),
    ];
    
    // Pattern: "11 Ağu 2025"
    let pattern = Regex::new(r"(\d{1,2})\s+(Oca|Şub|Mar|Nis|May|Haz|Tem|Ağu|Eyl|Eki|Kas|Ara)\s+(\d{4})").unwrap();
    
    if let Some(caps) = pattern.captures(text) {
        if caps.len() >= 4 {
            let day = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let month_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let year = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            
            for (name, num) in turkish_months.iter() {
                if month_name == *name {
                    let day_padded = if day.len() == 1 {
                        format!("0{}", day)
                    } else {
                        day.to_string()
                    };
                    return format!("{}.{}.{}", day_padded, num, year);
                }
            }
        }
    }
    
    // Fallback to standard date patterns
    extract_date_from_text(text)
}

// Domain'e göre scraper seç
pub async fn scrape_by_domain(url: &str) -> Result<Vec<DuyuruItem>, String> {
    if url.contains("yargitay.gov.tr") {
        scrape_yargitay(url).await
    } else if url.contains("sgk.gov.tr") {
        scrape_sgk(url).await
    } else if url.contains("iskur.gov.tr") {
        scrape_iskur(url).await
    } else {
        // Varsayılan olarak Yargıtay scraper'ı kullan
        scrape_yargitay(url).await
    }
}

pub async fn get_kurum_duyuru(
    state: web::Data<AppState>,
    query: web::Query<KurumDuyuruQuery>,
) -> HttpResponse {
    // kurum_id zorunlu kontrolü
    if query.kurum_id.is_empty() {
        return HttpResponse::build(StatusCode::BAD_REQUEST).json(KurumDuyuruScrapedResponse {
            success: false,
            data: vec![],
            count: 0,
            message: None,
            error: Some("kurum_id parameter is required".to_string()),
        });
    }
    
    // kurum_duyuru koleksiyonundan duyuru_linki'ni al
    let collection: Collection<MongoDocument> = state.db.collection("kurum_duyuru");
    
    let filter = doc! {
        "kurum_id": &query.kurum_id
    };
    
    let duyuru_doc = match collection.find_one(filter, None).await {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            return HttpResponse::build(StatusCode::NOT_FOUND).json(KurumDuyuruScrapedResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Kurum duyuru linki bulunamadı".to_string()),
            });
        }
        Err(e) => {
            log::error!("MongoDB sorgu hatası: {}", e);
            return HttpResponse::InternalServerError().json(KurumDuyuruScrapedResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Kurum duyuru linki bulunamadı".to_string()),
            });
        }
    };
    
    // duyuru_linki'ni al
    let duyuru_linki = match duyuru_doc.get_str("duyuru_linki") {
        Ok(link) if !link.is_empty() => link.to_string(),
        _ => {
            return HttpResponse::build(StatusCode::NOT_FOUND).json(KurumDuyuruScrapedResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Kurum için duyuru linki tanımlanmamış".to_string()),
            });
        }
    };
    
    // Web scraping yap
    let duyurular = match scrape_by_domain(&duyuru_linki).await {
        Ok(items) => items,
        Err(e) => {
            log::error!("Web scraping hatası: {}", e);
            return HttpResponse::InternalServerError().json(KurumDuyuruScrapedResponse {
                success: false,
                data: vec![],
                count: 0,
                message: None,
                error: Some("Duyuru sayfası çekilemedi".to_string()),
            });
        }
    };
    
    let count = duyurular.len() as u64;
    
    HttpResponse::Ok().json(KurumDuyuruScrapedResponse {
        success: true,
        data: duyurular,
        count,
        message: Some("Kurum duyuruları başarıyla çekildi".to_string()),
        error: None,
    })
}

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.route("", web::get().to(get_kurum_duyuru));
}

