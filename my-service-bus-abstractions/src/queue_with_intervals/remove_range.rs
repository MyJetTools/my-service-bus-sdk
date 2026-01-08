use super::{IndexRange, QueueIndexRange, QueueWithIntervals};

pub enum IndexToRemoveRange {
    NothingToDo,
    RemoveIntervals {
        from_index: usize,
        to_index: usize,
    },
    UpdateInterval {
        start_index_to_remove: usize,
        index: usize,
        from_id: i64,
        to_id: i64,
    },
}

impl QueueWithIntervals {
    pub fn remove_range(&mut self, range_to_remove: &QueueIndexRange) {
        let (from_index, to_index) = IndexRange::new(&self.intervals, range_to_remove);

        match from_index {
            IndexRange::Exact(exact_index) => {
                self.left_is_exact(exact_index, to_index, range_to_remove);
            }
            IndexRange::First => return self.left_is_between(0, to_index, range_to_remove),
            IndexRange::Last => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::First => todo!(),
                IndexRange::Last => return,
                IndexRange::Between {
                    left_index: _left_index,
                    right_index: _right_index,
                } => todo!(),
                IndexRange::JoinToIndexFrom(_) => todo!(),
                IndexRange::JoinToIndexTo(_) => todo!(),
                IndexRange::MergeIntervals(_index) => {
                    todo!("Implement")
                }
            },
            IndexRange::Between {
                left_index: _,
                right_index,
            } => return self.left_is_between(right_index, to_index, range_to_remove),
            IndexRange::JoinToIndexFrom(from_index) => {
                return self.left_is_between(from_index, to_index, range_to_remove);
            }
            IndexRange::JoinToIndexTo(from_index) => {
                return self.left_is_between(from_index + 1, to_index, range_to_remove);
            }
            IndexRange::MergeIntervals(_index) => {
                todo!("Implement")
            }
        }
    }

    fn left_is_between(
        &mut self,
        from_index: usize,
        to_index: IndexRange,
        range_to_remove: &QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(index) => {
                self.remove_and_update(from_index, index, range_to_remove.into());
            }
            IndexRange::JoinToIndexFrom(index) => {
                if index > from_index {
                    self.remove_intervals(from_index, index);
                }
            }
            IndexRange::JoinToIndexTo(index) => {
                self.remove_intervals(from_index, index + 1);
            }
            IndexRange::First => {}
            IndexRange::Last => {
                if from_index == 0 {
                    self.clean();
                } else {
                    self.remove_intervals(from_index, self.intervals.len());
                }
            }
            IndexRange::Between {
                left_index: _,
                right_index,
            } => {
                self.remove_intervals(from_index, right_index);
            }
            IndexRange::MergeIntervals(index) => {
                self.remove_intervals(from_index, index + 1);
            }
        }
    }

    fn left_is_exact(
        &mut self,
        from_index: usize,
        to_index: IndexRange,
        range_to_remove: &QueueIndexRange,
    ) {
        match to_index {
            IndexRange::Exact(to_index) => {
                if to_index == from_index {
                    let item = self.intervals.get_mut(from_index).unwrap();

                    // Remove the tail starting from range_to_remove.from_id through range_to_remove.to_id.
                    if range_to_remove.to_id >= item.to_id {
                        item.make_empty();
                    } else if range_to_remove.from_id <= item.from_id {
                        item.from_id = range_to_remove.to_id + 1;
                    } else {
                        item.to_id = range_to_remove.from_id - 1;
                    }

                    // Remove all intervals after the current one, since range extends beyond or ends here.
                    if self.intervals.len() > from_index + 1 {
                        self.remove_intervals(from_index + 1, self.intervals.len());
                    }
                } else {
                    let item = self.intervals.get_mut(to_index).unwrap();

                    if range_to_remove.to_id >= item.to_id {
                        // Range ends at or after the end of this interval; drop it completely.
                        self.remove_intervals(from_index + 1, to_index + 1);
                    } else {
                        // Trim the left part of the target interval and remove the ones in between.
                        item.from_id = range_to_remove.to_id + 1;
                        self.remove_intervals(from_index + 1, to_index);
                    }
                }
            }
            IndexRange::First => {
                return self.left_is_between(from_index, to_index, range_to_remove)
            }
            IndexRange::Last => {
                // Remove everything after the starting interval.
                self.remove_intervals(from_index + 1, self.intervals.len());
            }
            IndexRange::Between {
                left_index,
                right_index,
            } => {
                // Start removal right after the starting interval.
                return self.left_is_between(
                    from_index + 1,
                    IndexRange::Between {
                        left_index,
                        right_index,
                    },
                    range_to_remove,
                );
            }
            IndexRange::JoinToIndexFrom(index) => {
                // Remove intervals fully covered before the join target (not including it).
                self.remove_intervals(from_index + 1, index);
            }
            IndexRange::JoinToIndexTo(index) => {
                // Remove intervals up to and including the interval that ends exactly at to_id.
                self.remove_intervals(from_index + 1, index + 1);
            }
            IndexRange::MergeIntervals(_index) => {
                // Remove intervals that are fully covered when the end touches a merge point.
                self.remove_intervals(from_index + 1, _index + 1);
            }
        }
    }

    fn remove_and_update(
        &mut self,
        from_index: usize,
        to_index: usize,
        range_to_remove: &QueueIndexRange,
    ) {
        let item = self.intervals.get_mut(to_index).unwrap();
        if item.to_id == range_to_remove.to_id {
            item.make_empty();
            self.remove_intervals(from_index, to_index + 1);
        } else {
            item.from_id = range_to_remove.to_id + 1;

            self.remove_intervals(from_index, to_index);
        }
    }

    fn remove_intervals(&mut self, from_index: usize, to_index: usize) {
        for _ in from_index..to_index {
            if self.intervals.len() > 1 {
                self.intervals.remove(from_index);
            }
        }
    }
}

