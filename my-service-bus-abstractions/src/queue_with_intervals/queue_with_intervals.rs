use crate::queue_with_intervals::queue_index_range::{QueueIndexRange, RemoveResult};

use super::{
    iterator::QueueWithIntervalsIterator, queue_index_range::QueueIndexRangeCompare,
    QueueWithIntervalsInner,
};

#[derive(Debug, Clone)]
pub enum QueueWithIntervalsError {
    MessagesNotFound,
    QueueIsEmpty,
}

#[derive(Debug, Clone)]
pub struct QueueWithIntervals {
    inner: QueueWithIntervalsInner,
}

impl QueueWithIntervals {
    pub fn new() -> QueueWithIntervals {
        Self {
            inner: QueueWithIntervalsInner::new(0),
        }
    }

    pub fn restore(intervals: Vec<QueueIndexRange>) -> Self {
        Self {
            inner: QueueWithIntervalsInner::restore(intervals),
        }
    }

    pub fn from_single_interval(from_id: i64, to_id: i64) -> Self {
        Self {
            inner: QueueWithIntervalsInner::from_single_interval(from_id, to_id),
        }
    }

    pub fn reset(&mut self, intervals: Vec<QueueIndexRange>) {
        self.inner.reset(intervals);
    }

    pub fn clean(&mut self) {
        self.inner.clean();
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn remove(&mut self, id: i64) -> Result<(), QueueWithIntervalsError> {
        if self.is_empty() {
            return Err(QueueWithIntervalsError::QueueIsEmpty);
        }

        for index in 0..self.inner.intervals_amount() {
            if let Some(item) = self.inner.get_mut(index) {
                if item.is_in_my_interval(id) {
                    match item.remove(id) {
                        RemoveResult::NoUpdate => return Ok(()),
                        RemoveResult::InsertNew(new_item) => {
                            self.inner.insert(index + 1, new_item);
                            return Ok(());
                        }
                        RemoveResult::RemoveItem => {
                            self.inner.remove(index);
                            return Ok(());
                        }
                    }
                }
            }
        }

        return Err(QueueWithIntervalsError::MessagesNotFound);
    }

    fn merge_items_if_possible(&mut self, index: usize) {
        if index == 0 {
            let (current, next) = self.inner.get_two(index);
            let mut merged = None;
            if let Some(current) = current {
                if let Some(next) = next {
                    merged = current.try_to_merge_with_next_item(next);
                }
            }

            if let Some(merged) = merged {
                self.inner.update(index, merged);
                self.inner.remove(index + 1);
            }

            return;
        }

        let (prev, current) = self.inner.get_two(index - 1);
        let mut merged = None;
        if let Some(prev) = prev {
            if let Some(current) = current {
                merged = prev.try_to_merge_with_next_item(current);
            }
        }

        if let Some(merged) = merged {
            self.inner.update(index - 1, merged);
            self.inner.remove(index);
        }

        let (current, next) = self.inner.get_two(index);
        let mut merged = None;
        if let Some(current) = current {
            if let Some(next) = next {
                merged = current.try_to_merge_with_next_item(next);
            }
        }

        if let Some(merged) = merged {
            self.inner.update(index, merged);
            self.inner.remove(index + 1);
        }
    }

    pub fn enqueue(&mut self, message_id: i64) {
        let mut found_index = None;

        for index in 0..self.inner.intervals_amount() {
            let el = self.inner.get_mut(index).unwrap();

            if el.try_join(message_id) {
                found_index = Some(index);
                break;
            }

            if message_id < el.from_id - 1 {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.inner.insert(index, item);
                found_index = Some(index);
                break;
            }
        }

        match found_index {
            Some(index_we_handled) => self.merge_items_if_possible(index_we_handled),
            None => {
                let item = QueueIndexRange::new_with_single_value(message_id);
                self.inner.push(item);
            }
        }
    }

    fn get_indexes_it_covers(&self, range_to_insert: &QueueIndexRange) -> Vec<usize> {
        let mut result = Vec::new();

        for index in 0..self.inner.intervals_amount() {
            let el = self.inner.get(index).unwrap();

            if range_to_insert.from_id <= el.from_id && range_to_insert.to_id >= el.to_id {
                result.push(index);
            }
        }

        result
    }

    fn compact_it(&mut self) {
        let mut index = 0;
        while index < self.inner.intervals_amount() - 1 {
            let el_to_id = self.inner.get(index).unwrap().to_id;
            let next = self.inner.get(index + 1).unwrap().clone();

            if next.can_be_joined_to_interval_from_the_left(el_to_id) {
                let removed = self.inner.remove(index + 1);
                if let Some(removed) = removed {
                    self.inner.get_mut(index).unwrap().to_id = removed.to_id;
                }

                continue;
            }

            index += 1;
        }
    }

    pub fn merge_with(&mut self, other_queue: &QueueWithIntervals) {
        for range in other_queue.inner.iter() {
            self.enqueue_range(range);
        }
    }

    pub fn enqueue_range(&mut self, range_to_insert: QueueIndexRange) {
        let first_el_result: Option<&mut QueueIndexRange> = self.inner.get_mut(0);

        if first_el_result.is_none() {
            self.inner.push(range_to_insert.clone());
            return;
        }

        /*

        match first_el_result {
            Some(first_el) => {
                if first_el.is_empty() {
                    first_el.from_id = range_to_insert.from_id;
                    first_el.to_id = range_to_insert.to_id;
                    return;
                }
            }

            None => {
                self.inner.push(range_to_insert.clone());
                return;
            }
        }
        */

        let mut cover_indexes = self.get_indexes_it_covers(&range_to_insert);

        if cover_indexes.len() > 0 {
            let first_index = cover_indexes[0];
            let mut from_id = self.inner.get(first_index).unwrap().from_id;

            if range_to_insert.from_id < from_id {
                from_id = range_to_insert.from_id;
            }

            let last = cover_indexes.last().unwrap();

            let mut to_id = self.inner.get(*last).unwrap().to_id;

            if range_to_insert.to_id > to_id {
                to_id = range_to_insert.to_id;
            }

            while cover_indexes.len() > 1 {
                self.inner.remove(cover_indexes.len() - 1);
                cover_indexes.remove(0);
            }

            let el = self.inner.get_mut(first_index).unwrap();
            el.from_id = from_id;
            el.to_id = to_id;

            self.compact_it();
        }

        let mut from_index = None;

        for index in 0..self.inner.intervals_amount() {
            let current_range = self.inner.get(index).unwrap().clone();

            if current_range.from_id <= range_to_insert.from_id
                && current_range.to_id >= range_to_insert.to_id
            {
                return;
            }

            if current_range.can_be_joined_to_interval_from_the_left(range_to_insert.to_id) {
                self.inner.get_mut(index).unwrap().from_id = range_to_insert.from_id;
                return;
            }

            if range_to_insert.to_id < current_range.from_id - 1 {
                self.inner.insert(index, range_to_insert.clone());
                return;
            }

            if current_range.can_be_joined_to_interval_from_the_right(range_to_insert.from_id) {
                if index == self.inner.intervals_amount() - 1 {
                    self.inner.get_mut(index).unwrap().to_id = range_to_insert.to_id;
                    return;
                }

                let next_range = self.inner.get_mut(index + 1).unwrap();

                if range_to_insert.to_id < next_range.from_id - 1 {
                    self.inner.get_mut(index).unwrap().to_id = range_to_insert.to_id;
                    return;
                }

                from_index = Some(index);
                break;
            }
        }

        if from_index.is_none() {
            self.inner.push(range_to_insert.clone());
            return;
        }

        let from_index = from_index.unwrap();
        while from_index < self.inner.intervals_amount() - 1 {
            let next_range = self.inner.remove(from_index + 1).unwrap();

            if next_range.can_be_joined_to_interval_from_the_left(range_to_insert.to_id) {
                self.inner.get_mut(from_index).unwrap().to_id = next_range.to_id;
                return;
            }
        }

        self.inner.push(range_to_insert.clone());
    }

    pub fn dequeue(&mut self) -> Option<i64> {
        let first_interval = self.inner.get_mut(0)?;

        let result = first_interval.dequeue();

        if first_interval.is_empty() && self.inner.intervals_amount() > 1 {
            self.inner.remove(0);
        }

        result
    }

    pub fn peek(&self) -> Option<i64> {
        let first_interval = self.inner.get(0)?;

        first_interval.peek()
    }

    pub fn get_snapshot(&self) -> Vec<QueueIndexRange> {
        self.inner.get_snapshot()
    }

    pub fn push_interval(&mut self, index_range: QueueIndexRange) {
        self.inner.push(index_range);
    }

    // Returns non - only if we did not put any messages into the queue never
    pub fn get_min_id(&self) -> Option<i64> {
        self.inner.get_min_id()
    }

    pub fn get_min_id_even_if_empty(&self) -> Option<i64> {
        self.inner.get_min_id_even_if_empty()
    }

    pub fn get_max_id(&self) -> Option<i64> {
        self.inner.get_max_id()
    }

    pub fn has_message(&self, id: i64) -> bool {
        self.inner.has_item(|item| {
            if let Some(range) = item.compare_with(id) {
                if let QueueIndexRangeCompare::Inside = range {
                    return true;
                }
            }

            false
        })
    }

    pub fn split(&self, id: i64) -> (Option<QueueWithIntervals>, Option<QueueWithIntervals>) {
        let min_id = self.get_min_id();

        if min_id.is_none() {
            return (None, None);
        }

        let min_id = min_id.unwrap();

        if id < min_id {
            return (Some(self.clone()), None);
        }

        let max_id = self.get_max_id();

        if max_id.is_none() {
            return (None, None);
        }

        let max_id = max_id.unwrap();

        if id > max_id {
            return (Some(self.clone()), None);
        }

        let mut doing_left = true;
        let mut left: Vec<QueueIndexRange> = Vec::new();
        let mut right: Vec<QueueIndexRange> = Vec::new();

        for interval in self.inner.iter() {
            if doing_left {
                if interval.from_id <= id && id < interval.to_id {
                    left.push(QueueIndexRange {
                        from_id: interval.from_id,
                        to_id: id,
                    });

                    doing_left = false;

                    if id + 1 <= interval.to_id {
                        right.push(QueueIndexRange {
                            from_id: id + 1,
                            to_id: interval.to_id,
                        });
                    }
                } else if interval.from_id < id && id == interval.to_id {
                    left.push(QueueIndexRange {
                        from_id: interval.from_id,
                        to_id: interval.to_id,
                    });

                    doing_left = false;
                } else {
                    left.push(interval.clone());
                }
            } else {
                right.push(interval.clone())
            }
        }

        (
            Some(QueueWithIntervals::restore(left)),
            Some(QueueWithIntervals::restore(right)),
        )
    }

    pub fn queue_size(&self) -> usize {
        self.inner.queue_size()
    }
    #[cfg(test)]
    pub fn get_intervals_amount(&self) -> usize {
        self.inner.intervals_amount()
    }

    pub fn get_interval(&self, index: usize) -> Option<&QueueIndexRange> {
        self.inner.get(index)
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

        assert_eq!(5, queue.dequeue().unwrap());
        assert_eq!(6, queue.dequeue().unwrap());
        assert_eq!(true, queue.dequeue().is_none());
    }

    #[test]
    fn test_merge_intervals_at_the_end() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue(200);
        queue.enqueue(201);

        assert_eq!(1, queue.get_intervals_amount());

        queue.enqueue(203);

        assert_eq!(2, queue.get_intervals_amount());

        queue.enqueue(202);
        assert_eq!(1, queue.get_intervals_amount());
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

        assert_eq!(1, queue.get_intervals_amount());

        assert_eq!(201, queue.get_interval(0).unwrap().from_id);
        assert_eq!(204, queue.get_interval(0).unwrap().to_id);
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

        println!("Len: {}", queue.get_intervals_amount());

        assert_eq!(1, queue.get_intervals_amount());

        assert_eq!(200, queue.get_interval(0).unwrap().from_id);
        assert_eq!(203, queue.get_interval(0).unwrap().to_id);
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

        println!("Len: {}", queue.get_intervals_amount());

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(200, queue.get_interval(0).unwrap().from_id);
        assert_eq!(201, queue.get_interval(0).unwrap().to_id);

        assert_eq!(203, queue.get_interval(1).unwrap().from_id);
        assert_eq!(204, queue.get_interval(1).unwrap().to_id);
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
        assert_eq!(2, queue.get_intervals_amount());

        queue.remove(205).unwrap();
        assert_eq!(3, queue.get_intervals_amount());

        assert_eq!(200, queue.get_interval(0).unwrap().from_id);
        assert_eq!(201, queue.get_interval(0).unwrap().to_id);

        assert_eq!(203, queue.get_interval(1).unwrap().from_id);
        assert_eq!(204, queue.get_interval(1).unwrap().to_id);

        assert_eq!(206, queue.get_interval(2).unwrap().from_id);
        assert_eq!(206, queue.get_interval(2).unwrap().to_id);

        queue.remove(203).unwrap();
        queue.remove(204).unwrap();
        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(200, queue.get_interval(0).unwrap().from_id);
        assert_eq!(201, queue.get_interval(0).unwrap().to_id);

        assert_eq!(206, queue.get_interval(1).unwrap().from_id);
        assert_eq!(206, queue.get_interval(1).unwrap().to_id);
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
        assert_eq!(2, queue.get_intervals_amount());

        queue.remove(205).unwrap();
        assert_eq!(3, queue.get_intervals_amount());

        assert_eq!(200, queue.get_interval(0).unwrap().from_id);
        assert_eq!(201, queue.get_interval(0).unwrap().to_id);

        assert_eq!(203, queue.get_interval(1).unwrap().from_id);
        assert_eq!(204, queue.get_interval(1).unwrap().to_id);

        assert_eq!(206, queue.get_interval(2).unwrap().from_id);
        assert_eq!(206, queue.get_interval(2).unwrap().to_id);

        queue.remove(206).unwrap();
        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(200, queue.get_interval(0).unwrap().from_id);
        assert_eq!(201, queue.get_interval(0).unwrap().to_id);

        assert_eq!(203, queue.get_interval(1).unwrap().from_id);
        assert_eq!(204, queue.get_interval(1).unwrap().to_id);
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
        assert_eq!(queue.get_intervals_amount(), 2);

        queue.enqueue(506);
        assert_eq!(queue.get_intervals_amount(), 3);
        queue.enqueue(507);
        assert_eq!(queue.get_intervals_amount(), 2);
    }

