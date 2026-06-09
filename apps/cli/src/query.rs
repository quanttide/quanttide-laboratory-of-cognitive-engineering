use std::collections::HashMap;

use quanttide_think::{
    intention::Intention,
    situation::Situation,
    domain::Domain,
};
use crate::loader::GalleryLoader;

pub type WeekData = (Vec<Situation>, Vec<Intention>, HashMap<String, Vec<Intention>>);

pub struct QueryEngine {
    pub loader: GalleryLoader,
}

impl QueryEngine {
    pub fn new(gallery_base: &str) -> Self {
        Self {
            loader: GalleryLoader::new(gallery_base),
        }
    }

    pub fn week(&self, week: &str) -> Result<WeekData, String> {
        self.loader.load_week(week)
    }

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

    pub fn intentions(&self, week: &str, name: &str) -> Result<Vec<Intention>, String> {
        let data = self.loader.load_week(week)?;
        Ok(data.2.get(name).cloned().unwrap_or_default())
    }

    pub fn all_intentions(
        &self,
        week: Option<&str>,
        sit_name: Option<&str>,
        priority: Option<&str>,
        risk: Option<&str>,
        level: Option<&str>,
        agent: Option<&str>,
    ) -> Result<Vec<(String, String, Intention)>, String> {
        let weeks = match week {
            Some(w) => vec![w.to_string()],
            None => self.loader.list_weeks()?,
        };
        let mut results = Vec::new();
        for w in &weeks {
            if let Ok(d) = self.loader.load_week(w) {
                for (sn, intents) in &d.2 {
                    if let Some(ref filter_name) = sit_name {
                        if sn != filter_name {
                            continue;
                        }
                    }
                    for i in intents {
                        if let Some(ref p) = priority {
                            if i.priority.name != *p { continue; }
                        }
                        if let Some(ref r) = risk {
                            if i.risk.name != *r { continue; }
                        }
                        if let Some(ref l) = level {
                            if i.level.name != *l { continue; }
                        }
                        if let Some(ref a) = agent {
                            if i.agent.name != *a { continue; }
                        }
                        results.push((w.clone(), sn.clone(), i.clone()));
                    }
                }
            }
        }
        results.sort_by(|a, b| a.2.priority.name.cmp(&b.2.priority.name));
        Ok(results)
    }

    pub fn intention_by_id(&self, id: &str) -> Result<Option<(String, String, Intention)>, String> {
        let weeks = self.loader.list_weeks()?;
        for w in &weeks {
            if let Ok(d) = self.loader.load_week(w) {
                for (sn, intents) in &d.2 {
                    for i in intents {
                        if i.id.to_string() == id {
                            return Ok(Some((w.clone(), sn.clone(), i.clone())));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn list_weeks(&self) -> Result<Vec<String>, String> {
        self.loader.list_weeks()
    }

    pub fn registry(&self) -> Result<Vec<Domain>, String> {
        self.loader.load_registry()
    }
}
