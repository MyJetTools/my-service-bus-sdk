use crate::queue_with_intervals::queue_index_range::{QueueIndexRange, RemoveResult};

use super::{iterator::QueueWithIntervalsIterator, queue_index_range::QueueIndexRangeCompare};

const MIN_CAPACITY: usize = 32;
#[derive(Debug, Clone)]
pub enum QueueWithIntervalsError {
    MessagesNotFound,
    QueueIsEmpty,
}

#[derive(Debug, Clone)]
pub struct QueueWithIntervals {
    pub intervals: Vec<QueueIndexRange>,
}

impl QueueWithIntervals {
    pub fn new() -> QueueWithIntervals {
        Self {
            intervals: Vec::new(),
        }
    }

    pub fn restore(intervals: Vec<QueueIndexRange>) -> Self {
        return Self { intervals };
    }

    pub fn from_single_interval(from_id: i64, to_id: i64) -> Self {
        let mut result = Self {
            intervals: Vec::new(),
        };

        result.intervals.push(QueueIndexRange { from_id, to_id });

        result
    }

    fn remove_queue_index_range(&mut self, index: usize) -> QueueIndexRange {
        let result = self.intervals.remove(index);
        if self.intervals.len() < MIN_CAPACITY {
            self.intervals.shrink_to(MIN_CAPACITY);
        }
        result
    }

    pub fn reset(&mut self, intervals: Vec<QueueIndexRange>) {
        self.intervals.clear();
        self.intervals.shrink_to(intervals.len());
        self.intervals.extend(intervals)
    }

    pub fn clean(&mut self) {
        if let Some(first_interval) = self.intervals.get_mut(0) {
            first_interval.from_id = 0;
            first_interval.to_id = -1;
        }

        while self.intervals.len() > 1 {
            self.remove_queue_index_range(self.intervals.len() - 1);
        }
    }

    pub fn remove(&mut self, id: i64) -> Result<(), QueueWithIntervalsError> {
        if self.intervals.len() == 0 {
            return Err(QueueWithIntervalsError::QueueIsEmpty);
        }

        for index in 0..self.intervals.len() {
            let item = self.intervals.get_mut(index).unwrap();

            if item.is_my_interval_to_remove(id) {
                let remove_result = item.remove(id);

                match remove_result {
                    RemoveResult::NoUpdate => {}
                    RemoveResult::InsertNew(new_item) => {
                        self.intervals.insert(index + 1, new_item);
                    }
                    RemoveResult::RemoveItem => {
                        self.remove_queue_index_range(index);
                    }
                }

                return Ok(());
            }
        }

        return Err(QueueWithIntervalsError::MessagesNotFound);
    }

    pub fn enqueue(&mut self, message_id: i64) {
        match self.intervals.get_mut(0) {
            Some(el) => {
                if el.is_empty() {
                    el.from_id = message_id;
                    el.to_id = message_id;
                    return;
                }
            }
            None => {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.intervals.push(item);
                return;
            }
        }

        let mut found_index = None;

        for index in 0..self.intervals.len() {
            let el = self.intervals.get_mut(index).unwrap();

            if el.try_join(message_id) {
                found_index = Some(index);
                break;
            }

            if message_id < el.from_id - 1 {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.intervals.insert(index, item);
                found_index = Some(index);
                break;
            }
        }

        match found_index {
            Some(index_we_handled) => {
                if index_we_handled > 0 {
                    let current_el = self.intervals.get(index_we_handled).unwrap().clone();
                    let before_el = self.intervals.get_mut(index_we_handled - 1).unwrap();
                    if before_el.try_join_with_the_next_one(current_el) {
                        self.remove_queue_index_range(index_we_handled);
                    }
                }

                if index_we_handled < self.intervals.len() - 1 {
                    let after_el = self.intervals.get(index_we_handled + 1).unwrap().clone();

                    let current_el = self.intervals.get_mut(index_we_handled).unwrap();
                    if current_el.try_join_with_the_next_one(after_el) {
                        self.remove_queue_index_range(index_we_handled + 1);
                    }
                }
            }
            None => {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.intervals.push(item);
                return;
            }
        }
    }

