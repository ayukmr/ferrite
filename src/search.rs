use crate::syntax::HighlightType;

pub struct SearchIndex {
    // search index
    pub idx: usize,

    // previous highlight
    pub prev_row: Option<(usize, Vec<HighlightType>)>,
}

impl SearchIndex {
    // make new search index
    pub fn new() -> Self {
        Self {
            idx: 0,
            prev_row: None,
        }
    }

    // reset search index
    pub fn reset(&mut self) {
        self.idx = 0;
        self.prev_row = None;
    }
}
