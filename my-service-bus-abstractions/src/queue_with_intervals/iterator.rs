use super::{QueueIndexRange, QueueWithIntervals};

pub struct QueueWithIntervalsIterator {
    intervals: QueueWithIntervals,
}

impl QueueWithIntervalsIterator {
    pub fn new(intervals: QueueWithIntervals) -> Self {
        Self { intervals }
    }
}

impl Iterator for QueueWithIntervalsIterator {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        return self.intervals.dequeue();
    }
}

pub struct QueueIndexRangeIterator {
    from_id: i64,
    to_id: i64,
}

impl QueueIndexRangeIterator {
    pub fn new(range: &QueueIndexRange) -> Self {
        Self {
            from_id: range.from_id,
            to_id: range.to_id,
        }
    }
}
impl Iterator for QueueIndexRangeIterator {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        if self.from_id <= self.to_id {
            let result = self.from_id;
            self.from_id = self.from_id + 1;
            return Some(result);
        }

        return None;
    }
}

impl IntoIterator for QueueIndexRange {
    type Item = i64;
    type IntoIter = QueueIndexRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        QueueIndexRangeIterator::new(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(5);
        queue.enqueue(6);

        let mut result: Vec<i64> = Vec::new();
        result.extend(queue);

        assert_eq!(2, result.len());
        assert_eq!(5, result[0]);
        assert_eq!(6, result[1]);
    }
}
