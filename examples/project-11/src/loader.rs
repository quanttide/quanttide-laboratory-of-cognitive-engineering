use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::models::*;

pub struct GalleryLoader {
    gallery_base: PathBuf,
}

impl GalleryLoader {
    pub fn new(gallery_base: &str) -> Self {
        Self {
            gallery_base: PathBuf::from(gallery_base),
        }
    }

    /// Load the registry
    pub fn load_registry(&self) -> Result<Vec<RegistryEntry>, String> {
        let path = self.gallery_base.join("situation").join("registry.yaml");
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read registry: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse registry: {}", e))
    }

    /// Load all situations for a given week
    pub fn load_situations(&self, week: &str) -> Result<Vec<Situation>, String> {
        let dir = self.gallery_base.join("situation").join(week);
        let mut situations = Vec::new();
        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("Failed to read situation dir {}: {}", week, e))?;
        for entry in entries {
            let path = entry.map_err(|e| format!("IO: {}", e))?.path();
            if path.extension().map_or(false, |e| e == "yaml") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
                let sit: Situation = serde_yaml::from_str(&content)
                    .map_err(|e| format!("Failed to parse {:?}: {}", path, e))?;
                situations.push(sit);
            }
        }
        situations.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(situations)
    }

    /// Load intentions for a given week
    pub fn load_intentions(&self, week: &str) -> Result<Vec<Intention>, String> {
        let dir = self.gallery_base.join("intention").join(week);
        let mut intentions = Vec::new();
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => return Ok(Vec::new()),
        };
        for entry in entries {
            let path = entry.map_err(|e| format!("IO: {}", e))?.path();
            if path.extension().map_or(false, |e| e == "yaml") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;
                let mut parsed: Vec<Intention> = serde_yaml::from_str(&content)
                    .map_err(|e| format!("Failed to parse {:?}: {}", path, e))?;
                intentions.append(&mut parsed);
            }
        }
        Ok(intentions)
    }

    /// Load a full week into WeekData
    pub fn load_week(&self, week: &str) -> Result<WeekData, String> {
        let situations = self.load_situations(week)?;
        let intentions = self.load_intentions(week)?;
        let mut intention_map: HashMap<String, Vec<Intention>> = HashMap::new();
        for intent in &intentions {
            // infer situation name from filename: first non-week component
            let name = self
                .gallery_base
                .join("intention")
                .join(week)
                .read_dir()
                .ok()
                .and_then(|entries| {
                    for e in entries.flatten() {
                        if e.path().extension().map_or(false, |ext| ext == "yaml") {
                            let stem = e.path().file_stem().unwrap().to_str().unwrap().to_string();
                            // check if this file contains this intention id
                            if let Ok(content) = fs::read_to_string(e.path()) {
                                if content.contains(&intent.id) {
                                    return Some(stem);
                                }
                            }
                        }
                    }
                    None
                })
                .unwrap_or_else(|| "unknown".to_string());
            intention_map.entry(name).or_default().push(intent.clone());
        }
        Ok(WeekData {
            week: week.to_string(),
            situations,
            intentions,
            intention_map,
        })
    }

    /// List available weeks by scanning the situation directory
    pub fn list_weeks(&self) -> Result<Vec<String>, String> {
        let dir = self.gallery_base.join("situation");
        let mut weeks = Vec::new();
        let entries = fs::read_dir(&dir)
            .map_err(|e| format!("Failed to read gallery: {}", e))?;
        for entry in entries {
            let path = entry.map_err(|e| format!("IO: {}", e))?.path();
            if path.is_dir() && path.file_name().unwrap().to_str().unwrap().starts_with("2026-") {
                weeks.push(path.file_name().unwrap().to_str().unwrap().to_string());
            }
        }
        weeks.sort();
        Ok(weeks)
    }
}
