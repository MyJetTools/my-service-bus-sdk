use crate::queue_with_intervals::queue_index_range::QueueIndexRange;

use super::{iterator::QueueWithIntervalsIterator, *};

//Illustrations are https://docs.google.com/spreadsheets/d/1oRFoiUkPm3h8Tz3BSVNCSBG3_pM84MlZLpJDCAPGKLs/edit?gid=0#gid=0

#[derive(Debug, Clone)]
pub enum QueueWithIntervalsError {
    MessagesNotFound,
    QueueIsEmpty,
    MessageExists,
}

#[derive(Debug, Clone)]
pub struct QueueWithIntervals {
    intervals: Vec<QueueIndexRange>,
}

impl QueueWithIntervals {
    pub fn new() -> QueueWithIntervals {
        Self {
            intervals: vec![QueueIndexRange::new_empty(0)],
        }
    }

    pub fn merge(&mut self, other: Self) {
        for other_interval in other.intervals.into_iter().rev() {
            self.enqueue_range(other_interval);
        }
    }

    pub fn get_interval(&self, index: usize) -> Option<&QueueIndexRange> {
        self.intervals.get(index)
    }

    pub fn get_intervals(&self) -> &[QueueIndexRange] {
        self.intervals.as_slice()
    }

    pub fn restore(mut intervals: Vec<QueueIndexRange>) -> Self {
        if intervals.len() == 0 {
            return Self {
                intervals: vec![QueueIndexRange::new_empty(0)],
            };
        }
        intervals.sort_by_key(|itm| itm.from_id);
        Self { intervals }
    }

    pub fn from_single_interval(from_id: i64, to_id: i64) -> Self {
        Self {
            intervals: vec![QueueIndexRange { from_id, to_id }],
        }
    }

    pub fn reset(&mut self, mut intervals: Vec<QueueIndexRange>) {
        if intervals.is_empty() {
            self.clean();
            return;
        }

        intervals.sort_by_key(|itm| itm.from_id);
        self.intervals = intervals;
    }

    pub fn clean(&mut self) {
        let to_id = self.intervals.last().unwrap().to_id;

        self.intervals.truncate(1);
        let first = self.intervals.get_mut(0).unwrap();

        first.to_id = to_id;
        first.make_empty();
    }

    pub fn is_empty(&self) -> bool {
        if self.intervals.len() == 1 {
            return self.intervals.get(0).unwrap().is_empty();
        }

        false
    }

    pub fn remove(&mut self, value: i64) -> Result<(), QueueWithIntervalsError> {
        if self.is_empty() {
            return Err(QueueWithIntervalsError::QueueIsEmpty);
        }

        let mut index = 0;
        let mut split = None;

        let mut removed = false;
        for interval in &mut self.intervals {
            if interval.is_in_my_interval(value) {
                if interval.from_id == value {
                    interval.from_id += 1;
                    removed = true;
                    break;
                }

                if interval.to_id == value {
                    interval.to_id -= 1;
                    removed = true;
                    break;
                }

                split = Some((
                    QueueIndexRange {
                        from_id: interval.from_id,
                        to_id: value - 1,
                    },
                    QueueIndexRange {
                        from_id: value + 1,
                        to_id: interval.to_id,
                    },
                ));

                removed = true;
                break;
            }
            index += 1;
        }

        if let Some(split) = split {
            self.intervals[index] = split.0;
            self.intervals.insert(index + 1, split.1);
            return Ok(());
        } else {
            if self.intervals.len() > 1 {
                if self.intervals[index].is_empty() {
                    self.remove_interval(index);
                }
            }
        }

        if removed {
            return Ok(());
        }
        return Err(QueueWithIntervalsError::MessagesNotFound);
    }

    fn remove_interval(&mut self, index: usize) {
        if self.intervals.len() > 1 {
            self.intervals.remove(index);
            return;
        }

        let first = self.intervals.first_mut().unwrap();

        if !first.is_empty() {
            first.make_empty()
        }
    }

