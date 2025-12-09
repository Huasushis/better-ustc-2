use crate::rustustc::young::service::YouthService;
use crate::rustustc::young::{SCFilter, SecondClass};
use ammonia::Builder;
use anyhow::Result;
use jieba_rs::Jieba;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

static JIEBA: Lazy<Jieba> = Lazy::new(|| Jieba::new());
static EMPTY_SET: Lazy<HashSet<&'static str>> = Lazy::new(|| HashSet::new());

pub struct Recommender;

impl Recommender {
    /// Core recommendation function
    pub async fn recommend(service: &YouthService, limit: usize) -> Result<Vec<SecondClass>> {
        // 1. Fetch Data
        let history = SecondClass::get_participated(service).await?;

        // Get candidates
        let candidates = SecondClass::find(service, SCFilter::new(), false, false, -1).await?;

        println!("candidates size: {}, history size: {}", candidates.len(), history.len());

        if history.is_empty() {
            return Ok(candidates.into_iter().take(limit).collect());
        }

        let mut user_vocab: HashMap<String, f64> = HashMap::new();

        for item in &history {
            let tokens = Self::extract_tokens(item);
            for token in tokens {
                *user_vocab.entry(token).or_insert(0.0) += 1.0;
            }
        }

        let user_norm: f64 = user_vocab.values().map(|v| v * v).sum::<f64>().sqrt();

        // let mut mxsc1 = 0.0;
        // let mut mxsc2 = 0.0;
        // let mut mxsc3 = 0.0;
        // let mut mxsc4 = 0.0;
        let mut scored_candidates: Vec<(f64, SecondClass)> = candidates
            .into_iter()
            .map(|item| {
                let mut score = 0.0;

                // Text Similarity (Cosine Similarity)
                let tokens = Self::extract_tokens(&item);
                let mut dot_product = 0.0;
                let mut item_norm_sq = 0.0;

                let mut item_tf: HashMap<String, f64> = HashMap::new();
                for token in tokens {
                    *item_tf.entry(token).or_insert(0.0) += 1.0;
                }

                for (token, tf) in item_tf {
                    item_norm_sq += tf * tf;
                    if let Some(user_weight) = user_vocab.get(&token) {
                        dot_product += tf * user_weight;
                    }
                }

                let item_norm = item_norm_sq.sqrt();

                if user_norm > 0.0 && item_norm > 0.0 {
                    let text_similarity = dot_product / (user_norm * item_norm);
                    score += text_similarity * 5.0;

                    // mxsc1 = (mxsc1 as f64).max(text_similarity * 5.0);
                }

                if let Some(dept) = item.department() {
                    let same_dept_count = history
                        .iter()
                        .filter(|h| h.department().map(|d| d.name == dept.name).unwrap_or(false))
                        .count();

                    if same_dept_count > 0 {
                        score += (same_dept_count as f64).min(8.0) * 0.5;
                        // mxsc2 = (mxsc2 as f64).max((same_dept_count as f64).min(8.0) * 0.5);
                    }
                }

                if let Some(module) = item.module() {
                    let same_module_count = history
                        .iter()
                        .filter(|h| h.module().map(|m| m.value == module.value).unwrap_or(false))
                        .count();

                    if same_module_count > 0 {
                        score += (same_module_count as f64).min(10.0) * 0.2;
                        // mxsc3 = (mxsc3 as f64).max((same_module_count as f64).min(10.0) * 0.2);
                    }
                }

                // Because of popularity have to use SecondClass::update to get data otherwize zero
                // so deprecated popularity based scoring
                // let popularity = item.apply_num.unwrap_or(0) as f64;
                // score += (popularity + 1.0).ln() * 0.1;
                // mxsc4 = (mxsc4 as f64).max((popularity + 1.0).ln() * 0.1);
                // println!("pop:{}", popularity);

                (score, item)
            })
            .collect();

        // println!("Max Scores: Text Sim: {:.4}, Dept Match: {:.4}, Module Match: {:.4}, Popularity: {:.4}",
        //     mxsc1, mxsc2, mxsc3, mxsc4);

        scored_candidates
            .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let history_ids: HashSet<String> = history.iter().map(|h| h.id.clone()).collect();
        let result = scored_candidates
            .into_iter()
            .filter(|(_, item)| !history_ids.contains(&item.id))
            .take(limit)
            .map(|(_, item)| item)
            .collect();

        Ok(result)
    }

    fn extract_tokens(item: &SecondClass) -> Vec<String> {
        let mut text = item.name.clone();

        if let Some(dept) = item.department() {
            text.push_str(" ");
            text.push_str(&dept.name);
        }

        if let Some(conceive) = &item.conceive {
            text.push_str(" ");
            text.push_str(&Self::strip_html(conceive));
        }

        if let Some(content) = &item.base_content {
            text.push_str(" ");
            text.push_str(&Self::strip_html(content));
        }

        let words = JIEBA.cut(&text, true);

        words
            .into_iter()
            .map(|s| s.to_string())
            .filter(|s| s.chars().count() > 1)
            .collect()
    }

    fn strip_html(s: &str) -> String {
        let mut builder = Builder::new();
        builder.tags(EMPTY_SET.clone());
        builder.link_rel(None);

        let cleaned = builder.clean(s).to_string();
        cleaned.replace('\u{A0}', " ")
    }
}
