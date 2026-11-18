use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Aspect {
    Vertical,
    Landscape,
    Square,
    Portrait,
    Cinema,
}

impl Aspect {
    pub const ALL: [Aspect; 5] = [
        Aspect::Vertical,
        Aspect::Landscape,
        Aspect::Square,
        Aspect::Portrait,
        Aspect::Cinema,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Aspect::Vertical => "9:16",
            Aspect::Landscape => "16:9",
            Aspect::Square => "1:1",
            Aspect::Portrait => "4:5",
            Aspect::Cinema => "2.39:1",
        }
    }

    pub fn caption(self) -> &'static str {
        match self {
            Aspect::Vertical => "Reels / TikTok",
            Aspect::Landscape => "YouTube",
            Aspect::Square => "Square",
            Aspect::Portrait => "Portrait",
            Aspect::Cinema => "Cinema",
        }
    }

    /// Width divided by height.
    pub fn ratio(self) -> f32 {
        match self {
            Aspect::Vertical => 9.0 / 16.0,
            Aspect::Landscape => 16.0 / 9.0,
            Aspect::Square => 1.0,
            Aspect::Portrait => 4.0 / 5.0,
            Aspect::Cinema => 2.39,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Resolution {
    P720,
    P1080,
    K2,
    K4,
}

impl Resolution {
    pub const ALL: [Resolution; 4] = [
        Resolution::P720,
        Resolution::P1080,
        Resolution::K2,
        Resolution::K4,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Resolution::P720 => "720p",
            Resolution::P1080 => "1080p",
            Resolution::K2 => "2K",
            Resolution::K4 => "4K",
        }
    }
}

pub const FRAME_RATES: [u32; 3] = [24, 30, 60];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub aspect: Aspect,
    pub resolution: Resolution,
    pub fps: u32,
    pub background: [u8; 3],
    pub blur_background: bool,
    pub duration_ms: u64,
    /// Unix seconds
    pub modified: u64,
}

impl Project {
    pub fn new(
        name: String,
        aspect: Aspect,
        resolution: Resolution,
        fps: u32,
        background: [u8; 3],
        blur_background: bool,
    ) -> Self {
        Self {
            id: new_id(),
            name,
            aspect,
            resolution,
            fps,
            background,
            blur_background,
            duration_ms: 0,
            modified: now_secs(),
        }
    }

    pub fn duplicate(&self) -> Self {
        Self {
            id: new_id(),
            name: format!("{} copy", self.name),
            modified: now_secs(),
            ..self.clone()
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// Nanosecond timestamp keeps ids unique even for back-to-back duplicates
fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}

pub fn format_duration(ms: u64) -> String {
    let secs = ms / 1000;
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn open(dir: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn load_all(&self) -> Vec<Project> {
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut projects: Vec<Project> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .filter_map(|p| fs::read_to_string(p).ok())
            // A corrupt file shouldn't stop the rest from loading
            .filter_map(|s| serde_json::from_str(&s).ok())
            .collect();
        projects.sort_by(|a, b| b.modified.cmp(&a.modified));
        projects
    }

    pub fn save(&self, project: &Project) -> io::Result<()> {
        let json = serde_json::to_string_pretty(project)?;
        let path = self.dir.join(format!("{}.json", project.id));
        // Write then rename so a crash mid-save can't leave a half-written project
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, json)?;
        fs::rename(tmp, path)
    }

    pub fn delete(&self, id: &str) -> io::Result<()> {
        fs::remove_file(self.dir.join(format!("{id}.json")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_formats_as_minutes_and_seconds() {
        assert_eq!(format_duration(0), "0:00");
        assert_eq!(format_duration(65_000), "1:05");
    }

    #[test]
    fn duplicate_gets_a_new_id() {
        let p = Project::new("a".into(), Aspect::Square, Resolution::P720, 30, [0; 3], false);
        let d = p.duplicate();
        assert_ne!(p.id, d.id);
        assert_eq!(d.name, "a copy");
    }
}