    pub fn enqueue(&mut self, value: i64) {
        if let Some(first) = self.intervals.first_mut() {
            if first.is_empty() {
                first.from_id = value;
                first.to_id = value;
                return;
            }
        }

        match IndexToInsertValue::new(&self.intervals, value) {
            IndexToInsertValue::MergeToLeft(index) => {
                self.intervals.get_mut(index).unwrap().from_id -= 1;
            }
            IndexToInsertValue::MergeToRight(index) => {
                self.intervals.get_mut(index).unwrap().to_id += 1;
            }
            IndexToInsertValue::InsertAsNewInterval(index) => {
                self.intervals.insert(
                    index,
                    QueueIndexRange {
                        from_id: value,
                        to_id: value,
                    },
                );
            }
            IndexToInsertValue::MergeTwoIntervals(index) => {
                let value = self.intervals.remove(index + 1);
                if self.intervals.len() == 0 {
                    panic!("Somehow intervals got empty");
                }
                self.intervals.get_mut(index).unwrap().to_id = value.to_id;
            }
            IndexToInsertValue::HasValue => {}
        }
    }

    pub fn enqueue_range(&mut self, range_to_insert: QueueIndexRange) {
        if range_to_insert.is_empty() {
            return;
        }

        if self.is_empty() {
            let first = self.intervals.get_mut(0).unwrap();
            first.from_id = range_to_insert.from_id;
            first.to_id = range_to_insert.to_id;
            return;
        }

        let (from_index, to_index) = IndexToInsertRange::new(&self.intervals, &range_to_insert);

        match from_index {
            IndexToInsertRange::First => match to_index {
                IndexToInsertRange::Exact(to_index) => {
                    self.insert_with_override_to_right(0, to_index, range_to_insert);
                }
                IndexToInsertRange::First => {
                    let first_element = self.intervals.first_mut().unwrap();

                    if range_to_insert.to_id + 1 == first_element.from_id {
                        first_element.from_id = range_to_insert.from_id;
                        return;
                    }

                    self.intervals.insert(0, range_to_insert);
                }
                IndexToInsertRange::Last => {
                    self.intervals = vec![range_to_insert];
                }
                IndexToInsertRange::Between {
                    left_index,
                    right_index: _,
                } => {
                    self.insert_with_full_cover(0, left_index, range_to_insert);
                }
            },
            IndexToInsertRange::Exact(from_index) => match to_index {
                IndexToInsertRange::Exact(to_index) => {
                    self.insert_with_override_left_and_right(from_index, to_index);
                }
                IndexToInsertRange::First => {
                    panic!("Position between some interval and first element is not possible");
                }
                IndexToInsertRange::Last => {
                    self.insert_with_override_to_left(
                        from_index,
                        self.intervals.len() - 1,
                        range_to_insert,
                    );
                }
                IndexToInsertRange::Between {
                    left_index,
                    right_index: _,
                } => {
                    self.insert_with_override_to_left(from_index, left_index, range_to_insert);
                }
            },

            IndexToInsertRange::Last => match to_index {
                IndexToInsertRange::Exact(_) => {
                    panic!("Index can not be between Last and other element");
                }
                IndexToInsertRange::First => {
                    panic!("Index can not be between Last and First elements");
                }
                IndexToInsertRange::Last => {
                    let last = self.intervals.last_mut().unwrap();

                    if last.to_id + 1 == range_to_insert.from_id {
                        last.to_id = range_to_insert.to_id;
                        return;
                    }

                    self.intervals.push(range_to_insert);
                    return;
                }
                IndexToInsertRange::Between {
                    left_index: _,
                    right_index: _,
                } => {
                    panic!("Index can not be between Last and Between elements");
                }
            },
            IndexToInsertRange::Between {
                left_index: _,
                right_index,
            } => match to_index {
                IndexToInsertRange::Exact(to_index) => {
                    self.insert_with_override_to_right(right_index, to_index, range_to_insert);
                }
                IndexToInsertRange::First => {
                    panic!("Can not be between elements and first element")
                }
                IndexToInsertRange::Last => {
                    self.insert_with_full_cover(
                        right_index,
                        self.intervals.len() - 1,
                        range_to_insert,
                    );
                }
                IndexToInsertRange::Between {
                    left_index: to_left_index,
                    right_index: _,
                } => {
                    self.insert_with_full_cover(right_index, to_left_index, range_to_insert);
                }
            },
        }
    }

    fn insert_with_override_to_right(
        &mut self,
        from_index: usize,
        to_index: usize,
        range_to_insert: QueueIndexRange,
    ) {
        let to_id = self.intervals.get(to_index).unwrap().to_id;

        for _ in from_index..to_index {
            self.intervals.remove(from_index + 1);
        }
        if self.intervals.len() == 0 {
            panic!("Somehow intervals got empty");
        }

        let first = self.intervals.get_mut(from_index).unwrap();
        first.from_id = range_to_insert.from_id;
        first.to_id = to_id;
    }

