use crate::models::*;
use crate::loader::GalleryLoader;

pub struct QueryEngine {
    pub loader: GalleryLoader,
}

impl QueryEngine {
    pub fn new(gallery_base: &str) -> Self {
        Self {
            loader: GalleryLoader::new(gallery_base),
        }
    }

    /// Get all data for a week
    pub fn week(&self, week: &str) -> Result<WeekData, String> {
        self.loader.load_week(week)
    }

    /// Get a specific situation across weeks
    pub fn situation(&self, name: &str) -> Result<Vec<(String, Situation)>, String> {
        let weeks = self.loader.list_weeks()?;
        let mut results = Vec::new();
        for w in &weeks {
            if let Ok(sits) = self.loader.load_situations(w) {
                for s in sits {
                    if s.name == name {
                        results.push((w.clone(), s));
                    }
                }
            }
        }
        Ok(results)
    }

    /// Get intentions for a specific situation in a specific week
    pub fn intentions(&self, week: &str, name: &str) -> Result<Vec<Intention>, String> {
        let data = self.loader.load_week(week)?;
        Ok(data.intention_map.get(name).cloned().unwrap_or_default())
    }

    /// List all weeks
    pub fn list_weeks(&self) -> Result<Vec<String>, String> {
        self.loader.list_weeks()
    }

    /// Get registry
    pub fn registry(&self) -> Result<Vec<RegistryEntry>, String> {
        self.loader.load_registry()
    }
}