    fn get_indexes_it_covers(&self, range_to_insert: &QueueIndexRange) -> Option<Vec<usize>> {
        let mut result = Vec::new();

        for index in 0..self.intervals.len() {
            let el = self.intervals.get(index).unwrap();

            if range_to_insert.from_id <= el.from_id && range_to_insert.to_id >= el.to_id {
                result.push(index);
            }
        }

        if result.len() == 0 {
            return None;
        }

        Some(result)
    }

    fn compact_it(&mut self) {
        let mut index = 0;
        while index < self.intervals.len() - 1 {
            let el_to_id = self.intervals.get(index).unwrap().to_id;
            let next = self.intervals.get(index + 1).unwrap().clone();

            if next.can_be_joined_to_interval_from_the_left(el_to_id) {
                let removed = self.remove_queue_index_range(index + 1);
                self.intervals.get_mut(index).unwrap().to_id = removed.to_id;
                continue;
            }

            index += 1;
        }
    }

    pub fn merge_with(&mut self, other_queue: &QueueWithIntervals) {
        for range in &other_queue.intervals {
            self.enqueue_range(range);
        }
    }

    pub fn enqueue_range(&mut self, range_to_insert: &QueueIndexRange) {
        let first_el_result = self.intervals.get_mut(0);

        match first_el_result {
            Some(first_el) => {
                if first_el.is_empty() {
                    first_el.from_id = range_to_insert.from_id;
                    first_el.to_id = range_to_insert.to_id;
                    return;
                }
            }

            None => {
                self.intervals.push(range_to_insert.clone());
                return;
            }
        }

        let cover_indexes = self.get_indexes_it_covers(range_to_insert);

        if let Some(mut cover_indexes) = cover_indexes {
            let first_index = cover_indexes[0];
            let mut from_id = self.intervals[first_index].from_id;

            if range_to_insert.from_id < from_id {
                from_id = range_to_insert.from_id;
            }

            let last = cover_indexes.last().unwrap();

            let mut to_id = self.intervals[*last].to_id;

            if range_to_insert.to_id > to_id {
                to_id = range_to_insert.to_id;
            }

            while cover_indexes.len() > 1 {
                self.remove_queue_index_range(cover_indexes.len() - 1);
                cover_indexes.remove(0);
            }

            let el = self.intervals.get_mut(first_index).unwrap();
            el.from_id = from_id;
            el.to_id = to_id;

            self.compact_it();
        }

        let mut from_index = None;

        for index in 0..self.intervals.len() {
            let current_range = self.intervals.get(index).unwrap().clone();

            if current_range.from_id <= range_to_insert.from_id
                && current_range.to_id >= range_to_insert.to_id
            {
                return;
            }

            if current_range.can_be_joined_to_interval_from_the_left(range_to_insert.to_id) {
                self.intervals.get_mut(index).unwrap().from_id = range_to_insert.from_id;
                return;
            }

            if range_to_insert.to_id < current_range.from_id - 1 {
                self.intervals.insert(index, range_to_insert.clone());
                return;
            }

            if current_range.can_be_joined_to_interval_from_the_right(range_to_insert.from_id) {
                if index == self.intervals.len() - 1 {
                    self.intervals.get_mut(index).unwrap().to_id = range_to_insert.to_id;
                    return;
                }

                let next_range = self.intervals.get_mut(index + 1).unwrap();

                if range_to_insert.to_id < next_range.from_id - 1 {
                    self.intervals.get_mut(index).unwrap().to_id = range_to_insert.to_id;
                    return;
                }

                from_index = Some(index);
                break;
            }
        }

        if from_index.is_none() {
            self.intervals.push(range_to_insert.clone());
            return;
        }

        let from_index = from_index.unwrap();
        while from_index < self.intervals.len() - 1 {
            let next_range = self.remove_queue_index_range(from_index + 1);

            if next_range.can_be_joined_to_interval_from_the_left(range_to_insert.to_id) {
                self.intervals.get_mut(from_index).unwrap().to_id = next_range.to_id;
                return;
            }
        }

        self.intervals.push(range_to_insert.clone());
    }