    fn insert_with_override_to_left(
        &mut self,
        from_index: usize,
        to_index: usize,
        range_to_insert: QueueIndexRange,
    ) {
        for _ in from_index..to_index {
            self.intervals.remove(from_index + 1);
        }
        if self.intervals.len() == 0 {
            panic!("Somehow intervals got empty");
        }

        let first = self.intervals.get_mut(from_index).unwrap();
        first.to_id = range_to_insert.to_id;
    }

    fn insert_with_full_cover(
        &mut self,
        from_index: usize,
        to_index: usize,
        range_to_insert: QueueIndexRange,
    ) {
        for _ in from_index..to_index {
            self.intervals.remove(from_index + 1);
        }
        if self.intervals.len() == 0 {
            panic!("Somehow intervals got empty");
        }

        let first = self.intervals.get_mut(from_index).unwrap();
        first.from_id = range_to_insert.from_id;
        first.to_id = range_to_insert.to_id;
    }

    fn insert_with_override_left_and_right(&mut self, from_index: usize, to_index: usize) {
        let to_id = self.intervals.get(to_index).unwrap().to_id;

        for _ in from_index..to_index {
            self.remove_interval(from_index + 1);
        }

        let first = self.intervals.get_mut(from_index).unwrap();
        first.to_id = to_id;
    }

    pub fn dequeue(&mut self) -> Option<i64> {
        let (result, is_empty) = {
            let itm = self.intervals.get_mut(0).unwrap();
            if itm.is_empty() {
                return None;
            }

            let result = itm.from_id;
            itm.from_id += 1;

            (result, itm.is_empty())
        };

        if is_empty {
            self.remove_interval(0);
        }

        Some(result)
    }

    pub fn peek(&self) -> Option<i64> {
        let result = self.intervals.get(0).unwrap();

        if result.is_empty() {
            return None;
        }

        Some(result.from_id)
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        if self.is_empty() {
            return vec![];
        }

        self.intervals.clone()
    }

    // Returns non - only if we did not put any messages into the queue never

    pub fn get_min_id(&self) -> Option<i64> {
        let first = self.intervals.get(0).unwrap();

        if first.is_empty() {
            return None;
        }

        Some(first.from_id)
    }

    pub fn get_max_id(&self) -> Option<i64> {
        let last = self.intervals.get(self.intervals.len() - 1).unwrap();
        if last.is_empty() {
            return None;
        }

        Some(last.to_id)
    }

    pub fn has_message(&self, id: i64) -> bool {
        for interval in &self.intervals {
            if interval.is_in_my_interval(id) {
                return true;
            }
        }
        false
    }

    pub fn queue_size(&self) -> usize {
        let mut result = 0;

        for interval in &self.intervals {
            result += interval.len()
        }

        result as usize
    }

    pub fn iter(&self) -> QueueWithIntervalsIterator {
        QueueWithIntervalsIterator::new(self.clone())
    }
    pub fn len(&self) -> i64 {
        let mut result = 0;

        for interval in &self.intervals {
            result += interval.len();
        }

        result
    }
}

impl IntoIterator for QueueWithIntervals {
    type Item = i64;

    type IntoIter = QueueWithIntervalsIterator;

    fn into_iter(self) -> QueueWithIntervalsIterator {
        QueueWithIntervalsIterator::new(self.clone())
    }
}

impl<'s> IntoIterator for &'s QueueWithIntervals {
    type Item = i64;

    type IntoIter = QueueWithIntervalsIterator;

    fn into_iter(self) -> QueueWithIntervalsIterator {
        QueueWithIntervalsIterator::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let queue = QueueWithIntervals::new();

        assert_eq!(true, queue.get_min_id().is_none());
        assert_eq!(0, queue.queue_size());
    }

    #[test]
    fn test_enqueue_and_dequeue() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(5);
        queue.enqueue(6);

