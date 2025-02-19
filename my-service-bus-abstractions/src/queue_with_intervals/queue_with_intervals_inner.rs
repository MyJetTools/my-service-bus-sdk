use super::{QueueIndexRange, QueueWithIntervalsIteratorInner};
const MIN_CAPACITY: usize = 2;

#[derive(Debug, Clone)]
pub struct QueueWithIntervalsInner {
    intervals: Vec<QueueIndexRange>,
}

impl QueueWithIntervalsInner {
    pub fn new(start_id: i64) -> Self {
        Self {
            intervals: vec![QueueIndexRange::new_empty(start_id)],
        }
    }

    pub fn restore(mut intervals: Vec<QueueIndexRange>) -> Self {
        if intervals.len() == 0 {
            return Self::new(0);
        }

        intervals.sort_by_key(|itm| itm.from_id);

        return Self { intervals };
    }

    pub fn from_single_interval(from_id: i64, to_id: i64) -> Self {
        Self {
            intervals: vec![QueueIndexRange { from_id, to_id }],
        }
    }

    pub fn get(&self, index: usize) -> Option<&QueueIndexRange> {
        self.intervals.get(index)
    }

    pub fn merge(&mut self, new_item: QueueIndexRange) {
        if self.intervals.get(0).unwrap().is_empty() {
            self.intervals[0] = new_item;
            return;
        }

        let mut insert_index = 0;

        for itm in &self.intervals {
            if new_item.to_id < itm.from_id {
                break;
            }
        }
    }

