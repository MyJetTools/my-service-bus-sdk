use super::{IndexRange, QueueIndexRange, QueueWithIntervals};

impl QueueWithIntervals {
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

        let (from_index, to_index) = IndexRange::new(&self.intervals, &range_to_insert);

        match from_index {
            IndexRange::First => {
                self.enqueue_as_first_as_between(0, to_index, range_to_insert);
            }
            IndexRange::Exact(from_index) => {
                self.enqueue_as_left_as_exact(from_index, to_index, range_to_insert);
            }
            IndexRange::Last => {
                self.enqueue_as_first_as_last(to_index, range_to_insert);
            }
            IndexRange::Between {
                left_index: _,
                right_index,
            } => {
                self.enqueue_as_first_as_between(right_index, to_index, range_to_insert);
            }
            IndexRange::JoinToIndexFrom(from_index) => {
                self.enqueue_as_left_as_joint_to_index_from(from_index, to_index, range_to_insert)
            }
            IndexRange::JoinToIndexTo(from_index) => {
                self.enqueue_as_left_as_join_to_index_to(from_index, to_index, range_to_insert)
            }
            IndexRange::MergeIntervals(index) => {
                self.enqueue_as_first_merge_intervals(index, to_index, range_to_insert);
            }
        }
    }

    fn enqueue_as_first_merge_intervals(
        &mut self,
        from_index: usize,
        to_index: IndexRange,
        range_to_insert: QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(index) => {
                self.do_update(from_index, index, None, None);
            }
            IndexRange::JoinToIndexFrom(index) => {
                self.do_update(from_index, index, None, None);
            }
            IndexRange::JoinToIndexTo(index) => {
                self.do_update(from_index, index, None, range_to_insert.to_id.into());
            }
            IndexRange::First => {
                panic!(
                    "Not possible be at first interval. Intervals: {:?}. range_to_insert {:?}",
                    self.intervals, range_to_insert
                );
            }
            IndexRange::Last => {
                self.do_update(
                    from_index,
                    self.intervals.len() - 1,
                    None,
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::Between {
                left_index,
                right_index: _,
            } => {
                self.do_update(from_index, left_index, None, range_to_insert.to_id.into());
            }
            IndexRange::MergeIntervals(index) => {
                self.do_update(from_index, index + 1, None, None);
            }
        }
    }

    fn enqueue_as_first_as_last(&mut self, to_index: IndexRange, range_to_insert: QueueIndexRange) {
        match to_index {
            IndexRange::Exact(_) => {
                panic!(
                    "Index can not be between Last and other element. Intervals: {:?}. RangeToInsert: {:?}",
                    self.intervals,
                    range_to_insert
                );
            }
            IndexRange::First => {
                panic!("Index can not be between Last and First elements. Intervals: {:?}. RangeToInsert: {:?}",
                    self.intervals,
                    range_to_insert);
            }
            IndexRange::Last => {
                let last = self.intervals.last_mut().unwrap();

                if last.to_id + 1 == range_to_insert.from_id {
                    last.to_id = range_to_insert.to_id;
                    return;
                }

                self.intervals.push(range_to_insert);
                return;
            }
            IndexRange::Between {
                left_index: _,
                right_index: _,
            } => {
                panic!("Index can not be between Last and Between elements. Intervals: {:?}. RangeToInsert: {:?}",
                    self.intervals,
                    range_to_insert);
            }
            IndexRange::JoinToIndexFrom(index) => {
                panic!("Index can not be between Last and JointToIndexFrom({}). Intervals: {:?}. range_to_insert: {:?}", 
                index,  self.intervals,
                range_to_insert );
            }
            IndexRange::JoinToIndexTo(index) => {
                panic!("Index can not be between Last and join_index_to {} element. Intervals: {:?}. range_to_insert: {:?}",
                index,
                self.intervals,
                range_to_insert)
            }
            IndexRange::MergeIntervals(index) => {
                panic!(
                    "Index can not be between Last and Merge Intervals [{}] elements. Intervals: {:?}. range_to_insert: {:?}",
                    index,
                    self.intervals,
                    range_to_insert
                );
            }
        }
    }

    fn enqueue_as_first_as_between(
        &mut self,
        left_index_to: usize,
        to_index: IndexRange,
        range_to_insert: QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(to_index) => {
                self.do_update(
                    left_index_to,
                    to_index,
                    range_to_insert.from_id.into(),
                    None,
                );
            }
            IndexRange::First => {
                if left_index_to > 0 {
                    panic!(
                        "Somehow right_index={} and left index=0. Intervals: {:?}. range_to_insert: {:?}",
                        left_index_to, self.intervals, range_to_insert
                    )
                }

                self.intervals.insert(0, range_to_insert);
            }
            IndexRange::Last => {
                self.do_update(
                    left_index_to,
                    self.intervals.len() - 1,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::Between {
                left_index: to_left_index,
                right_index: to_right_index,
            } => {
                if left_index_to == to_right_index {
                    self.intervals.insert(to_right_index, range_to_insert);
                    return;
                }

                self.do_update(
                    left_index_to,
                    to_left_index,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::JoinToIndexFrom(index) => {
                self.do_update(left_index_to, index, range_to_insert.from_id.into(), None);
            }
            IndexRange::JoinToIndexTo(index) => {
                self.do_update(
                    left_index_to,
                    index,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::MergeIntervals(index) => {
                self.do_update(
                    left_index_to,
                    index + 1,
                    range_to_insert.from_id.into(),
                    None,
                );
            }
        }
    }

    fn enqueue_as_left_as_joint_to_index_from(
        &mut self,
        from_index: usize,
        to_index: IndexRange,
        range_to_insert: QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(to_index) => {
                self.do_update(from_index, to_index, range_to_insert.from_id.into(), None);
            }
            IndexRange::First => {
                panic!("Position between some interval and first element is not possible");
            }
            IndexRange::Last => {
                self.do_update(
                    from_index,
                    self.intervals.len() - 1,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::Between {
                left_index,
                right_index: _,
            } => {
                self.do_update(
                    from_index,
                    left_index,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
                //  self.insert_with_override_to_left(from_index, left_index, range_to_insert);
            }
            IndexRange::JoinToIndexFrom(to_index) => {
                self.do_update(from_index, to_index, range_to_insert.from_id.into(), None);
            }
            IndexRange::JoinToIndexTo(to_index) => {
                self.do_update(
                    from_index,
                    to_index,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::MergeIntervals(index) => {
                self.do_update(from_index, index + 1, range_to_insert.from_id.into(), None);
            }
        }
    }

    fn enqueue_as_left_as_join_to_index_to(
        &mut self,
        from_index: usize,
        to_index: IndexRange,
        range_to_insert: QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(to_index) => {
                self.do_update(from_index, to_index, None, None);
            }
            IndexRange::First => {
                panic!("Position between some interval and first element is not possible");
            }
            IndexRange::Last => {
                self.do_update(
                    from_index,
                    self.intervals.len() - 1,
                    None,
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::Between {
                left_index,
                right_index: _,
            } => {
                self.do_update(from_index, left_index, None, range_to_insert.to_id.into());
            }
            IndexRange::JoinToIndexFrom(to_index) => {
                self.do_update(from_index, to_index, None, None);
            }
            IndexRange::JoinToIndexTo(index) => {
                self.do_update(from_index, index, None, range_to_insert.to_id.into());
            }
            IndexRange::MergeIntervals(index) => {
                self.do_update(from_index, index + 1, None, None);
            }
        }
    }

    fn enqueue_as_left_as_exact(
        &mut self,
        from_index: usize,
        to_index: IndexRange,
        range_to_insert: QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(to_index) => {
                self.do_update(from_index, to_index, None, None);
            }
            IndexRange::First => {
                panic!("Position between some interval and first element is not possible");
            }
            IndexRange::Last => {
                self.do_update(
                    from_index,
                    self.intervals.len() - 1,
                    None,
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::Between {
                left_index,
                right_index: _,
            } => {
                self.do_update(from_index, left_index, None, range_to_insert.to_id.into());
            }
            IndexRange::JoinToIndexFrom(index) => {
                self.do_update(from_index, index, range_to_insert.from_id.into(), None);
            }
            IndexRange::JoinToIndexTo(to_index) => {
                self.do_update(
                    from_index,
                    to_index,
                    range_to_insert.from_id.into(),
                    range_to_insert.to_id.into(),
                );
            }
            IndexRange::MergeIntervals(index) => {
                self.do_update(from_index, index + 1, range_to_insert.from_id.into(), None);
            }
        }
    }

    fn do_update(
        &mut self,
        from_index: usize,
        to_index: usize,
        override_from_id: Option<i64>,
        override_to_id: Option<i64>,
    ) {
        let to_id = if let Some(override_to_id) = override_to_id {
            override_to_id
        } else {
            self.intervals.get(to_index).unwrap().to_id
        };

        for _ in from_index..to_index {
            self.intervals.remove(from_index + 1);
        }
        if self.intervals.len() == 0 {
            panic!("Somehow intervals got empty");
        }

        let first = self.intervals.get_mut(from_index).unwrap();

        if let Some(from_id) = override_from_id {
            first.from_id = from_id;
        }

        first.to_id = to_id;
    }
}

#[cfg(test)]
mod tests_left_is_exact {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn enqueue_left_is_exact_and_join_to_left() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(10, 29);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_exact_and_join_to_right_same_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(10, 21);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(21, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_exact_and_join_to_right_skipping_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(10, 41);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(41, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_exact_and_join_to_right_as_last() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(10, 81);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(81, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_iss_exact_and_merge_next_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 30));
        queue.enqueue_range(QueueIndexRange::restore(40, 50));
        queue.enqueue_range(QueueIndexRange::restore(60, 70));

        let range_to_insert = QueueIndexRange::restore(10, 21);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(30, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(40, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(50, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(60, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_exact_and_merge_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(20, 30));
        queue.enqueue_range(QueueIndexRange::restore(32, 50));
        queue.enqueue_range(QueueIndexRange::restore(60, 70));

        let range_to_insert = QueueIndexRange::restore(10, 31);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(50, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(60, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_exact_and_between_single_interval() {
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
    fn enqueue_left_is_second_exact_and_to_last() {
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

    #[test]
    fn enqueue_left_is_second_exact_and_between_two_intervals_after() {
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
}

#[cfg(test)]
mod tests_left_as_join_to_index_to {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn enqueue_left_as_join_to_index_to_and_merge_to_intervals_next() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 29);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_to_and_join_to_index_to() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 41);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(41, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_to_and_same_between() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 25);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_to_and_exact() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 35);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_to_and_after() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 55);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_to_and_last() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_second_join_to_index_to_and_last() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(41, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(1).unwrap().to_id);
    }
    #[test]
    fn enqueue_left_as_second_join_to_index_to_and_next_exact() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(41, 55);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(20, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_joint_to_index_to_and_join_to_index_to() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 41);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(41, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_joint_to_index_to_and_merge_two_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 41);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);
    }
}

#[cfg(test)]
mod tests_left_as_join_to_index_from {
    //Join Left as join_to_left
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn enqueue_left_as_join_to_index_from_and_join_to_left() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 29);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_from_and_join_to_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 21);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(21, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_from_between() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 25);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(25, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_from_exact() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 32);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_join_to_index_from_to_last() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 85);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(85, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_joint_to_index_from_to_merge_two_intervals_next() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 21);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_as_joint_to_index_from_to_merge_two_intervals_skip_two() {
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(9, 41);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(9, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);
    }
}

#[cfg(test)]
mod tests_left_is_between {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn enqueue_left_is_between_and_left_to_the_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(25, 29);

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
    fn enqueue_left_is_between_and_right_to_the_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(25, 29);

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
    fn enqueue_left_is_between_and_exact_next() {
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
    fn enqueue_left_is_between_and_exact_after_interval() {
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
    fn enqueue_left_is_between_covering_second_interval() {
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
    fn enqueue_left_is_second_between_and_last() {
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
    fn enqueue_left_is_last_between_and_last_interval() {
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
    fn enqueue_left_between_first_and_between_third_intervals() {
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
    fn enqueue_left_between_first_and_between_second_intervals() {
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
    fn test_some_other_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));
        queue.enqueue_range(QueueIndexRange::restore(90, 100));

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
}

#[cfg(test)]
mod tests_left_is_first {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn enqueue_left_is_first_and_to_merging_two_intervals_after_several_intervals() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(62, 70));
        queue.enqueue_range(QueueIndexRange::restore(80, 90));

        let range_to_insert = QueueIndexRange::restore(5, 61);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(80, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(90, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_first_and_to_merging_two_intervals_as_next_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(5, 21);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(5, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn enqueue_left_is_first_covering_first_two_intervals() {
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
    fn enqueue_left_is_first_covering_first_interval() {
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
    fn enqueue_range_from_first_to_next_to_the_right() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(8, 21);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(4, queue.intervals.len());

        assert_eq!(8, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(21, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(30, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(2).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(3).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(3).unwrap().to_id);
    }

    #[test]
    fn enqueue_range_from_first_to_right() {
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
    fn test_some_case() {
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
}

#[cfg(test)]
mod tests_left_is_merge_two_intervals {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn test_second_is_exact_same_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 25);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(40, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn test_second_is_exact_after_the_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 55);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_second_as_between_intervals_next() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 45);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(45, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(50, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn test_second_as_between_intervals_skipping_single() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 65);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(65, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_second_as_joining_two_intervals_next() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_insert = QueueIndexRange::restore(21, 41);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(60, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(70, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(80, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_second_as_joining_two_intervals_skipping_one_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(62, 70));
        queue.enqueue_range(QueueIndexRange::restore(80, 90));

        let range_to_insert = QueueIndexRange::restore(21, 61);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(80, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(90, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_second_as_last() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(62, 70));
        queue.enqueue_range(QueueIndexRange::restore(80, 90));

        let range_to_insert = QueueIndexRange::restore(21, 95);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(1, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(95, queue.intervals.get(0).unwrap().to_id);
    }

    #[test]
    fn test_second_as_join_to_index_to_same_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 30));
        queue.enqueue_range(QueueIndexRange::restore(40, 50));
        queue.enqueue_range(QueueIndexRange::restore(60, 70));

        let range_to_insert = QueueIndexRange::restore(21, 31);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(3, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(31, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(40, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(50, queue.intervals.get(1).unwrap().to_id);

        assert_eq!(60, queue.intervals.get(2).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(2).unwrap().to_id);
    }

    #[test]
    fn test_second_as_join_to_index_to_skip_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 30));
        queue.enqueue_range(QueueIndexRange::restore(40, 50));
        queue.enqueue_range(QueueIndexRange::restore(60, 70));

        let range_to_insert = QueueIndexRange::restore(21, 51);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(51, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(60, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(1).unwrap().to_id);
    }

    #[test]
    fn test_second_as_join_to_index_from_same_interval() {
        //Preparing data
        let mut queue = QueueWithIntervals::new();
        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 30));
        queue.enqueue_range(QueueIndexRange::restore(40, 50));
        queue.enqueue_range(QueueIndexRange::restore(60, 70));

        let range_to_insert = QueueIndexRange::restore(21, 39);

        // Executing data
        queue.enqueue_range(range_to_insert);

        assert_eq!(2, queue.intervals.len());

        assert_eq!(10, queue.intervals.get(0).unwrap().from_id);
        assert_eq!(50, queue.intervals.get(0).unwrap().to_id);

        assert_eq!(60, queue.intervals.get(1).unwrap().from_id);
        assert_eq!(70, queue.intervals.get(1).unwrap().to_id);
    }
}