    pub fn dequeue(&mut self) -> Option<i64> {
        let first_interval = self.intervals.get_mut(0)?;

        let result = first_interval.dequeue();

        if first_interval.is_empty() && self.intervals.len() > 1 {
            self.remove_queue_index_range(0);
        }

        result
    }

    pub fn peek(&self) -> Option<i64> {
        let first_interval = self.intervals.get(0)?;

        first_interval.peek()
    }

    pub fn len(&self) -> i64 {
        let mut result = 0i64;

        for i in 0..self.intervals.len() {
            result = result + self.intervals[i].len();
        }

        result
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        self.intervals.clone()
    }

    // Returns non - only if we did not put any messages into the queue never
    pub fn get_min_id(&self) -> Option<i64> {
        let result = self.intervals.get(0)?;

        let result = result.from_id;
        if result < 0 {
            return None;
        }

        Some(result)
    }

    pub fn get_max_id(&self) -> Option<i64> {
        if self.len() == 0 {
            return None;
        }

        let result = self.intervals.get(self.intervals.len() - 1)?;

        Some(result.to_id)
    }

    pub fn has_message(&self, id: i64) -> bool {
        for item in &self.intervals {
            if let Some(range) = item.compare_with(id) {
                if let QueueIndexRangeCompare::Inside = range {
                    return true;
                }
            }
        }

        return false;
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
        assert_eq!(0, queue.len());
    }

    #[test]
    fn test_enqueue_and_dequeue() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(5);
        queue.enqueue(6);

        assert_eq!(2, queue.len());

        assert_eq!(5, queue.dequeue().unwrap());
        assert_eq!(6, queue.dequeue().unwrap());
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

        assert_eq!(201, queue.intervals[0].from_id);
        assert_eq!(204, queue.intervals[0].to_id);
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

        println!("Len: {}", queue.intervals.len());

