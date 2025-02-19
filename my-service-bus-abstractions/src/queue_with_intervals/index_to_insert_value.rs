use super::QueueIndexRange;

#[derive(Debug)]
pub enum IndexToInsertValue {
    MergeToLeft(usize),
    MergeToRight(usize),
    InsertAsNewInterval(usize),
    MergeTwoIntervals(usize),
    HasValue,
}

impl IndexToInsertValue {
    pub fn new(intervals: &Vec<QueueIndexRange>, value: i64) -> Self {
        let result = Self::detect_interval(intervals, value);

        match &result {
            IndexToInsertValue::MergeToLeft(index) => {
                let index = *index;
                if index > 0 {
                    let left_interval = intervals.get(index - 1).unwrap();
                    let right_interval = intervals.get(index).unwrap();

                    if left_interval.to_id + 1 == value && value + 1 == right_interval.from_id {
                        return Self::MergeTwoIntervals(index - 1);
                    }
                }
            }
            IndexToInsertValue::MergeToRight(index) => {
                let index = *index;
                if index < intervals.len() - 1 {
                    let left_interval = intervals.get(index).unwrap();
                    let right_interval = intervals.get(index + 1).unwrap();

                    if left_interval.to_id + 1 == value && value + 1 == right_interval.from_id {
                        return Self::MergeTwoIntervals(index);
                    }
                }
            }
            IndexToInsertValue::InsertAsNewInterval(_) => {}
            IndexToInsertValue::MergeTwoIntervals(_) => {}
            IndexToInsertValue::HasValue => {}
        }

        result
    }

    fn detect_interval(intervals: &[QueueIndexRange], value: i64) -> Self {
        let mut prev_element: Option<&QueueIndexRange> = None;

        let mut index = 0;
        for itm in intervals {
            if itm.from_id <= value && value <= itm.to_id {
                return Self::HasValue;
            }

            if value + 1 == itm.from_id {
                return Self::MergeToLeft(index);
            }

            if itm.to_id + 1 == value {
                return Self::MergeToRight(index);
            }

            if let Some(prev_element) = prev_element {
                if prev_element.to_id < value && value < itm.from_id {
                    return Self::InsertAsNewInterval(index);
                }
            } else {
                if value < itm.from_id {
                    return Self::InsertAsNewInterval(0);
                }
            }

            prev_element = Some(itm);

            index += 1;
        }

        Self::InsertAsNewInterval(intervals.len())
    }

    pub fn unwrap_as_merge_to_right(&self) -> usize {
        match self {
            IndexToInsertValue::MergeToRight(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_as_merge_to_left(&self) -> usize {
        match self {
            IndexToInsertValue::MergeToLeft(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_as_merge_two_intervals(&self) -> usize {
        match self {
            IndexToInsertValue::MergeTwoIntervals(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_insert_as_new_interval(&self) -> usize {
        match self {
            IndexToInsertValue::InsertAsNewInterval(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_as_has_value(&self) {
        match self {
            IndexToInsertValue::HasValue => {}
            _ => panic!("{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::{IndexToInsertValue, QueueIndexRange};

    #[test]
    fn test_intervals() {
        let intervals = vec![
            QueueIndexRange {
                from_id: 10,
                to_id: 20,
            },
            QueueIndexRange {
                from_id: 22,
                to_id: 30,
            },
            QueueIndexRange {
                from_id: 40,
                to_id: 50,
            },
        ];

        assert_eq!(
            IndexToInsertValue::new(&intervals, 5).unwrap_insert_as_new_interval(),
            0
        );

        assert_eq!(
            IndexToInsertValue::new(&intervals, 9).unwrap_as_merge_to_left(),
            0
        );

        IndexToInsertValue::new(&intervals, 10).unwrap_as_has_value();

        assert_eq!(
            IndexToInsertValue::new(&intervals, 21).unwrap_as_merge_two_intervals(),
            0
        );

        assert_eq!(
            IndexToInsertValue::new(&intervals, 31).unwrap_as_merge_to_right(),
            1
        );

        assert_eq!(
            IndexToInsertValue::new(&intervals, 32).unwrap_insert_as_new_interval(),
            2
        );

        assert_eq!(
            IndexToInsertValue::new(&intervals, 39).unwrap_as_merge_to_left(),
            2
        );

        IndexToInsertValue::new(&intervals, 40).unwrap_as_has_value();

        assert_eq!(
            IndexToInsertValue::new(&intervals, 51).unwrap_as_merge_to_right(),
            2
        );

        assert_eq!(
            IndexToInsertValue::new(&intervals, 52).unwrap_insert_as_new_interval(),
            3
        );
    }
}
