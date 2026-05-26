use rusqlite::{params, Connection};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::models::*;

pub struct Database {
    conn: Connection,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                ai_pending INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS materials (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT,
                content_snippet TEXT,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS session_materials (
                session_id INTEGER NOT NULL,
                material_id INTEGER NOT NULL,
                PRIMARY KEY (session_id, material_id),
                FOREIGN KEY (session_id) REFERENCES sessions(id),
                FOREIGN KEY (material_id) REFERENCES materials(id)
            );

            CREATE TABLE IF NOT EXISTS thoughts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                material_id INTEGER,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                sort_order INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id),
                FOREIGN KEY (material_id) REFERENCES materials(id)
            );

            CREATE TABLE IF NOT EXISTS ideas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                sort_order INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE TABLE IF NOT EXISTS idea_thoughts (
                idea_id INTEGER NOT NULL,
                thought_id INTEGER NOT NULL,
                PRIMARY KEY (idea_id, thought_id),
                FOREIGN KEY (idea_id) REFERENCES ideas(id),
                FOREIGN KEY (thought_id) REFERENCES thoughts(id)
            );

            CREATE INDEX IF NOT EXISTS idx_thoughts_session ON thoughts(session_id);
            CREATE INDEX IF NOT EXISTS idx_ideas_session ON ideas(session_id);
            CREATE INDEX IF NOT EXISTS idx_thoughts_status ON thoughts(status);
            CREATE INDEX IF NOT EXISTS idx_ideas_status ON ideas(status);
            ",
        )?;
        Ok(())
    }

    // --- Session operations ---

    pub fn create_session(&self, title: Option<&str>) -> Result<Session> {
        let ts = now();
        self.conn.execute(
            "INSERT INTO sessions (title, created_at, updated_at, ai_pending) VALUES (?1, ?2, ?3, 0)",
            params![title, ts, ts],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(Session {
            id,
            title: title.map(|s| s.to_string()),
            created_at: ts,
            updated_at: ts,
            ai_pending: false,
        })
    }

    pub fn get_session(&self, id: i64) -> Result<Option<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, created_at, updated_at, ai_pending FROM sessions WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                ai_pending: row.get::<_, i64>(4)? != 0,
            })
        })?;
        match rows.next() {
            Some(Ok(session)) => Ok(Some(session)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, created_at, updated_at, ai_pending FROM sessions ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                ai_pending: row.get::<_, i64>(4)? != 0,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn set_session_ai_pending(&self, id: i64, pending: bool) -> Result<()> {
        let ts = now();
        self.conn.execute(
            "UPDATE sessions SET ai_pending = ?1, updated_at = ?2 WHERE id = ?3",
            params![pending as i64, ts, id],
        )?;
        Ok(())
    }

    // --- Material operations ---

    pub fn create_material(&self, path: Option<&str>, content_snippet: Option<&str>) -> Result<Material> {
        let ts = now();
        self.conn.execute(
            "INSERT INTO materials (path, content_snippet, created_at) VALUES (?1, ?2, ?3)",
            params![path, content_snippet, ts],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(Material {
            id,
            path: path.map(|s| s.to_string()),
            content_snippet: content_snippet.map(|s| s.to_string()),
            created_at: ts,
        })
    }

    pub fn get_material(&self, id: i64) -> Result<Option<Material>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, content_snippet, created_at FROM materials WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Material {
                id: row.get(0)?,
                path: row.get(1)?,
                content_snippet: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        match rows.next() {
            Some(Ok(m)) => Ok(Some(m)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    // --- Session-Material association ---

    pub fn add_material_to_session(&self, session_id: i64, material_id: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO session_materials (session_id, material_id) VALUES (?1, ?2)",
            params![session_id, material_id],
        )?;
        Ok(())
    }

    pub fn remove_material_from_session(&self, session_id: i64, material_id: i64) -> Result<()> {
        self.conn.execute(
            "DELETE FROM session_materials WHERE session_id = ?1 AND material_id = ?2",
            params![session_id, material_id],
        )?;
        Ok(())
    }

    pub fn get_session_materials(&self, session_id: i64) -> Result<Vec<Material>> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.path, m.content_snippet, m.created_at
             FROM materials m
             JOIN session_materials sm ON m.id = sm.material_id
             WHERE sm.session_id = ?1",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Material {
                id: row.get(0)?,
                path: row.get(1)?,
                content_snippet: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    // --- Thought operations ---

    pub fn create_thought(&self, session_id: i64, material_id: Option<i64>, content: &str) -> Result<Thought> {
        let ts = now();
        let next_order: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM thoughts WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .unwrap_or(1);
        self.conn.execute(
            "INSERT INTO thoughts (session_id, material_id, content, status, sort_order, created_at)
             VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
            params![session_id, material_id, content, next_order, ts],
        )?;
        let id = self.conn.last_insert_rowid();
        let status = ThoughtStatus::Pending;
        Ok(Thought {
            id,
            session_id,
            material_id,
            content: content.to_string(),
            status,
            sort_order: next_order,
            created_at: ts,
        })
    }

    pub fn update_thought_status(&self, id: i64, status: &ThoughtStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE thoughts SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        Ok(())
    }

    pub fn get_recent_thoughts(&self, session_id: i64, limit: usize) -> Result<Vec<Thought>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, material_id, content, status, sort_order, created_at
             FROM thoughts
             WHERE session_id = ?1
             ORDER BY sort_order DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            let status_str: String = row.get(4)?;
            let status = ThoughtStatus::try_from(status_str.as_str()).unwrap_or(ThoughtStatus::Pending);
            Ok(Thought {
                id: row.get(0)?,
                session_id: row.get(1)?,
                material_id: row.get(2)?,
                content: row.get(3)?,
                status,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_pending_thoughts_count(&self, session_id: i64) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM thoughts WHERE session_id = ?1 AND status = 'pending'",
            params![session_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    // --- Idea operations ---

    pub fn create_idea(&self, session_id: i64, content: &str) -> Result<Idea> {
        let ts = now();
        let next_order: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), 0) + 1 FROM ideas WHERE session_id = ?1",
                params![session_id],
                |row| row.get(0),
            )
            .unwrap_or(1);
        self.conn.execute(
            "INSERT INTO ideas (session_id, content, status, sort_order, created_at)
             VALUES (?1, ?2, 'pending', ?3, ?4)",
            params![session_id, content, next_order, ts],
        )?;
        let id = self.conn.last_insert_rowid();
        Ok(Idea {
            id,
            session_id,
            content: content.to_string(),
            status: IdeaStatus::Pending,
            sort_order: next_order,
            created_at: ts,
        })
    }

    pub fn update_idea_status(&self, id: i64, status: &IdeaStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE ideas SET status = ?1 WHERE id = ?2",
            params![status.as_str(), id],
        )?;
        Ok(())
    }

    pub fn get_recent_ideas(&self, session_id: i64, limit: usize) -> Result<Vec<Idea>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, status, sort_order, created_at
             FROM ideas
             WHERE session_id = ?1
             ORDER BY sort_order DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![session_id, limit as i64], |row| {
            let status_str: String = row.get(3)?;
            let status = IdeaStatus::try_from(status_str.as_str()).unwrap_or(IdeaStatus::Pending);
            Ok(Idea {
                id: row.get(0)?,
                session_id: row.get(1)?,
                content: row.get(2)?,
                status,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_accepted_ideas(&self, session_id: i64) -> Result<Vec<Idea>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, status, sort_order, created_at
             FROM ideas
             WHERE session_id = ?1 AND status = 'accepted'
             ORDER BY sort_order ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            let status_str: String = row.get(3)?;
            let status = IdeaStatus::try_from(status_str.as_str()).unwrap_or(IdeaStatus::Pending);
            Ok(Idea {
                id: row.get(0)?,
                session_id: row.get(1)?,
                content: row.get(2)?,
                status,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_latest_pending_idea(&self, session_id: i64) -> Result<Option<Idea>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, content, status, sort_order, created_at
             FROM ideas
             WHERE session_id = ?1 AND status = 'pending'
             ORDER BY sort_order DESC
             LIMIT 1",
        )?;
        let mut rows = stmt.query_map(params![session_id], |row| {
            let status_str: String = row.get(3)?;
            let status = IdeaStatus::try_from(status_str.as_str()).unwrap_or(IdeaStatus::Pending);
            Ok(Idea {
                id: row.get(0)?,
                session_id: row.get(1)?,
                content: row.get(2)?,
                status,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        match rows.next() {
            Some(Ok(idea)) => Ok(Some(idea)),
            Some(Err(e)) => Err(e.into()),
            None => Ok(None),
        }
    }

    // --- Idea-Thought associations ---

    pub fn link_idea_to_thoughts(&self, idea_id: i64, thought_ids: &[i64]) -> Result<()> {
        for tid in thought_ids {
            self.conn.execute(
                "INSERT OR IGNORE INTO idea_thoughts (idea_id, thought_id) VALUES (?1, ?2)",
                params![idea_id, tid],
            )?;
        }
        Ok(())
    }

    pub fn get_idea_thoughts(&self, idea_id: i64) -> Result<Vec<Thought>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.session_id, t.material_id, t.content, t.status, t.sort_order, t.created_at
             FROM thoughts t
             JOIN idea_thoughts it ON t.id = it.thought_id
             WHERE it.idea_id = ?1",
        )?;
        let rows = stmt.query_map(params![idea_id], |row| {
            let status_str: String = row.get(4)?;
            let status = ThoughtStatus::try_from(status_str.as_str()).unwrap_or(ThoughtStatus::Pending);
            Ok(Thought {
                id: row.get(0)?,
                session_id: row.get(1)?,
                material_id: row.get(2)?,
                content: row.get(3)?,
                status,
                sort_order: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>().map_err(Into::into)
    }

    // --- AI Context ---

    pub fn get_ai_context(
        &self,
        session_id: i64,
        thought_window: usize,
        max_tokens: usize,
    ) -> Result<AiContext> {
        let materials = self.get_session_materials(session_id)?;
        let thoughts = self.get_recent_thoughts(session_id, thought_window)?;
        let accepted_ideas = self.get_accepted_ideas(session_id)?;
        Ok(AiContext {
            materials,
            thoughts,
            accepted_ideas,
            max_tokens,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn create_test_session(db: &Database) -> Session {
        db.create_session(Some("test-session")).unwrap()
    }

    #[test]
    fn test_create_and_get_session() {
        let db = setup_db();
        let session = create_test_session(&db);
        assert!(session.id > 0);
        assert_eq!(session.title.as_deref(), Some("test-session"));
        assert!(!session.ai_pending);

        let fetched = db.get_session(session.id).unwrap().unwrap();
        assert_eq!(fetched.id, session.id);
        assert_eq!(fetched.title, session.title);
    }

    #[test]
    fn test_list_sessions() {
        let db = setup_db();
        assert!(db.list_sessions().unwrap().is_empty());

        db.create_session(Some("s1")).unwrap();
        db.create_session(Some("s2")).unwrap();
        assert_eq!(db.list_sessions().unwrap().len(), 2);
    }

    #[test]
    fn test_ai_pending_flag() {
        let db = setup_db();
        let session = create_test_session(&db);
        assert!(!session.ai_pending);

        db.set_session_ai_pending(session.id, true).unwrap();
        let updated = db.get_session(session.id).unwrap().unwrap();
        assert!(updated.ai_pending);

        db.set_session_ai_pending(session.id, false).unwrap();
        let updated = db.get_session(session.id).unwrap().unwrap();
        assert!(!updated.ai_pending);
    }

    #[test]
    fn test_create_and_get_material() {
        let db = setup_db();
        let m = db.create_material(Some("/path/to/file.txt"), Some("file content snippet")).unwrap();
        assert!(m.id > 0);

        let fetched = db.get_material(m.id).unwrap().unwrap();
        assert_eq!(fetched.path, Some("/path/to/file.txt".to_string()));
    }

    #[test]
    fn test_session_material_association() {
        let db = setup_db();
        let session = create_test_session(&db);
        let m = db.create_material(Some("doc.txt"), Some("doc content")).unwrap();

        // Initially no materials
        assert!(db.get_session_materials(session.id).unwrap().is_empty());

        // Add material
        db.add_material_to_session(session.id, m.id).unwrap();
        let mats = db.get_session_materials(session.id).unwrap();
        assert_eq!(mats.len(), 1);
        assert_eq!(mats[0].id, m.id);

        // Remove material
        db.remove_material_from_session(session.id, m.id).unwrap();
        assert!(db.get_session_materials(session.id).unwrap().is_empty());
    }

    #[test]
    fn test_create_and_list_thoughts() {
        let db = setup_db();
        let session = create_test_session(&db);

        let t1 = db.create_thought(session.id, None, "first thought").unwrap();
        let t2 = db.create_thought(session.id, None, "second thought").unwrap();

        assert_eq!(t1.sort_order, 1);
        assert_eq!(t2.sort_order, 2);
        assert_eq!(t1.status, ThoughtStatus::Pending);

        let recent = db.get_recent_thoughts(session.id, 10).unwrap();
        assert_eq!(recent.len(), 2);
        // Should be DESC by sort_order
        assert_eq!(recent[0].id, t2.id);
        assert_eq!(recent[1].id, t1.id);
    }

    #[test]
    fn test_update_thought_status() {
        let db = setup_db();
        let session = create_test_session(&db);
        let t = db.create_thought(session.id, None, "test").unwrap();

        db.update_thought_status(t.id, &ThoughtStatus::Processing).unwrap();
        let recent = db.get_recent_thoughts(session.id, 10).unwrap();
        assert_eq!(recent[0].status, ThoughtStatus::Processing);

        db.update_thought_status(t.id, &ThoughtStatus::Completed).unwrap();
        let recent = db.get_recent_thoughts(session.id, 10).unwrap();
        assert_eq!(recent[0].status, ThoughtStatus::Completed);
    }

    #[test]
    fn test_idea_lifecycle() {
        let db = setup_db();
        let session = create_test_session(&db);

        let idea = db.create_idea(session.id, "AI generated insight").unwrap();
        assert_eq!(idea.status, IdeaStatus::Pending);
        assert_eq!(idea.sort_order, 1);

        // Accept the idea
        db.update_idea_status(idea.id, &IdeaStatus::Accepted).unwrap();
        let accepted = db.get_accepted_ideas(session.id).unwrap();
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted[0].status, IdeaStatus::Accepted);

        // Reject next idea
        let idea2 = db.create_idea(session.id, "another idea").unwrap();
        db.update_idea_status(idea2.id, &IdeaStatus::Rejected).unwrap();

        let recent = db.get_recent_ideas(session.id, 10).unwrap();
        assert_eq!(recent.len(), 2);
        // Only accepted should appear
        let accepted = db.get_accepted_ideas(session.id).unwrap();
        assert_eq!(accepted.len(), 1);
    }

    #[test]
    fn test_idea_failed_status() {
        let db = setup_db();
        let session = create_test_session(&db);
        let idea = db.create_idea(session.id, "test idea").unwrap();
        db.update_idea_status(idea.id, &IdeaStatus::Failed).unwrap();

        let recent = db.get_recent_ideas(session.id, 10).unwrap();
        assert_eq!(recent[0].status, IdeaStatus::Failed);
    }

    #[test]
    fn test_idea_thought_association() {
        let db = setup_db();
        let session = create_test_session(&db);

        let t1 = db.create_thought(session.id, None, "thought 1").unwrap();
        let t2 = db.create_thought(session.id, None, "thought 2").unwrap();
        let idea = db.create_idea(session.id, "generated idea").unwrap();

        db.link_idea_to_thoughts(idea.id, &[t1.id, t2.id]).unwrap();

        let linked = db.get_idea_thoughts(idea.id).unwrap();
        assert_eq!(linked.len(), 2);
    }

    #[test]
    fn test_ai_context_includes_accepted_ideas() {
        let db = setup_db();
        let session = create_test_session(&db);

        // Add materials
        let m = db.create_material(Some("bug.txt"), Some("bug description")).unwrap();
        db.add_material_to_session(session.id, m.id).unwrap();

        // Add thoughts
        let t1 = db.create_thought(session.id, None, "step 1 to reproduce").unwrap();
        let _t2 = db.create_thought(session.id, None, "step 2 to reproduce").unwrap();

        // Add accepted idea (simulating previous round)
        let prev_idea = db.create_idea(session.id, "previous accepted conclusion").unwrap();
        db.update_idea_status(prev_idea.id, &IdeaStatus::Accepted).unwrap();
        db.link_idea_to_thoughts(prev_idea.id, &[t1.id]).unwrap();

        // Build context
        let ctx = db.get_ai_context(session.id, 10, 4096).unwrap();
        assert_eq!(ctx.materials.len(), 1);
        assert_eq!(ctx.thoughts.len(), 2);
        assert_eq!(ctx.accepted_ideas.len(), 1);
        assert_eq!(ctx.accepted_ideas[0].content, "previous accepted conclusion");
    }

    #[test]
    fn test_latest_pending_idea() {
        let db = setup_db();
        let session = create_test_session(&db);

        assert!(db.get_latest_pending_idea(session.id).unwrap().is_none());

        let idea = db.create_idea(session.id, "new idea").unwrap();
        let latest = db.get_latest_pending_idea(session.id).unwrap().unwrap();
        assert_eq!(latest.id, idea.id);

        db.update_idea_status(idea.id, &IdeaStatus::Accepted).unwrap();
        assert!(db.get_latest_pending_idea(session.id).unwrap().is_none());
    }

    #[test]
    fn test_pending_thoughts_count() {
        let db = setup_db();
        let session = create_test_session(&db);

        assert_eq!(db.get_pending_thoughts_count(session.id).unwrap(), 0);

        db.create_thought(session.id, None, "t1").unwrap();
        db.create_thought(session.id, None, "t2").unwrap();
        assert_eq!(db.get_pending_thoughts_count(session.id).unwrap(), 2);
    }

    #[test]
    fn test_sort_order_is_monotonic() {
        let db = setup_db();
        let session = create_test_session(&db);

        for i in 0..5 {
            let t = db.create_thought(session.id, None, &format!("thought {i}")).unwrap();
            assert_eq!(t.sort_order, i + 1);
        }
    }
}