    pub fn get_two(
        &self,
        first_index: usize,
    ) -> (Option<&QueueIndexRange>, Option<&QueueIndexRange>) {
        return (
            self.intervals.get(first_index),
            self.intervals.get(first_index + 1),
        );
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut QueueIndexRange> {
        self.intervals.get_mut(index)
    }

    fn shrink_dynamic_content(&mut self) {
        if self.intervals.len() < MIN_CAPACITY {
            self.intervals.shrink_to(MIN_CAPACITY);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<QueueIndexRange> {
        if index < self.intervals.len() {
            let result = self.intervals.remove(index);
            self.shrink_dynamic_content();
            return Some(result);
        }

        None
    }

    pub fn reset(&mut self, mut intervals: Vec<QueueIndexRange>) {
        intervals.sort_by_key(|itm| itm.from_id);
        self.intervals = intervals;
    }

    pub fn clean(&mut self) {
        self.intervals.truncate(1);
        self.intervals.get_mut(0).unwrap().reset();
        self.shrink_dynamic_content();
    }

    pub fn insert(&mut self, index: usize, item: QueueIndexRange) {
        self.intervals.insert(index, item);
    }

    pub fn update(&mut self, index: usize, item: QueueIndexRange) {
        self.intervals[index] = item;
    }

    pub fn queue_size(&self) -> usize {
        let mut result = 0;

        for interval in &self.intervals {
            result += interval.len();
        }

        return result as usize;
    }

    pub fn intervals_amount(&self) -> usize {
        self.intervals.len()
    }

    pub fn is_empty(&self) -> bool {
        if self.intervals.len() == 1 {
            return self.intervals.get(0).unwrap().is_empty();
        }

        false
    }

    pub fn find_my_interval_index(&self, id: i64) -> Option<usize> {
        self.intervals.iter().enumerate().find_map(|(index, item)| {
            if item.is_in_my_interval(id) {
                return Some(index);
            }

            None
        })
    }

    pub fn get_range_mut(&mut self, index: usize) -> Option<&mut QueueIndexRange> {
        self.intervals.get_mut(index)
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        if self.get(0).unwrap().is_empty() {
            return vec![];
        }

        self.intervals.clone()
    }

    pub fn get_min_id(&self) -> Option<i64> {
        let first = self.get(0).unwrap();
        if first.is_empty() {
            return None;
        }

        Some(first.from_id)
    }

    pub fn get_max_id(&self) -> Option<i64> {
        let last = self.get(self.intervals.len() - 1).unwrap();
        if last.is_empty() {
            return None;
        }

        Some(last.to_id)
    }

    pub fn find(&self, callback: impl Fn(&QueueIndexRange) -> bool) -> Option<&QueueIndexRange> {
        for item in &self.intervals {
            if callback(item) {
                return Some(item);
            }
        }

        None
    }

    pub fn has_item(&self, callback: impl Fn(&QueueIndexRange) -> bool) -> bool {
        for item in &self.intervals {
            if callback(item) {
                return true;
            }
        }

        false
    }

    pub fn iter(&self) -> QueueWithIntervalsIteratorInner {
        QueueWithIntervalsIteratorInner::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::QueueIndexRange;

    use super::QueueWithIntervalsInner;

    #[test]
    fn test_len() {
        let mut inner = QueueWithIntervalsInner::new(0);

        assert_eq!(inner.intervals_amount(), 1);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        assert_eq!(inner.intervals_amount(), 2);

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        assert_eq!(inner.intervals_amount(), 3);
    }

    #[test]
    fn remove_first_element_when_we_have_3() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        assert_eq!(inner.intervals_amount(), 3);

        let removed = inner.remove(0).unwrap();

        assert_eq!(removed.from_id, 0);
        assert_eq!(removed.to_id, 10);

        assert_eq!(inner.intervals_amount(), 2);

        assert_eq!(inner.get(0).unwrap().from_id, 15);
        assert_eq!(inner.get(0).unwrap().to_id, 20);

        assert_eq!(inner.get(1).unwrap().from_id, 25);
        assert_eq!(inner.get(1).unwrap().to_id, 30);
    }

    #[test]
    fn remove_second_element_when_we_have_3() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        assert_eq!(inner.intervals_amount(), 3);

        let removed = inner.remove(1).unwrap();

        assert_eq!(removed.from_id, 15);
        assert_eq!(removed.to_id, 20);

        assert_eq!(inner.intervals_amount(), 2);

        assert_eq!(inner.get(0).unwrap().from_id, 0);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 25);
        assert_eq!(inner.get(1).unwrap().to_id, 30);
    }

    #[test]
    fn remove_third_element_when_we_have_3() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        assert_eq!(inner.intervals_amount(), 3);

        let removed = inner.remove(2).unwrap();

        assert_eq!(removed.from_id, 25);
        assert_eq!(removed.to_id, 30);

        assert_eq!(inner.intervals_amount(), 2);

        assert_eq!(inner.get(0).unwrap().from_id, 0);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 15);
        assert_eq!(inner.get(1).unwrap().to_id, 20);
    }

    #[test]
    fn update_first_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.update(
            0,
            QueueIndexRange {
                from_id: 1,
                to_id: 9,
            },
        );

        assert_eq!(inner.intervals_amount(), 3);

        assert_eq!(inner.get(0).unwrap().from_id, 1);
        assert_eq!(inner.get(0).unwrap().to_id, 9);

        assert_eq!(inner.get(1).unwrap().from_id, 15);
        assert_eq!(inner.get(1).unwrap().to_id, 20);

        assert_eq!(inner.get(2).unwrap().from_id, 25);
        assert_eq!(inner.get(2).unwrap().to_id, 30);
    }

    #[test]
    fn update_second_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.update(
            1,
            QueueIndexRange {
                from_id: 12,
                to_id: 13,
            },
        );

        assert_eq!(inner.intervals_amount(), 3);

        assert_eq!(inner.get(0).unwrap().from_id, 0);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 12);
        assert_eq!(inner.get(1).unwrap().to_id, 13);

        assert_eq!(inner.get(2).unwrap().from_id, 25);
        assert_eq!(inner.get(2).unwrap().to_id, 30);
    }

    #[test]
    fn update_third_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.update(
            2,
            QueueIndexRange {
                from_id: 22,
                to_id: 23,
            },
        );

        assert_eq!(inner.intervals_amount(), 3);

        assert_eq!(inner.get(0).unwrap().from_id, 0);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 15);
        assert_eq!(inner.get(1).unwrap().to_id, 20);

        assert_eq!(inner.get(2).unwrap().from_id, 22);
        assert_eq!(inner.get(2).unwrap().to_id, 23);
    }

    #[test]
    fn insert_first_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 5,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.insert(
            0,
            QueueIndexRange {
                from_id: 0,
                to_id: 1,
            },
        );

        assert_eq!(inner.intervals_amount(), 4);

        assert_eq!(inner.get(0).unwrap().from_id, 0);
        assert_eq!(inner.get(0).unwrap().to_id, 1);

        assert_eq!(inner.get(1).unwrap().from_id, 5);
        assert_eq!(inner.get(1).unwrap().to_id, 10);

        assert_eq!(inner.get(2).unwrap().from_id, 15);
        assert_eq!(inner.get(2).unwrap().to_id, 20);

        assert_eq!(inner.get(3).unwrap().from_id, 25);
        assert_eq!(inner.get(3).unwrap().to_id, 30);
    }

    #[test]
    fn insert_second_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 5,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.insert(
            1,
            QueueIndexRange {
                from_id: 11,
                to_id: 12,
            },
        );

        assert_eq!(inner.intervals_amount(), 4);

        assert_eq!(inner.get(0).unwrap().from_id, 5);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 11);
        assert_eq!(inner.get(1).unwrap().to_id, 12);

        assert_eq!(inner.get(2).unwrap().from_id, 15);
        assert_eq!(inner.get(2).unwrap().to_id, 20);

        assert_eq!(inner.get(3).unwrap().from_id, 25);
        assert_eq!(inner.get(3).unwrap().to_id, 30);
    }

    #[test]
    fn insert_third_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 5,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.insert(
            2,
            QueueIndexRange {
                from_id: 22,
                to_id: 23,
            },
        );

        assert_eq!(inner.intervals_amount(), 4);

        assert_eq!(inner.get(0).unwrap().from_id, 5);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 15);
        assert_eq!(inner.get(1).unwrap().to_id, 20);

        assert_eq!(inner.get(2).unwrap().from_id, 22);
        assert_eq!(inner.get(2).unwrap().to_id, 23);

        assert_eq!(inner.get(3).unwrap().from_id, 25);
        assert_eq!(inner.get(3).unwrap().to_id, 30);
    }

    #[test]
    fn insert_fourth_element() {
        let mut inner = QueueWithIntervalsInner::new(0);

        inner.push(QueueIndexRange {
            from_id: 5,
            to_id: 10,
        });

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        inner.push(QueueIndexRange {
            from_id: 25,
            to_id: 30,
        });

        inner.insert(
            3,
            QueueIndexRange {
                from_id: 40,
                to_id: 45,
            },
        );

        assert_eq!(inner.intervals_amount(), 4);

        assert_eq!(inner.get(0).unwrap().from_id, 5);
        assert_eq!(inner.get(0).unwrap().to_id, 10);

        assert_eq!(inner.get(1).unwrap().from_id, 15);
        assert_eq!(inner.get(1).unwrap().to_id, 20);

        assert_eq!(inner.get(2).unwrap().from_id, 25);
        assert_eq!(inner.get(2).unwrap().to_id, 30);

        assert_eq!(inner.get(3).unwrap().from_id, 40);
        assert_eq!(inner.get(3).unwrap().to_id, 45);
    }
}
