use crate::rustustc::young::{SecondClass, SCFilter};
use crate::rustustc::young::service::YouthService;
use anyhow::Result;
use std::collections::HashMap;

pub struct Recommender;

impl Recommender {
    /// Core recommendation function
    pub async fn recommend(
        service: &YouthService,
        limit: usize,
    ) -> Result<Vec<SecondClass>> {
        // 1. Get user history
        let history = SecondClass::get_participated(service).await?;
        
        // 2. Generate tag weights from history 
        let mut tag_weights: HashMap<String, i32> = HashMap::new();
        for item in &history {
            // Statistics module weight
            if let Some(m) = item.module() {
                *tag_weights.entry(m.value).or_insert(0) += 2; // 模块权重更高
            }
            // Statistics label weights
            for label in item.labels() {
                *tag_weights.entry(label.id).or_insert(0) += 1;
            }
        }

        // 3. Get candidate activities
        let candidates = SecondClass::find(
            service, 
            SCFilter::new(), 
            false, 
            true,
            100
        ).await?;

        // 4. Score candidates
        let mut scored_candidates: Vec<(f64, SecondClass)> = candidates.into_iter().map(|item| {
            let mut score = 0.0;
            
            let popularity = item.apply_num.unwrap_or(0) as f64;
            score += popularity / item.apply_limit.unwrap_or(1) as f64;

            if let Some(m) = item.module() {
                if let Some(weight) = tag_weights.get(&m.value) {
                    score += *weight as f64 * 0.3;
                }
            }
            for label in item.labels() {
                if let Some(weight) = tag_weights.get(&label.id) {
                    score += *weight as f64 * 1.0;
                }
            }
            
            (score, item)
        }).collect();

        // 5. sort
        scored_candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // 6. return
        Ok(scored_candidates.into_iter().take(limit).map(|x| x.1).collect())
    }
}