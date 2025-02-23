use super::QueueIndexRange;

#[derive(Debug)]
pub enum IndexRange {
    Exact(usize),
    JoinToIndexFrom(usize),
    JoinToIndexTo(usize),
    First,
    Last,
    Between {
        left_index: usize,
        right_index: usize,
    },
    MergeIntervals(usize),
}
impl IndexRange {
    pub fn new(intervals: &Vec<QueueIndexRange>, other_range: &QueueIndexRange) -> (Self, Self) {
        let mut from_index = None;
        let mut to_index = None;

        let mut index = 0;

        let mut prev_interval: Option<&QueueIndexRange> = None;

        for interval in intervals {
            match &prev_interval {
                Some(prev_interval) => {
                    if prev_interval.to_id + 1 == other_range.from_id
                        && other_range.from_id + 1 == interval.from_id
                    {
                        from_index = Self::MergeIntervals(index - 1).into();
                    }

                    if prev_interval.to_id + 1 == other_range.to_id
                        && other_range.to_id + 1 == interval.from_id
                    {
                        to_index = Self::MergeIntervals(index - 1).into();
                    }

                    if from_index.is_some() && to_index.is_some() {
                        break;
                    }

                    if from_index.is_none() {
                        if prev_interval.to_id + 1 == other_range.to_id {
                            to_index = Some(Self::JoinToIndexTo(index))
                        } else if prev_interval.to_id + 1 == other_range.from_id {
                            from_index = Some(IndexRange::JoinToIndexTo(index - 1))
                        } else if prev_interval.to_id < other_range.from_id
                            && other_range.from_id < interval.from_id
                        {
                            from_index = Some(IndexRange::Between {
                                left_index: index - 1,
                                right_index: index,
                            });
                        }
                    }

                    if to_index.is_none() {
                        if other_range.to_id + 1 == interval.from_id {
                            if prev_interval.to_id + 1 == other_range.to_id {
                                to_index = Some(IndexRange::MergeIntervals(index - 1))
                            } else {
                                to_index = Some(IndexRange::JoinToIndexFrom(index))
                            }
                        } else if prev_interval.to_id + 1 == other_range.to_id {
                            to_index = Some(Self::JoinToIndexTo(index - 1))
                        } else if prev_interval.to_id < other_range.to_id
                            && other_range.to_id < interval.from_id
                        {
                            to_index = Some(IndexRange::Between {
                                left_index: index - 1,
                                right_index: index,
                            });
                        }
                    }
                }
                None => {
                    if other_range.from_id + 1 == interval.from_id {
                        from_index = Some(IndexRange::JoinToIndexFrom(0));
                    } else if other_range.from_id < interval.from_id {
                        from_index = Some(IndexRange::First);
                    }

                    if other_range.to_id + 1 == interval.from_id {
                        to_index = Some(IndexRange::JoinToIndexFrom(0));
                    } else if interval.to_id + 1 == other_range.to_id {
                        to_index = Some(IndexRange::JoinToIndexTo(0));
                    } else if other_range.to_id < interval.from_id {
                        to_index = Some(IndexRange::First);
                    }
                }
            }

            if interval.is_in_my_interval(other_range.from_id) {
                from_index = Some(IndexRange::Exact(index));
            }

            if interval.is_in_my_interval(other_range.to_id) {
                to_index = Some(IndexRange::Exact(index));
            }

            prev_interval = Some(&interval);

            index += 1;
        }

        let to_index = match to_index {
            Some(to_index) => to_index,
            None => IndexRange::Last,
        };

        let from_index = match from_index {
            Some(from_index) => from_index,
            None => {
                if intervals.last().unwrap().to_id + 1 == other_range.from_id {
                    IndexRange::JoinToIndexTo(intervals.len() - 1)
                } else {
                    IndexRange::Last
                }
            }
        };

        (from_index, to_index)
    }

