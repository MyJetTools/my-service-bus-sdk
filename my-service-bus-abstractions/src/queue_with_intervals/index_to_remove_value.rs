use super::QueueIndexRange;
#[derive(Debug)]
pub enum IndexToRemoveValue {
    IncLeft(usize),
    DecRight(usize),
    Split {
        index: usize,
        left: QueueIndexRange,
        right: QueueIndexRange,
    },
    Remove(usize),
    NoValue,
}

impl IndexToRemoveValue {
    pub fn new(intervals: &Vec<QueueIndexRange>, value: i64) -> Self {
        let mut index = 0;
        for interval in intervals {
            if interval.from_id == value {
                if interval.to_id == value {
                    return Self::Remove(index);
                }
                return Self::IncLeft(index);
            }

            if interval.to_id == value {
                return Self::DecRight(index);
            }

            if interval.from_id < value && value < interval.to_id {
                return Self::Split {
                    index,
                    left: QueueIndexRange {
                        from_id: interval.from_id,
                        to_id: value - 1,
                    },
                    right: QueueIndexRange {
                        from_id: value + 1,
                        to_id: interval.to_id,
                    },
                };
            }

            index += 1;
        }

        Self::NoValue
    }

    pub fn is_no_value(&self) -> bool {
        match self {
            IndexToRemoveValue::NoValue => true,
            _ => false,
        }
    }

    pub fn unwrap_as_inc_left(&self) -> usize {
        match self {
            IndexToRemoveValue::IncLeft(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_as_dec_right(&self) -> usize {
        match self {
            IndexToRemoveValue::DecRight(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_as_remove(&self) -> usize {
        match self {
            IndexToRemoveValue::Remove(index) => *index,
            _ => panic!("{:?}", self),
        }
    }

    pub fn unwrap_as_split(&self) -> (usize, &QueueIndexRange, &QueueIndexRange) {
        match self {
            IndexToRemoveValue::Split { index, left, right } => (*index, left, right),
            _ => panic!("{:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::{IndexToRemoveValue, QueueIndexRange};

    #[test]
    fn test_options() {
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
            QueueIndexRange {
                from_id: 55,
                to_id: 55,
            },
        ];

        assert_eq!(IndexToRemoveValue::new(&intervals, 5).is_no_value(), true);

        assert_eq!(
            IndexToRemoveValue::new(&intervals, 10).unwrap_as_inc_left(),
            0
        );

        let result = IndexToRemoveValue::new(&intervals, 11);
        let result = result.unwrap_as_split();
        assert_eq!(result.0, 0);
        assert_eq!(result.1.from_id, 10);
        assert_eq!(result.1.to_id, 10);

        assert_eq!(result.2.from_id, 12);
        assert_eq!(result.2.to_id, 20);

        assert_eq!(
            IndexToRemoveValue::new(&intervals, 20).unwrap_as_dec_right(),
            0
        );

        assert_eq!(
            IndexToRemoveValue::new(&intervals, 55).unwrap_as_remove(),
            3
        );
    }
}
