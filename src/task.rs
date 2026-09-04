#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskProgress {
    pub description: String,
    pub percent: u8,
}

impl TaskProgress {
    pub fn new(description: impl Into<String>, percent: u8) -> Self {
        Self {
            description: description.into(),
            percent: percent.min(100),
        }
    }

    pub(crate) fn from_count(
        description: impl Into<String>,
        completed: usize,
        total: usize,
    ) -> Self {
        let percent = if total == 0 {
            100
        } else {
            ((completed.saturating_mul(100) / total).min(100)) as u8
        };
        Self {
            description: description.into(),
            percent,
        }
    }
}
