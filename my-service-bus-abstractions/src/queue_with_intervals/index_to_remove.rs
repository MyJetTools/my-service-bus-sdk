use super::QueueIndexRange;

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
}
