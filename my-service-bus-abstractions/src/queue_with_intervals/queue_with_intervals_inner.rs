use super::QueueIndexRange;
const MIN_CAPACITY: usize = 2;

#[derive(Debug, Clone)]
pub struct QueueWithIntervalsInner {
    first: QueueIndexRange,
    additional: Vec<QueueIndexRange>,
}

impl QueueWithIntervalsInner {
    pub fn new(start_id: i64) -> Self {
        Self {
            first: QueueIndexRange::new_empty(start_id),
            additional: Vec::new(),
        }
    }

    pub fn restore(mut intervals: Vec<QueueIndexRange>) -> Self {
        if intervals.len() == 0 {
            return Self::new(0);
        }

        let first = intervals.remove(0);

        return Self {
            first,
            additional: intervals,
        };
    }

    pub fn from_single_interval(from_id: i64, to_id: i64) -> Self {
        Self {
            first: QueueIndexRange { from_id, to_id },
            additional: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&QueueIndexRange> {
        if index == 0 {
            if self.first.is_empty() {
                return None;
            }

            return Some(&self.first);
        }
        self.additional.get(index - 1)
    }

    pub fn push(&mut self, item: QueueIndexRange) {
        if self.first.is_empty() {
            self.first = item;
            return;
        }

        self.additional.push(item);
    }

    pub fn get_two(
        &self,
        first_index: usize,
    ) -> (Option<&QueueIndexRange>, Option<&QueueIndexRange>) {
        if first_index == 0 {
            if self.first.is_empty() {
                return (None, None);
            }

            return (Some(&self.first), self.additional.get(0));
        }
        return (
            self.additional.get(first_index - 1),
            self.additional.get(first_index),
        );
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut QueueIndexRange> {
        if index == 0 {
            if self.first.is_empty() {
                return None;
            }

            return Some(&mut self.first);
        }
        self.additional.get_mut(index - 1)
    }

    fn shrink_dynamic_content(&mut self) {
        if self.additional.len() < MIN_CAPACITY {
            self.additional.shrink_to(MIN_CAPACITY);
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<QueueIndexRange> {
        if index == 0 {
            if self.first.is_empty() {
                return None;
            }
            let result = self.first.clone();

            if self.additional.len() == 0 {
                self.first.reset();
                return Some(result);
            }
            self.first = self.additional.remove(0);
            self.shrink_dynamic_content();
            return Some(result);
        }

        let index = index - 1;

        if index < self.additional.len() {
            let result = self.additional.remove(index);
            self.shrink_dynamic_content();
            return Some(result);
        }

        None
    }

    pub fn reset(&mut self, mut intervals: Vec<QueueIndexRange>) {
        self.additional.clear();
        self.shrink_dynamic_content();

        if intervals.len() == 0 {
            self.first.reset();
            return;
        }

        self.first = intervals.remove(0);
        self.additional = intervals;
    }

    pub fn clean(&mut self) {
        self.first.reset();
        self.additional.clear();
        self.shrink_dynamic_content();
    }

    pub fn insert(&mut self, index: usize, item: QueueIndexRange) {
        if index == 0 {
            if !self.first.is_empty() {
                self.additional.insert(0, self.first.clone());
            }
            self.first = item;
            return;
        }

        self.additional.insert(index - 1, item);
    }

    pub fn update(&mut self, index: usize, item: QueueIndexRange) {
        if index == 0 {
            self.first = item;
            return;
        }

        let index = index - 1;
        self.additional[index] = item;
    }

    pub fn queue_size(&self) -> usize {
        let mut result = self.first.len();

        for interval in &self.additional {
            result += interval.len();
        }

        return result as usize;
    }

    pub fn intervals_amount(&self) -> usize {
        if self.first.is_empty() {
            return 0;
        }

        self.additional.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        if self.additional.len() > 0 {
            return false;
        }

        self.first.is_empty()
    }

    pub fn find_my_interval_index(&self, id: i64) -> Option<usize> {
        if self.first.is_in_my_interval(id) {
            return Some(0);
        }

        self.additional
            .iter()
            .enumerate()
            .find_map(|(index, item)| {
                if item.is_in_my_interval(id) {
                    return Some(index + 1);
                }

                None
            })
    }

    pub fn get_range_mut(&mut self, index: usize) -> Option<&mut QueueIndexRange> {
        if index == 0 {
            if self.first.is_empty() {
                return None;
            }

            return Some(&mut self.first);
        }

        self.additional.get_mut(index - 1)
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        if self.first.is_empty() {
            return vec![];
        }

        let mut result = Vec::with_capacity(self.additional.len() + 1);
        result.push(self.first.clone());
        result.extend_from_slice(self.additional.as_slice());

        result
    }

    pub fn get_min_id(&self) -> Option<i64> {
        if self.first.is_empty() {
            return None;
        }

        Some(self.first.from_id)
    }

    pub fn get_min_id_even_if_empty(&self) -> Option<i64> {
        if self.first.from_id < 0 {
            return None;
        }

        Some(self.first.from_id)
    }

    pub fn get_max_id(&self) -> Option<i64> {
        if self.first.is_empty() {
            return None;
        }

        if self.additional.len() == 0 {
            return Some(self.first.to_id);
        }

        let result = self.additional.get(self.additional.len() - 1)?;

        Some(result.to_id)
    }

    pub fn find(&self, callback: impl Fn(&QueueIndexRange) -> bool) -> Option<&QueueIndexRange> {
        if self.first.is_empty() {
            return None;
        }

        if callback(&self.first) {
            return Some(&self.first);
        }

        for item in &self.additional {
            if callback(item) {
                return Some(item);
            }
        }

        None
    }

    pub fn has_item(&self, callback: impl Fn(&QueueIndexRange) -> bool) -> bool {
        if self.first.is_empty() {
            return false;
        }

        if callback(&self.first) {
            return true;
        }

        for item in &self.additional {
            if callback(item) {
                return true;
            }
        }

        false
    }

    pub fn iter(&self) -> QueueWithIntervalsInnerIterator {
        QueueWithIntervalsInnerIterator::new(self)
    }
}

pub struct QueueWithIntervalsInnerIterator {
    first: Option<QueueIndexRange>,
    additional: Vec<QueueIndexRange>,
}

impl QueueWithIntervalsInnerIterator {
    pub fn new(inner: &QueueWithIntervalsInner) -> Self {
        Self {
            first: Some(inner.first.clone()),
            additional: inner.additional.clone(),
        }
    }
}

impl Iterator for QueueWithIntervalsInnerIterator {
    type Item = QueueIndexRange;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            return Some(first);
        }

        if self.additional.len() == 0 {
            return None;
        }

        Some(self.additional.remove(0))
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::QueueIndexRange;

    use super::QueueWithIntervalsInner;

    #[test]
    fn test_len() {
        let mut inner = QueueWithIntervalsInner::new(0);

        assert_eq!(inner.intervals_amount(), 0);

        inner.push(QueueIndexRange {
            from_id: 0,
            to_id: 10,
        });

        assert_eq!(inner.intervals_amount(), 1);

        inner.push(QueueIndexRange {
            from_id: 15,
            to_id: 20,
        });

        assert_eq!(inner.intervals_amount(), 2);
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
