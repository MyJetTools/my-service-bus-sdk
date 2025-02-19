use super::QueueIndexRange;

pub struct QueueWithIntervalsIteratorInner {
    intervals: Vec<QueueIndexRange>,
}

impl QueueWithIntervalsIteratorInner {
    pub fn new(inner: &QueueWithIntervalsInner) -> Self {
        Self {
            intervals: inner.get_snapshot(),
        }
    }
}

impl Iterator for QueueWithIntervalsIteratorInner {
    type Item = QueueIndexRange;

    fn next(&mut self) -> Option<Self::Item> {
        if self.intervals.len() == 0 {
            return None;
        }

        Some(self.intervals.remove(0))
    }
}
