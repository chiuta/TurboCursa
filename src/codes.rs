//! Every code the player can type or paste: championships, stage codes, ghosts, paint words and the
//! two challenges. One parser for the native start screen and the browser page.
use crate::champ::ChampCode;
use crate::ghost::Ghost;
use crate::share::StageCode;

/// Longest text taken as a code (a ghost of ten minutes is the largest).
pub const MAX_LEN: usize = 12_000;

#[derive(Clone, Debug, PartialEq)]
pub enum Code {
    Champ(ChampCode),
    Stage(StageCode),
    Ghost(Box<Ghost>),
    /// a paint job (1..)
    Paint(u8),
    Daily,
    Weekly,
}

/// Hostile-safe: anything that is not a valid code gives `None`.
pub fn parse(txt: &str) -> Option<Code> {
    let t = txt.trim();
    if t.is_empty() || t.len() > MAX_LEN || t.chars().any(|c| c.is_control()) {
        return None;
    }
    let up = t.to_ascii_uppercase();
    if up.starts_with("TCC1-") {
        return ChampCode::decode(t)
            .or_else(|| ChampCode::decode(&up))
            .map(Code::Champ);
    }
    if up.starts_with("TCS1-") {
        return StageCode::decode(t)
            .or_else(|| StageCode::decode(&up))
            .map(Code::Stage);
    }
    if t.starts_with("TG1;") || t.starts_with("TG2;") {
        return Ghost::decode(t).map(|g| Code::Ghost(Box::new(g)));
    }
    match up.as_str() {
        "DAILY" => return Some(Code::Daily),
        "WEEKLY" => return Some(Code::Weekly),
        _ => {}
    }
    crate::settings::secret_code(t).map(Code::Paint)
}

impl Code {
    /// Command-line arguments that start the game for this code (`None` = applied in place).
    pub fn launch_args(&self, raw: &str) -> Option<Vec<String>> {
        match self {
            Code::Champ(_) => Some(vec!["--champ".into(), raw.trim().into()]),
            Code::Stage(_) => Some(vec!["--code".into(), raw.trim().into()]),
            Code::Daily => Some(vec!["--daily".into()]),
            Code::Weekly => Some(vec!["--weekly".into()]),
            Code::Ghost(_) | Code::Paint(_) => None,
        }
    }

    /// Small integer for the browser page: 1 championship, 2 stage, 3 ghost, 4 paint, 5 daily, 6 weekly.
    pub fn kind(&self) -> u32 {
        match self {
            Code::Champ(_) => 1,
            Code::Stage(_) => 2,
            Code::Ghost(_) => 3,
            Code::Paint(_) => 4,
            Code::Daily => 5,
            Code::Weekly => 6,
        }
    }
}