        assert_eq!(2, queue.queue_size());

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 5);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 6);

        assert_eq!(5, queue.dequeue().unwrap());
        assert_eq!(queue.intervals.get(0).unwrap().from_id, 6);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 6);
        assert_eq!(6, queue.dequeue().unwrap());
        assert!(queue.intervals.get(0).unwrap().is_empty());

        assert_eq!(true, queue.dequeue().is_none());
    }

    #[test]
    fn test_merge_intervals_at_the_end() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);

        assert_eq!(1, queue.intervals.len());

        queue.enqueue(203);

        assert_eq!(2, queue.intervals.len());

        queue.enqueue(202);
        assert_eq!(1, queue.intervals.len());
    }

    #[test]
    fn test_remove_first_element() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);

        queue.remove(200).unwrap();

        assert_eq!(1, queue.intervals.len());

        assert_eq!(201, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(204, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn test_remove_last_element() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);

        queue.remove(204).unwrap();

        assert_eq!(1, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(203, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn test_remove_middle_element_and_separate() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);

        queue.remove(202).unwrap();

        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(203, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(204, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_remove_middle_element_and_empty_it() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);
        queue.enqueue(205);
        queue.enqueue(206);

        queue.remove(202).unwrap();
        assert_eq!(2, queue.intervals.len());

        queue.remove(205).unwrap();
        assert_eq!(3, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(203, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(204, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(206, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(206, queue.intervals.get(2).unwrap().to_id);

        queue.remove(203).unwrap();
        queue.remove(204).unwrap();
        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(206, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(206, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_remove_element_and_empty_last_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);
        queue.enqueue(202);
        queue.enqueue(203);
        queue.enqueue(204);
        queue.enqueue(205);
        queue.enqueue(206);

        queue.remove(202).unwrap();
        assert_eq!(2, queue.intervals.len());

        queue.remove(205).unwrap();
        assert_eq!(3, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(203, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(204, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(206, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(206, queue.intervals.get(2).unwrap().to_id);

        queue.remove(206).unwrap();
        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(203, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(204, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn one_insert_one_remove_len_should_be_0() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(20466);

        let result = queue.dequeue();

        assert_eq!(20466, result.unwrap());
        assert_eq!(0, queue.queue_size());

        let result = queue.dequeue();

        assert_eq!(true, result.is_none());

        assert_eq!(0, queue.queue_size());
    }

    #[test]
    fn test_if_we_push_intervals_randomly_but_as_one_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(502);
        queue.enqueue(503);
        queue.enqueue(504);

        queue.enqueue(508);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(502, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(504, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(508, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(508, queue.intervals.get(1).unwrap().to_id);

        queue.enqueue(506);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(502, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(504, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(506, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(506, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(508, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(508, queue.intervals.get(2).unwrap().to_id);

        queue.enqueue(507);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(502, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(504, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(506, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(508, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_exact_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(502);
        queue.enqueue(503);
        queue.enqueue(504);

        queue.enqueue(506);
        queue.enqueue(507);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(502, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(504, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(506, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(507, queue.intervals.get(1).unwrap().to_id);

        queue.enqueue(505);

        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(502, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(507, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_case_to_empty_list() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(15, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        let new_interval = QueueIndexRange::restore(20, 25);

        // Doing action
        queue.enqueue_range(new_interval);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(15, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(20, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list_with_merge() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        let new_interval = QueueIndexRange::restore(16, 25);

        // Doing action
        queue.enqueue_range(new_interval);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(15, 20));

        let range_to_insert = QueueIndexRange::restore(5, 10);

        // Doing action
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(10, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(15, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_with_merge() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(15, 20));

        let range_to_insert = QueueIndexRange::restore(5, 14);

        // Doing action
        queue.enqueue_range(range_to_insert);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(20, 25));
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(15, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(20, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(1).unwrap().to_id);

        // Doing action
        let range_to_insert = QueueIndexRange::restore(5, 12);

        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(15, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(20, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_and_second_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));
        queue.enqueue_range(QueueIndexRange::restore(90, 100));

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);

        assert_eq!(90, queue.intervals.get(4).unwrap().from_id);
        assert_eq!(100, queue.intervals.get(4).unwrap().to_id);

        // Doing action
        let range_to_insert = QueueIndexRange::restore(35, 75);

        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(90, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(100, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn test_initializing_multiple_intervals() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn test_initializing_multiple_intervals_mixed_order() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_covering_one_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        // Executing data
        queue.enqueue_range(QueueIndexRange::restore(25, 45));

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(25, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(45, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_first_covering_first_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(5, 25);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_first_covering_first_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(5, 45);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(45, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_covering_second_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(25, 45);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(25, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(45, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_covering_second_and_third_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(25, 65);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(25, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(65, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_covering_last_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(65, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(65, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_covering_last_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(45, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(45, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_covering_everything() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(5, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_covering_between_and_exact_single_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(25, 35);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(25, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_covering_between_and_exact_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(25, 55);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(25, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_covering_exact_and_between_single_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(35, 45);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(45, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_covering_exact_and_between_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(35, 65);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(65, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_covering_exact_and_to_last() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(35, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(1).unwrap().to_id);
    }

    /*
    #[test]
    fn enqueue_range_in_the_middle_with_cover_several_elements_touching_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 35));

        queue.enqueue_range(QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(25, 43));

        assert_eq!(3, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(20, queue.get_interval(0).unwrap().to_id);

        assert_eq!(25, queue.get_interval(1).unwrap().from_id);
        assert_eq!(45, queue.get_interval(1).unwrap().to_id);

        assert_eq!(100, queue.get_interval(2).unwrap().from_id);
        assert_eq!(105, queue.get_interval(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover_several_elements_touching_left_and_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 35));

        queue.enqueue_range(QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(21, 43));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(45, queue.get_interval(0).unwrap().to_id);

        assert_eq!(100, queue.get_interval(1).unwrap().from_id);
        assert_eq!(105, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_which_covers_everything() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 35));

        queue.enqueue_range(QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(1, 200));

        assert_eq!(1, queue.get_intervals_amount());

        assert_eq!(1, queue.get_interval(0).unwrap().from_id);
        assert_eq!(200, queue.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn test_clean_several_intervals() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(2);
        assert_eq!(1, queue.queue_size());

        queue.clean();

        assert_eq!(0, queue.queue_size());
    }

    #[test]
    fn test_split_beyond_left() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(100);
        queue.enqueue(101);
        queue.enqueue(102);

        let split = queue.split(99);

        let left_q = split.0.unwrap();
        assert_eq!(100, left_q.get_min_id().unwrap());
        assert_eq!(102, left_q.get_max_id().unwrap());

        assert_eq!(true, split.1.is_none());
    }

    #[test]
    fn test_split_beyond_right() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(100);
        queue.enqueue(101);
        queue.enqueue(102);

        let split = queue.split(103);

        let left_q = split.0.unwrap();
        assert_eq!(100, left_q.get_min_id().unwrap());
        assert_eq!(102, left_q.get_max_id().unwrap());

        assert_eq!(true, split.1.is_none());
    }

    #[test]
    fn test_split_at_number_exist() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(100);
        queue.enqueue(101);
        queue.enqueue(102);

        let split = queue.split(101);

        let left_q = split.0.unwrap();
        assert_eq!(100, left_q.get_min_id().unwrap());
        assert_eq!(101, left_q.get_max_id().unwrap());

        let right_q = split.1.unwrap();
        assert_eq!(102, right_q.get_min_id().unwrap());
        assert_eq!(102, right_q.get_max_id().unwrap());
    }

    #[test]
    fn test_split_at_number_exist_but_we_have_two_intervals() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(100);
        queue.enqueue(101);
        queue.enqueue(103);
        queue.enqueue(104);

        let split = queue.split(103);

        let left_q = split.0.unwrap();
        assert_eq!(100, left_q.get_min_id().unwrap());
        assert_eq!(103, left_q.get_max_id().unwrap());
        assert_eq!(2, left_q.get_intervals_amount());

        let right_q = split.1.unwrap();
        assert_eq!(104, right_q.get_min_id().unwrap());
        assert_eq!(104, right_q.get_max_id().unwrap());
        assert_eq!(1, right_q.get_intervals_amount());
    }

    #[test]
    fn test_compact() {
        let queue = QueueWithIntervals::restore(vec![
            QueueIndexRange {
                from_id: 242375,
                to_id: 253853,
            },
            QueueIndexRange {
                from_id: 109315,
                to_id: 109315,
            },
            QueueIndexRange {
                from_id: 105846,
                to_id: 105850,
            },
            QueueIndexRange {
                from_id: 857,
                to_id: 857,
            },
            QueueIndexRange {
                from_id: 856,
                to_id: 856,
            },
            QueueIndexRange {
                from_id: 855,
                to_id: 855,
            },
            QueueIndexRange {
                from_id: 854,
                to_id: 854,
            },
            QueueIndexRange {
                from_id: 853,
                to_id: 853,
            },
            QueueIndexRange {
                from_id: 852,
                to_id: 852,
            },
            QueueIndexRange {
                from_id: 850,
                to_id: 851,
            },
        ]);

        println!("{:?}", queue.get_snapshot())
    }
     */
}