        assert_eq!(1, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(203, queue.intervals[0].to_id);
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

        println!("Len: {}", queue.intervals.len());

        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);
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

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);

        assert_eq!(206, queue.intervals[2].from_id);
        assert_eq!(206, queue.intervals[2].to_id);

        queue.remove(203).unwrap();
        queue.remove(204).unwrap();
        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(206, queue.intervals[1].from_id);
        assert_eq!(206, queue.intervals[1].to_id);
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

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);

        assert_eq!(206, queue.intervals[2].from_id);
        assert_eq!(206, queue.intervals[2].to_id);

        queue.remove(206).unwrap();
        assert_eq!(2, queue.intervals.len());

        assert_eq!(200, queue.intervals[0].from_id);
        assert_eq!(201, queue.intervals[0].to_id);

        assert_eq!(203, queue.intervals[1].from_id);
        assert_eq!(204, queue.intervals[1].to_id);
    }

    #[test]
    fn one_insert_one_remove_len_should_be_0() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(20466);

        let result = queue.dequeue();

        assert_eq!(20466, result.unwrap());
        assert_eq!(0, queue.len());

        let result = queue.dequeue();

        assert_eq!(true, result.is_none());

        assert_eq!(0, queue.len());
    }

    #[test]
    fn test_if_we_push_intervals_randomly_but_as_one_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(502);
        queue.enqueue(503);
        queue.enqueue(504);

        queue.enqueue(508);
        assert_eq!(queue.intervals.len(), 2);

        queue.enqueue(506);
        assert_eq!(queue.intervals.len(), 3);
        queue.enqueue(507);
        assert_eq!(queue.intervals.len(), 2);
    }

    #[test]
    fn enqueue_range_case_to_empty_list() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(15, queue.intervals[0].to_id);
    }

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(20, 25));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(15, queue.intervals[0].to_id);

        assert_eq!(20, queue.intervals[1].from_id);
        assert_eq!(25, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(15, 20));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(5, 10));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(5, queue.intervals[0].from_id);
        assert_eq!(10, queue.intervals[0].to_id);

        assert_eq!(15, queue.intervals[1].from_id);
        assert_eq!(20, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_one() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(15, 20));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(5, 14));

        assert_eq!(1, queue.intervals.len());

        assert_eq!(5, queue.intervals[0].from_id);
        assert_eq!(20, queue.intervals[0].to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_one_case_2() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(20, 25));
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(5, 12));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(5, queue.intervals[0].from_id);
        assert_eq!(15, queue.intervals[0].to_id);

        assert_eq!(20, queue.intervals[1].from_id);
        assert_eq!(25, queue.intervals[1].to_id);
    }
    #[test]
    fn enqueue_range_in_the_middle() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(200, 205));
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(300, 305));
        queue.enqueue_range(&QueueIndexRange::restore(250, 255));

        assert_eq!(4, queue.intervals.len());

        assert_eq!(100, queue.intervals[0].from_id);
        assert_eq!(105, queue.intervals[0].to_id);

        assert_eq!(200, queue.intervals[1].from_id);
        assert_eq!(205, queue.intervals[1].to_id);

        assert_eq!(250, queue.intervals[2].from_id);
        assert_eq!(255, queue.intervals[2].to_id);

        assert_eq!(300, queue.intervals[3].from_id);
        assert_eq!(305, queue.intervals[3].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_left() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(16, 20));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(20, queue.intervals[0].to_id);

        assert_eq!(100, queue.intervals[1].from_id);
        assert_eq!(105, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_left_including() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(15, 20));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(20, queue.intervals[0].to_id);

        assert_eq!(100, queue.intervals[1].from_id);
        assert_eq!(105, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(90, 99));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(15, queue.intervals[0].to_id);

        assert_eq!(90, queue.intervals[1].from_id);
        assert_eq!(105, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_right_including() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(90, 100));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(15, queue.intervals[0].to_id);

        assert_eq!(90, queue.intervals[1].from_id);
        assert_eq!(105, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_left_and_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(16, 99));

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(105, queue.intervals[0].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 20));
        queue.enqueue_range(&QueueIndexRange::restore(30, 35));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(25, 40));

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(20, queue.intervals[0].to_id);

        assert_eq!(25, queue.intervals[1].from_id);
        assert_eq!(40, queue.intervals[1].to_id);

        assert_eq!(100, queue.intervals[2].from_id);
        assert_eq!(105, queue.intervals[2].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover_several_elements() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 20));
        queue.enqueue_range(&QueueIndexRange::restore(30, 35));

        queue.enqueue_range(&QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(25, 70));

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(20, queue.intervals[0].to_id);

        assert_eq!(25, queue.intervals[1].from_id);
        assert_eq!(70, queue.intervals[1].to_id);

        assert_eq!(100, queue.intervals[2].from_id);
        assert_eq!(105, queue.intervals[2].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover_several_elements_touching_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 20));
        queue.enqueue_range(&QueueIndexRange::restore(30, 35));

        queue.enqueue_range(&QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(25, 43));

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(20, queue.intervals[0].to_id);

        assert_eq!(25, queue.intervals[1].from_id);
        assert_eq!(45, queue.intervals[1].to_id);

        assert_eq!(100, queue.intervals[2].from_id);
        assert_eq!(105, queue.intervals[2].to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover_several_elements_touching_left_and_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 20));
        queue.enqueue_range(&QueueIndexRange::restore(30, 35));

        queue.enqueue_range(&QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(21, 43));

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals[0].from_id);
        assert_eq!(45, queue.intervals[0].to_id);

        assert_eq!(100, queue.intervals[1].from_id);
        assert_eq!(105, queue.intervals[1].to_id);
    }

    #[test]
    fn enqueue_range_which_covers_everything() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(&QueueIndexRange::restore(100, 105));
        queue.enqueue_range(&QueueIndexRange::restore(10, 20));
        queue.enqueue_range(&QueueIndexRange::restore(30, 35));

        queue.enqueue_range(&QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(&QueueIndexRange::restore(1, 200));

        assert_eq!(1, queue.intervals.len());

        assert_eq!(1, queue.intervals[0].from_id);
        assert_eq!(200, queue.intervals[0].to_id);
    }

    #[test]
    fn test_clean_several_intervals() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue(2);
        assert_eq!(1, queue.len());

        queue.clean();

        assert_eq!(0, queue.len());
    }
}