    #[test]
    fn enqueue_range_case_to_empty_list() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        assert_eq!(1, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(15, queue.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(20, 25));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(15, queue.get_interval(0).unwrap().to_id);

        assert_eq!(20, queue.get_interval(1).unwrap().from_id);
        assert_eq!(25, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(15, 20));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(5, 10));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(5, queue.get_interval(0).unwrap().from_id);
        assert_eq!(10, queue.get_interval(0).unwrap().to_id);

        assert_eq!(15, queue.get_interval(1).unwrap().from_id);
        assert_eq!(20, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_one() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(15, 20));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(5, 14));

        assert_eq!(1, queue.get_intervals_amount());

        assert_eq!(5, queue.get_interval(0).unwrap().from_id);
        assert_eq!(20, queue.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_one_case_2() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(20, 25));
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(5, 12));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(5, queue.get_interval(0).unwrap().from_id);
        assert_eq!(15, queue.get_interval(0).unwrap().to_id);

        assert_eq!(20, queue.get_interval(1).unwrap().from_id);
        assert_eq!(25, queue.get_interval(1).unwrap().to_id);
    }
    #[test]
    fn enqueue_range_in_the_middle() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(200, 205));
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(300, 305));
        queue.enqueue_range(QueueIndexRange::restore(250, 255));

        assert_eq!(4, queue.get_intervals_amount());

        assert_eq!(100, queue.get_interval(0).unwrap().from_id);
        assert_eq!(105, queue.get_interval(0).unwrap().to_id);

        assert_eq!(200, queue.get_interval(1).unwrap().from_id);
        assert_eq!(205, queue.get_interval(1).unwrap().to_id);

        assert_eq!(250, queue.get_interval(2).unwrap().from_id);
        assert_eq!(255, queue.get_interval(2).unwrap().to_id);

        assert_eq!(300, queue.get_interval(3).unwrap().from_id);
        assert_eq!(305, queue.get_interval(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_left() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 15));
        queue.enqueue_range(QueueIndexRange::restore(100, 105));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(16, 20));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(20, queue.get_interval(0).unwrap().to_id);

        assert_eq!(100, queue.get_interval(1).unwrap().from_id);
        assert_eq!(105, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_left_including() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(15, 20));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(20, queue.get_interval(0).unwrap().to_id);

        assert_eq!(100, queue.get_interval(1).unwrap().from_id);
        assert_eq!(105, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(90, 99));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(15, queue.get_interval(0).unwrap().to_id);

        assert_eq!(90, queue.get_interval(1).unwrap().from_id);
        assert_eq!(105, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_right_including() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(90, 100));

        assert_eq!(2, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(15, queue.get_interval(0).unwrap().to_id);

        assert_eq!(90, queue.get_interval(1).unwrap().from_id);
        assert_eq!(105, queue.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_stick_to_the_left_and_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 15));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(16, 99));

        assert_eq!(1, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(105, queue.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 35));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(25, 40));

        assert_eq!(3, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(20, queue.get_interval(0).unwrap().to_id);

        assert_eq!(25, queue.get_interval(1).unwrap().from_id);
        assert_eq!(40, queue.get_interval(1).unwrap().to_id);

        assert_eq!(100, queue.get_interval(2).unwrap().from_id);
        assert_eq!(105, queue.get_interval(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_in_the_middle_with_cover_several_elements() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(100, 105));
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 35));

        queue.enqueue_range(QueueIndexRange::restore(40, 45));

        // Doing action
        queue.enqueue_range(QueueIndexRange::restore(25, 70));

        assert_eq!(3, queue.get_intervals_amount());

        assert_eq!(10, queue.get_interval(0).unwrap().from_id);
        assert_eq!(20, queue.get_interval(0).unwrap().to_id);

        assert_eq!(25, queue.get_interval(1).unwrap().from_id);
        assert_eq!(70, queue.get_interval(1).unwrap().to_id);

        assert_eq!(100, queue.get_interval(2).unwrap().from_id);
        assert_eq!(105, queue.get_interval(2).unwrap().to_id);
    }

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
}
