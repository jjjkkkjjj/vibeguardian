use aho_corasick::AhoCorasick;

const MASK: &str = "***[MASKED]***";

/// Masks secret values in log lines using Aho-Corasick multi-string search.
/// Pre-compile once with all known secret values; call [`LogMasker::mask`] per line.
pub struct LogMasker {
    ac: Option<AhoCorasick>,
    /// One MASK entry per pattern, pre-built so `replace_all` gets the right count.
    replacements: Vec<String>,
}

impl LogMasker {
    /// Build the masker. If `secrets` is empty, masking is a no-op.
    pub fn new(secrets: &[String]) -> anyhow::Result<Self> {
        let non_empty: Vec<&String> = secrets.iter().filter(|s| !s.is_empty()).collect();
        if non_empty.is_empty() {
            return Ok(Self {
                ac: None,
                replacements: vec![],
            });
        }
        let count = non_empty.len();
        let ac = AhoCorasick::new(non_empty)?;
        Ok(Self {
            ac: Some(ac),
            replacements: vec![MASK.to_string(); count],
        })
    }

    /// Return a copy of `line` with all secret values replaced by `***[MASKED]***`.
    pub fn mask(&self, line: &str) -> String {
        match &self.ac {
            None => line.to_string(),
            Some(ac) => {
                let reps: Vec<&str> = self.replacements.iter().map(|s| s.as_str()).collect();
                ac.replace_all(line, &reps)
            }
        }
    }
}
