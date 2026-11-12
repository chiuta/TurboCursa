//! Which kind of game a start request launches. The native binary and the tests share this rule,
//! so a split-screen setting can never hide the career, the rally or a championship.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Launch {
    /// 3 or 4 cars in a 2x2 split screen
    Multi,
    Career,
    /// championship or country tour
    Champ,
    Rally,
    Versus,
    Single,
}

/// `players` is the split-screen setting (0 = off, 3 or 4 on). Split screen only replaces the
/// plain race and the two-player modes; the career, the rally and the championships keep their own screens.
pub fn decide(players: usize, career: bool, champ: bool, rally: bool, versus: bool) -> Launch {
    if career {
        Launch::Career
    } else if champ {
        Launch::Champ
    } else if rally {
        Launch::Rally
    } else if players >= 3 {
        Launch::Multi
    } else if versus {
        Launch::Versus
    } else {
        Launch::Single
    }
}
