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
    pub(crate) intervals: Vec<QueueIndexRange>,
}

impl Default for QueueWithIntervals {
    fn default() -> Self {
        Self::new()
    }
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

        let index = IndexToRemoveValue::new(&self.intervals, value);

        match index {
            IndexToRemoveValue::IncLeft(index) => {
                self.intervals.get_mut(index).unwrap().from_id += 1;
            }

            IndexToRemoveValue::DecRight(index) => {
                self.intervals.get_mut(index).unwrap().to_id -= 1;
            }
            IndexToRemoveValue::Split { index, left, right } => {
                self.intervals.insert(index + 1, right);
                let left_part = self.intervals.get_mut(index).unwrap();
                left_part.from_id = left.from_id;
                left_part.to_id = left.to_id;
            }
            IndexToRemoveValue::Remove(index) => {
                self.remove_interval(index);
            }
            IndexToRemoveValue::NoValue => return Err(QueueWithIntervalsError::MessagesNotFound),
        }

        Ok(())
    }

    pub(crate) fn remove_interval(&mut self, index: usize) {
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

    /*
       fn insert_with_override_left_and_right(&mut self, from_index: usize, to_index: usize) {
           let to_id = self.intervals.get(to_index).unwrap().to_id;

           for _ in from_index..to_index {
               self.remove_interval(from_index + 1);
           }

           let first = self.intervals.get_mut(from_index).unwrap();
           first.to_id = to_id;
       }
    */
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
    fn restore_sorts_and_handles_empty_input() {
        let queue = QueueWithIntervals::restore(vec![
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(25, 27),
        ]);

        assert_eq!(3, queue.intervals.len());
        assert_eq!(
            (10, 20),
            (queue.intervals[0].from_id, queue.intervals[0].to_id)
        );
        assert_eq!(
            (25, 27),
            (queue.intervals[1].from_id, queue.intervals[1].to_id)
        );
        assert_eq!(
            (30, 40),
            (queue.intervals[2].from_id, queue.intervals[2].to_id)
        );

        let empty_restored = QueueWithIntervals::restore(vec![]);
        assert!(empty_restored.is_empty());
        assert_eq!(1, empty_restored.intervals.len());
        assert!(empty_restored.intervals[0].is_empty());
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

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(203, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(206, queue.intervals.get(1).unwrap().to_id);

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

        queue.remove(206).unwrap();

        assert_eq!(1, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(201, queue.intervals.get(0).unwrap().to_id);

        queue.remove(201).unwrap();

        assert_eq!(1, queue.intervals.len());

        assert_eq!(200, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(200, queue.intervals.get(0).unwrap().to_id);

        queue.remove(200).unwrap();

        assert_eq!(1, queue.intervals.len());
        assert!(queue.intervals.get(0).unwrap().is_empty());
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
    fn peek_min_max_on_empty_and_after_operations() {
        let mut queue = QueueWithIntervals::new();

        assert_eq!(None, queue.peek());
        assert_eq!(None, queue.get_min_id());
        assert_eq!(None, queue.get_max_id());

        queue.enqueue(11);
        queue.enqueue(12);

        assert_eq!(Some(11), queue.peek());
        assert_eq!(Some(11), queue.get_min_id());
        assert_eq!(Some(12), queue.get_max_id());

        queue.dequeue();
        queue.dequeue();

        assert_eq!(None, queue.peek());
        assert_eq!(None, queue.get_min_id());
        assert_eq!(None, queue.get_max_id());
    }

    #[test]
    fn has_message_and_lengths_match() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(30);
        queue.enqueue(31);
        queue.enqueue(33);

        assert!(queue.has_message(30));
        assert!(!queue.has_message(29));
        assert!(!queue.has_message(32));
        assert_eq!(3, queue.queue_size());
        assert_eq!(3, queue.len());

        queue.remove(31).unwrap();

        assert_eq!(2, queue.queue_size());
        assert_eq!(2, queue.len());
        assert!(!queue.has_message(31));
    }

    #[test]
    fn enqueue_existing_value_does_not_change_state() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(5);
        queue.enqueue(6);
        queue.enqueue(6);

        assert_eq!(1, queue.intervals.len());
        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(6, queue.intervals.get(0).unwrap().to_id);
        assert_eq!(2, queue.queue_size());
    }

    #[test]
    fn merge_combines_and_merges_adjacent_intervals() {
        let mut base = QueueWithIntervals::new();
        base.enqueue(1);
        base.enqueue(2);
        base.enqueue(10);
        base.enqueue(11);
        base.enqueue(12);

        let mut other = QueueWithIntervals::new();
        other.enqueue(3);
        other.enqueue(4);
        other.enqueue(8);
        other.enqueue(9);
        other.enqueue(13);

        base.merge(other);

        assert_eq!(2, base.intervals.len());
        assert_eq!(1, base.intervals[0].from_id);
        assert_eq!(4, base.intervals[0].to_id);
        assert_eq!(8, base.intervals[1].from_id);
        assert_eq!(13, base.intervals[1].to_id);
    }

    #[test]
    fn reset_and_clean_behaviour() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(5);
        queue.enqueue(6);
        queue.enqueue(10);

        let last_to_id = queue.get_max_id().unwrap();
        queue.clean();

        assert_eq!(1, queue.intervals.len());
        let first = queue.intervals.first().unwrap();
        assert!(first.is_empty());
        assert_eq!(last_to_id, first.to_id);

        queue.reset(vec![]);
        assert_eq!(1, queue.intervals.len());
        assert!(queue.is_empty());
    }

    #[test]
    fn snapshot_is_copy() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(100);
        queue.enqueue(101);

        let mut snapshot = queue.get_snapshot();
        snapshot[0].from_id = 999;

        assert_eq!(Some(100), queue.get_min_id());
        assert_eq!(Some(101), queue.get_max_id());
    }

    #[test]
    fn dequeue_until_empty_then_none() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(42);

        assert_eq!(Some(42), queue.dequeue());
        assert_eq!(None, queue.dequeue());
        assert!(queue.is_empty());
    }

    #[test]
    fn remove_errors_are_returned() {
        let mut queue = QueueWithIntervals::new();
        assert!(matches!(
            queue.remove(5).unwrap_err(),
            QueueWithIntervalsError::QueueIsEmpty
        ));

        queue.enqueue(1);
        assert!(matches!(
            queue.remove(2).unwrap_err(),
            QueueWithIntervalsError::MessagesNotFound
        ));
    }

    #[test]
    fn borrowed_iterator_keeps_original_intact() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(7);
        queue.enqueue(8);
        queue.enqueue(9);

        let collected: Vec<i64> = (&queue).into_iter().collect();
        assert_eq!(vec![7, 8, 9], collected);
        assert_eq!(Some(7), queue.peek());
        assert_eq!(3, queue.queue_size());
    }

    #[test]
    fn iterator_spans_multiple_intervals() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(5);
        queue.enqueue(6);

        let collected: Vec<i64> = queue.iter().collect();
        assert_eq!(vec![1, 2, 5, 6], collected);

        assert_eq!(Some(1), queue.peek());
        assert_eq!(4, queue.queue_size());
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
        queue.enqueue_range(QueueIndexRange::restore(90, 100));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        assert_eq!(5, queue.intervals.len());

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
    }
}
