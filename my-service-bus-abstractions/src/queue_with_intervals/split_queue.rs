use super::{QueueIndexRange, QueueWithIntervals};

impl QueueWithIntervals {
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

        for interval in &self.intervals {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(2, left_q.intervals.len());

        let right_q = split.1.unwrap();
        assert_eq!(104, right_q.get_min_id().unwrap());
        assert_eq!(104, right_q.get_max_id().unwrap());
        assert_eq!(1, right_q.intervals.len());
    }
}