    #[cfg(test)]
    pub fn unwrap_as_exact(&self) -> usize {
        match self {
            Self::Exact(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    #[cfg(test)]
    pub fn unwrap_as_merge_intervals(&self) -> usize {
        match self {
            Self::MergeIntervals(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    #[cfg(test)]
    pub fn unwrap_as_between(&self) -> (usize, usize) {
        match self {
            Self::Between {
                left_index,
                right_index,
            } => (*left_index, *right_index),
            _ => panic!("{:?}", self),
        }
    }

    #[cfg(test)]
    pub fn is_first(&self) -> bool {
        match self {
            Self::First => true,
            _ => false,
        }
    }

    #[cfg(test)]
    pub fn unwrap_as_join_to_index_from(&self) -> usize {
        match self {
            Self::JoinToIndexFrom(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    #[cfg(test)]
    pub fn unwrap_as_join_to_index_to(&self) -> usize {
        match self {
            Self::JoinToIndexTo(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    #[cfg(test)]
    pub fn is_last(&self) -> bool {
        match self {
            Self::Last => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::QueueIndexRange;

    use super::IndexRange;

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(10, 15)];

        let new_interval = QueueIndexRange::restore(20, 25);

        // Checking if index_form and index_to are calculated ok
        let (index_from, index_to) = IndexRange::new(&intervals, &new_interval);
        assert!(index_from.is_last());
        assert!(index_to.is_last());
    }

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list_with_merge() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(10, 15)];

        let new_interval = QueueIndexRange::restore(16, 25);

        // Checking if index_form and index_to are calculated ok
        let (index_from, index_to) = IndexRange::new(&intervals, &new_interval);
        assert_eq!(index_from.unwrap_as_join_to_index_to(), 0);
        assert!(index_to.is_last());
    }

    #[test]
    fn enqueue_range_at_the_beginning() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(15, 20)];

        let range_to_insert = QueueIndexRange::restore(5, 10);

        let (from_id, to_id) = IndexRange::new(&intervals, &range_to_insert);

        assert!(from_id.is_first());
        assert!(to_id.is_first());
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 15),
            QueueIndexRange::restore(20, 25),
        ];

        // Doing action
        let range_to_insert = QueueIndexRange::restore(5, 12);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert!(from_index.is_first());

        assert_eq!(to_index.unwrap_as_exact(), 0);
    }

    #[test]
    fn enqueue_range_at_the_beginning_joining_the_first_and_second_intervals() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
            QueueIndexRange::restore(90, 100),
        ];

        // Doing action
        let range_to_insert = QueueIndexRange::restore(35, 75);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert_eq!(from_index.unwrap_as_exact(), 1);

        assert_eq!(to_index.unwrap_as_exact(), 3);
    }

    #[test]
    fn enqueue_range_from_first_covering_first_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(5, 25);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert!(from_index.is_first());

        let to_index = to_index.unwrap_as_between();

        assert_eq!(to_index.0, 0);
        assert_eq!(to_index.1, 1);
    }

    #[test]
    fn enqueue_range_from_first_covering_first_two_intervals() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(5, 45);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert!(from_index.is_first());

        let to_index = to_index.unwrap_as_between();

        assert_eq!(to_index.0, 1);
        assert_eq!(to_index.1, 2);
    }

    #[test]
    fn enqueue_range_from_covering_second_interval() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(25, 45);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();

        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        let to_index = to_index.unwrap_as_between();

        assert_eq!(to_index.0, 1);
        assert_eq!(to_index.1, 2);
    }

    #[test]
    fn enqueue_range_from_covering_second_and_third_intervals() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(25, 65);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();

        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        let to_index = to_index.unwrap_as_between();

        assert_eq!(to_index.0, 2);
        assert_eq!(to_index.1, 3);
    }

    #[test]
    fn enqueue_range_from_covering_last_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(65, 85);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();

        assert_eq!(from_index.0, 2);
        assert_eq!(from_index.1, 3);

