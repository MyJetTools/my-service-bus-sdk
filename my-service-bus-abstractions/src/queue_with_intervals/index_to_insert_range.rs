use super::QueueIndexRange;

#[derive(Debug)]
pub enum IndexToInsertRange {
    Exact(usize),
    First,
    Last,
    Between {
        left_index: usize,
        right_index: usize,
    },
}
impl IndexToInsertRange {
    pub fn new(
        intervals: &Vec<QueueIndexRange>,
        range_to_insert: &QueueIndexRange,
    ) -> (Self, Self) {
        let mut from_index = None;
        let mut to_index = None;

        let mut index = 0;

        let mut prev_interval: Option<QueueIndexRange> = None;

        for interval in intervals {
            if from_index.is_some() && to_index.is_some() {
                break;
            }

            match &prev_interval {
                Some(prev_interval) => {
                    if prev_interval.to_id + 1 == range_to_insert.from_id {
                        from_index = Some(IndexToInsertRange::Exact(index - 1))
                    } else if prev_interval.to_id < range_to_insert.from_id
                        && range_to_insert.from_id < interval.from_id
                    {
                        from_index = Some(IndexToInsertRange::Between {
                            left_index: index - 1,
                            right_index: index,
                        });
                    }

                    if range_to_insert.to_id + 1 == interval.from_id {
                        to_index = Some(IndexToInsertRange::Exact(index))
                    } else if prev_interval.to_id < range_to_insert.to_id
                        && range_to_insert.to_id < interval.from_id
                    {
                        to_index = Some(IndexToInsertRange::Between {
                            left_index: index - 1,
                            right_index: index,
                        });
                    }
                }
                None => {
                    if range_to_insert.from_id < interval.from_id {
                        from_index = Some(IndexToInsertRange::First);
                    }

                    if range_to_insert.to_id + 1 == interval.from_id {
                        to_index = Some(IndexToInsertRange::Exact(0));
                        break;
                    } else {
                        if range_to_insert.to_id < interval.from_id {
                            to_index = Some(IndexToInsertRange::First);
                            break;
                        }
                    }
                }
            }

            if interval.is_in_my_interval(range_to_insert.from_id) {
                from_index = Some(IndexToInsertRange::Exact(index));
            }

            if interval.is_in_my_interval(range_to_insert.to_id) {
                to_index = Some(IndexToInsertRange::Exact(index));
            }

            prev_interval = Some(interval.clone());

            index += 1;
        }

        let to_index = match to_index {
            Some(to_index) => to_index,
            None => IndexToInsertRange::Last,
        };

        let from_index = match from_index {
            Some(from_index) => from_index,
            None => {
                if intervals.last().unwrap().to_id + 1 == range_to_insert.from_id {
                    IndexToInsertRange::Exact(intervals.len() - 1)
                } else {
                    IndexToInsertRange::Last
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
    pub fn unwrap_as_first(&self) {
        match self {
            Self::First => {}
            _ => panic!("{:?}", self),
        }
    }

    #[cfg(test)]
    pub fn unwrap_as_last(&self) {
        match self {
            Self::Last => {}
            _ => panic!("{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::QueueIndexRange;

    use super::IndexToInsertRange;

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(10, 15)];

        let new_interval = QueueIndexRange::restore(20, 25);

        // Checking if index_form and index_to are calculated ok
        let (index_from, index_to) = IndexToInsertRange::new(&intervals, &new_interval);
        index_from.unwrap_as_last();
        index_to.unwrap_as_last();
    }

    #[test]
    fn enqueue_range_case_to_the_end_of_the_list_with_merge() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(10, 15)];

        let new_interval = QueueIndexRange::restore(16, 25);

        // Checking if index_form and index_to are calculated ok
        let (index_from, index_to) = IndexToInsertRange::new(&intervals, &new_interval);
        assert_eq!(index_from.unwrap_as_exact(), 0);
        index_to.unwrap_as_last();
    }

    #[test]
    fn enqueue_range_at_the_beginning() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(15, 20)];

        let range_to_insert = QueueIndexRange::restore(5, 10);

        let (from_id, to_id) = IndexToInsertRange::new(&intervals, &range_to_insert);

        from_id.unwrap_as_first();
        to_id.unwrap_as_first();
    }

    #[test]
    fn enqueue_range_at_the_beginning_with_merge() {
        //Preparing data
        let intervals = vec![QueueIndexRange::restore(15, 20)];

        let range_to_insert = QueueIndexRange::restore(5, 14);

        let (from_id, to_id) = IndexToInsertRange::new(&intervals, &range_to_insert);

        from_id.unwrap_as_first();
        assert_eq!(to_id.unwrap_as_exact(), 0);
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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        from_index.unwrap_as_first();

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        from_index.unwrap_as_first();

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        from_index.unwrap_as_first();

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();

        assert_eq!(from_index.0, 2);
        assert_eq!(from_index.1, 3);

        to_index.unwrap_as_last();
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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        let from_index = from_index.unwrap_as_between();

        assert_eq!(from_index.0, 1);
        assert_eq!(from_index.1, 2);

        to_index.unwrap_as_last();
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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        from_index.unwrap_as_first();

        to_index.unwrap_as_last();
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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

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

        let (from_index, to_index) = IndexToInsertRange::new(&intervals, &range_to_insert);

        assert_eq!(from_index.unwrap_as_exact(), 1);

        to_index.unwrap_as_last();
    }

    #[test]
    fn test_index_to_insert_range() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
        ];

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(31, 32));

        assert_eq!(from_index.unwrap_as_exact(), 1);
        assert_eq!(to_index.unwrap_as_exact(), 1);
    }

    #[test]
    fn text_inserting_range_exactly_between() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(21, 29));

        assert_eq!(from_index.unwrap_as_exact(), 0);
        assert_eq!(to_index.unwrap_as_exact(), 1);

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(5, 9));

        from_index.unwrap_as_first();

        assert_eq!(to_index.unwrap_as_exact(), 0);

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(81, 85));

        assert_eq!(from_index.unwrap_as_exact(), 3);
        to_index.unwrap_as_last()
    }

    #[test]
    fn test_index_to_insert_range_3() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(25, 65));

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        let to_index = to_index.unwrap_as_between();
        assert_eq!(to_index.0, 2);
        assert_eq!(to_index.1, 3);

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(25, 70));

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        assert_eq!(to_index.unwrap_as_exact(), 3);

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(25, 45));

        let from_index = from_index.unwrap_as_between();
        assert_eq!(from_index.0, 0);
        assert_eq!(from_index.1, 1);

        let to_index = to_index.unwrap_as_between();
        assert_eq!(to_index.0, 1);
        assert_eq!(to_index.1, 2);

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(5, 85));

        from_index.unwrap_as_first();
        to_index.unwrap_as_last();
    }

    #[test]
    fn test_index_to_insert_range_at_and_after() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
        ];

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(31, 45));

        assert_eq!(from_index.unwrap_as_exact(), 1);
        to_index.unwrap_as_last();

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(5, 6));

        from_index.unwrap_as_first();
        to_index.unwrap_as_first();

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(45, 45));

        from_index.unwrap_as_last();
        to_index.unwrap_as_last();

        let (from_index, to_index) =
            IndexToInsertRange::new(&intervals, &QueueIndexRange::restore(5, 45));

        from_index.unwrap_as_first();
        to_index.unwrap_as_last();
    }
}
