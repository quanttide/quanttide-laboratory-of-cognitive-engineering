use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use quanttide_think::{
    intention::Intention,
    situation::Situation,
    schema::Schema,
    domain::Domain,
};
use crate::query::WeekData;

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
    pub fn load_registry(&self) -> Result<Vec<Domain>, String> {
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

    /// Load a full week
    pub fn load_week(&self, week: &str) -> Result<WeekData, String> {
        let situations = self.load_situations(week)?;
        let intentions = self.load_intentions(week)?;
        let intention_map = Self::build_intention_map(&self.gallery_base, week);

        Ok((situations, intentions, intention_map))
    }

    /// Load domain registry (ordered list of situations from contract/domain.yaml)
    pub fn load_category(&self) -> Result<Vec<Domain>, String> {
        let path = self.gallery_base.join("contract").join("domain.yaml");
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read category: {}", e))?;
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse category: {}", e))
    }

    /// Load schemas for a given week
    pub fn load_schemas(&self, week: &str) -> Result<Vec<Schema>, String> {
        let path = self.gallery_base.join("schema").join(format!("{}.yaml", week));
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Ok(Vec::new()),
        };
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse schema {}: {}", week, e))
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

    fn build_intention_map(gallery_base: &std::path::PathBuf, week: &str) -> HashMap<String, Vec<Intention>> {
        let mut map: HashMap<String, Vec<Intention>> = HashMap::new();
        let dir = gallery_base.join("intention").join(week);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "yaml") {
                    let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(list) = serde_yaml::from_str::<Vec<Intention>>(&content) {
                            map.entry(stem).or_default().extend(list);
                        }
                    }
                }
            }
        }
        map
    }
}