#[cfg(test)]
mod tests_basics {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn test_all_cases_do_nothing() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 7);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        let range_to_remove = QueueIndexRange::restore(21, 29);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        let range_to_remove = QueueIndexRange::restore(42, 48);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        let range_to_remove = QueueIndexRange::restore(42, 49);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        let range_to_remove = QueueIndexRange::restore(81, 85);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        let range_to_remove = QueueIndexRange::restore(82, 85);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);
    }

    #[test]
    fn test_all_cases_we_go_between_intervals() {}
}

#[cfg(test)]
mod tests_left_is_first {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn test_right_is_between() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 25);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_is_between_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 45);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 10);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 11);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(3).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(3).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 20);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 33);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 34);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 40);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_last_interval_cutting_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 80);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().is_empty(), true);
    }

    #[test]
    fn test_last_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 82);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().is_empty(), true);
    }

    #[test]
    fn test_with_merge_intervals() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 21);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 22);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_merge_intervals_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 42);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 21);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to_skipping_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(5, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }
}

#[cfg(test)]
mod tests_left_is_between {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    #[test]
    fn test_right_is_between() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 45);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_is_between_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 65);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 30);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 31);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(3).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(3).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 40);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 53);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 54);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 60);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_last_interval_cutting_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 80);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);
    }

    #[test]
    fn test_last_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 82);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);
    }

    #[test]
    fn test_with_merge_intervals() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 42);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_merge_intervals_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(62, 80));

        let range_to_remove = QueueIndexRange::restore(25, 61);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 62);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to_skipping_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(25, 61);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }
}

#[cfg(test)]
mod tests_left_is_joining_to_from_id {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    const FROM_ID: i64 = 9;

    #[test]
    fn test_right_is_between() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 25);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_is_between_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 45);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 10);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 11);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(3).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(3).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 20);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 33);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 34);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 40);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_last_interval_cutting_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 80);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().is_empty(), true);
    }

    #[test]
    fn test_last_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 82);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().is_empty(), true);
    }

    #[test]
    fn test_with_merge_intervals() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(22, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 21);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 22);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_merge_intervals_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 42);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 21);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 30);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to_skipping_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }
}

#[cfg(test)]
mod tests_left_is_joining_to_to_id {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    const FROM_ID: i64 = 21;

    #[test]
    fn test_right_is_between() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 45);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_is_between_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 65);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 30);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 31);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(3).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(3).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 40);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 53);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 54);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 60);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_last_interval_cutting_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 80);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);
    }

    #[test]
    fn test_last_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 82);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);
    }

    #[test]
    fn test_with_merge_intervals() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 42);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_merge_intervals_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(62, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 61);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 62);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to_skipping_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 61);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }
}

#[cfg(test)]
mod tests_left_is_first_exact {
    use crate::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

    const FROM_ID: i64 = 15;

    #[test]
    fn test_right_is_between() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 45);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_is_between_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 65);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 30);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 4);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 31);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 40);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(3).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(3).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_first_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 40);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 53);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 54);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_right_exact_second_cutting_it_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 60);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_last_interval_cutting_off() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 80);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);
    }

    #[test]
    fn test_last_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 82);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 1);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);
    }

    #[test]
    fn test_with_merge_intervals() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(42, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 42);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_merge_intervals_skipping_one() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(62, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 61);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 62);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 41);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 3);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 50);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 60);

        assert_eq!(queue.intervals.get(2).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(2).unwrap().to_id, 80);
    }

    #[test]
    fn test_with_joint_to_index_to_skipping_interval() {
        let mut queue = QueueWithIntervals::new();

        queue.enqueue_range(QueueIndexRange::restore(10, 20));
        queue.enqueue_range(QueueIndexRange::restore(30, 40));
        queue.enqueue_range(QueueIndexRange::restore(50, 60));
        queue.enqueue_range(QueueIndexRange::restore(70, 80));

        let range_to_remove = QueueIndexRange::restore(FROM_ID, 61);
        queue.remove_range(&range_to_remove);
        assert_eq!(queue.intervals.len(), 2);

        assert_eq!(queue.intervals.get(0).unwrap().from_id, 10);
        assert_eq!(queue.intervals.get(0).unwrap().to_id, 20);

        assert_eq!(queue.intervals.get(1).unwrap().from_id, 70);
        assert_eq!(queue.intervals.get(1).unwrap().to_id, 80);
    }
}