        assert!(to_index.is_last());
    }

    #[test]
    fn enqueue_range_from_covering_last_two_intervals() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(45, 85);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();

        assert_eq!(from_index.0, 1);
        assert_eq!(from_index.1, 2);

        assert!(to_index.is_last());
    }

    #[test]
    fn enqueue_range_from_covering_everything() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(5, 85);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert!(from_index.is_first());

        assert!(to_index.is_last());
    }

    #[test]
    fn enqueue_range_covering_between_and_exact_single_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(25, 35);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        assert_eq!(to_index.unwrap_as_exact(), 1);
    }

    #[test]
    fn enqueue_range_covering_between_and_exact_two_intervals() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(25, 55);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        assert_eq!(to_index.unwrap_as_exact(), 2);
    }

    #[test]
    fn enqueue_range_covering_exact_and_between_single_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(35, 45);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert_eq!(from_index.unwrap_as_exact(), 1);

        let to_index = to_index.unwrap_as_between();
        assert_eq!(to_index.0, 1);
        assert_eq!(to_index.1, 2);
    }

    #[test]
    fn enqueue_range_covering_exact_and_between_two_intervals() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(35, 65);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert_eq!(from_index.unwrap_as_exact(), 1);

        let to_index = to_index.unwrap_as_between();
        assert_eq!(to_index.0, 2);
        assert_eq!(to_index.1, 3);
    }

    #[test]
    fn enqueue_range_covering_exact_and_to_last() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_insert = QueueIndexRange::restore(35, 85);

        let (from_index, to_index) = IndexRange::new(&intervals, &range_to_insert);

        assert_eq!(from_index.unwrap_as_exact(), 1);

        assert!(to_index.is_last());
    }

    #[test]
    fn test_index_to_insert_range() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
        ];

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(31, 32));

        assert_eq!(from_index.unwrap_as_exact(), 1);
        assert_eq!(to_index.unwrap_as_exact(), 1);
    }

    #[test]
    fn test_inserting_range_exactly_between() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let other_range = QueueIndexRange::restore(21, 29);

        let (from_index, to_index) = IndexRange::new(&intervals, &other_range);

        assert_eq!(from_index.unwrap_as_join_to_index_to(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_from(), 1);

        let other_range = QueueIndexRange::restore(5, 9);
        let (from_index, to_index) = IndexRange::new(&intervals, &other_range);

        assert!(from_index.is_first());

        assert_eq!(to_index.unwrap_as_join_to_index_from(), 0);

        let other_range = QueueIndexRange::restore(81, 85);
        let (from_index, to_index) = IndexRange::new(&intervals, &other_range);

        assert_eq!(from_index.unwrap_as_join_to_index_to(), 3);
        assert!(to_index.is_last());
    }

    #[test]
    fn test_index_to_insert_range_3() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(25, 65));

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        let to_index = to_index.unwrap_as_between();
        assert_eq!(to_index.0, 2);
        assert_eq!(to_index.1, 3);

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(25, 70));

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        assert_eq!(to_index.unwrap_as_exact(), 3);

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(25, 45));

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        let to_index = to_index.unwrap_as_between();
        assert_eq!(to_index.0, 1);
        assert_eq!(to_index.1, 2);

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(5, 85));

        assert!(from_index.is_first());
        assert!(to_index.is_last());
    }

    #[test]
    fn test_index_to_insert_range_at_and_after() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
        ];

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(31, 45));

        assert_eq!(from_index.unwrap_as_exact(), 1);
        assert!(to_index.is_last());

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(5, 6));

        assert!(from_index.is_first());
        assert!(to_index.is_first());

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(45, 45));

        assert!(from_index.is_last());
        assert!(to_index.is_last());

        let (from_index, to_index) = IndexRange::new(&intervals, &QueueIndexRange::restore(5, 45));

        assert!(from_index.is_first());
        assert!(to_index.is_last());
    }

    #[test]
    fn test_join_indexes() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
        ];

        let range = QueueIndexRange::restore(5, 9);
        let (from_index, to_index) = IndexRange::new(&intervals, &range);
        assert_eq!(from_index.is_first(), true);
        assert_eq!(to_index.unwrap_as_join_to_index_from(), 0);

        let range = QueueIndexRange::restore(9, 21);
        let (from_index, to_index) = IndexRange::new(&intervals, &range);
        assert_eq!(from_index.unwrap_as_join_to_index_from(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_to(), 0);

        let range = QueueIndexRange::restore(9, 29);
        let (from_index, to_index) = IndexRange::new(&intervals, &range);
        assert_eq!(from_index.unwrap_as_join_to_index_from(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_from(), 1);

        let range = QueueIndexRange::restore(21, 29);
        let (from_index, to_index) = IndexRange::new(&intervals, &range);
        assert_eq!(from_index.unwrap_as_join_to_index_to(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_from(), 1);

        let index = QueueIndexRange::restore(41, 49);
        let (from_index, to_index) = IndexRange::new(&intervals, &index);

        assert_eq!(from_index.unwrap_as_join_to_index_to(), 1);
        assert_eq!(to_index.unwrap_as_join_to_index_from(), 2);

        let index = QueueIndexRange::restore(21, 49);
        let (from_index, to_index) = IndexRange::new(&intervals, &index);
        assert_eq!(from_index.unwrap_as_join_to_index_to(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_from(), 2);

        let index = QueueIndexRange::restore(61, 70);
        let (from_index, to_index) = IndexRange::new(&intervals, &index);
        assert_eq!(from_index.unwrap_as_join_to_index_to(), 2);
        assert_eq!(to_index.is_last(), true);
    }

    #[test]
    fn test_join_indexes_with_one_value_between() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(62, 65),
        ];

        let index = QueueIndexRange::restore(15, 61);
        let (from_index, to_index) = IndexRange::new(&intervals, &index);
        assert_eq!(from_index.unwrap_as_exact(), 0);
        assert_eq!(to_index.unwrap_as_merge_intervals(), 2);

        let index = QueueIndexRange::restore(61, 68);
        let (from_index, to_index) = IndexRange::new(&intervals, &index);
        assert_eq!(from_index.unwrap_as_merge_intervals(), 2);
        assert_eq!(to_index.is_last(), true);
    }

    #[test]
    fn test_second_as_join_to_index_to_same_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(22, 30),
            QueueIndexRange::restore(40, 50),
            QueueIndexRange::restore(60, 70),
        ];

        let other_range = QueueIndexRange::restore(21, 31);

        let (from_index, to_index) = IndexRange::new(&intervals, &other_range);

        assert_eq!(from_index.unwrap_as_merge_intervals(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_to(), 1);
    }

    #[test]
    fn test_second_as_join_to_index_to_skip_interval() {
        //Preparing data
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(22, 30),
            QueueIndexRange::restore(40, 50),
            QueueIndexRange::restore(60, 70),
        ];

        let other_range = QueueIndexRange::restore(21, 51);

        let (from_index, to_index) = IndexRange::new(&intervals, &other_range);

        assert_eq!(from_index.unwrap_as_merge_intervals(), 0);
        assert_eq!(to_index.unwrap_as_join_to_index_to(), 2);
    }
}
