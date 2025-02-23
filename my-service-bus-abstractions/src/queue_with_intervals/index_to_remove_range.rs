use super::{IndexRange, QueueIndexRange};

pub enum IndexToRemoveRange {
    NothingToDo,
    RemoveIntervals { from: usize, to: usize },
}

impl IndexToRemoveRange {
    pub fn new(intervals: &Vec<QueueIndexRange>, range_to_remove: &QueueIndexRange) -> Self {
        let (from_index, to_index) = IndexRange::new(intervals, range_to_remove);

        match from_index {
            IndexRange::Exact(_) => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::First => todo!(),
                IndexRange::Last => todo!(),
                IndexRange::Between {
                    left_index,
                    right_index,
                } => todo!(),
                IndexRange::JoinToIndexFrom(_) => todo!(),
                IndexRange::JoinToIndexTo(_) => todo!(),
                IndexRange::MergeIntervals(index) => {
                    todo!("Implement")
                }
            },
            IndexRange::First => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::First => {
                    return IndexToRemoveRange::NothingToDo;
                }
                IndexRange::Last => todo!(),
                IndexRange::Between {
                    left_index,
                    right_index: _,
                } => {
                    return Self::RemoveIntervals {
                        from: 0,
                        to: left_index,
                    }
                }
                IndexRange::JoinToIndexFrom(_) => todo!(),
                IndexRange::JoinToIndexTo(_) => todo!(),
                IndexRange::MergeIntervals(index) => {
                    todo!("Implement")
                }
            },
            IndexRange::Last => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::First => todo!(),
                IndexRange::Last => {
                    return IndexToRemoveRange::NothingToDo;
                }
                IndexRange::Between {
                    left_index,
                    right_index,
                } => todo!(),
                IndexRange::JoinToIndexFrom(_) => todo!(),
                IndexRange::JoinToIndexTo(_) => todo!(),
                IndexRange::MergeIntervals(index) => {
                    todo!("Implement")
                }
            },
            IndexRange::Between {
                left_index,
                right_index,
            } => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::First => {}
                IndexRange::Last => todo!(),
                IndexRange::Between {
                    left_index: to_left_index,
                    right_index: to_right_index,
                } => {
                    if left_index + 1 == right_index
                        && to_left_index + 1 == to_right_index
                        && left_index == to_left_index
                    {
                        return IndexToRemoveRange::NothingToDo;
                    }
                }
                IndexRange::JoinToIndexFrom(to_index) => {
                    if left_index + 1 == to_index {
                        return IndexToRemoveRange::NothingToDo;
                    }
                }
                IndexRange::JoinToIndexTo(_) => todo!(),
                IndexRange::MergeIntervals(index) => {
                    todo!("Implement")
                }
            },
            IndexRange::JoinToIndexFrom(from_index) => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::JoinToIndexFrom(to_index) => todo!(),
                IndexRange::JoinToIndexTo(to_index) => {
                    todo!()
                }
                IndexRange::First => todo!(),
                IndexRange::Last => todo!(),
                IndexRange::Between {
                    left_index,
                    right_index,
                } => todo!(),
                IndexRange::MergeIntervals(index) => {
                    todo!("Implement")
                }
            },
            IndexRange::JoinToIndexTo(from_index) => match to_index {
                IndexRange::Exact(_) => todo!(),
                IndexRange::JoinToIndexFrom(to_index) => {
                    if from_index + 1 == to_index {
                        return IndexToRemoveRange::NothingToDo;
                    }
                }
                IndexRange::JoinToIndexTo(_) => todo!(),
                IndexRange::First => todo!(),
                IndexRange::Last => {
                    if from_index == intervals.len() - 1 {
                        return IndexToRemoveRange::NothingToDo;
                    }
                }
                IndexRange::Between {
                    left_index,
                    right_index,
                } => todo!(),
                IndexRange::MergeIntervals(index) => {
                    todo!("Implement")
                }
            },
            IndexRange::MergeIntervals(index) => {
                todo!("Implement")
            }
        }

        todo!(
            "Should not be here. Intervals: {:?}. range: {:?}",
            intervals,
            range_to_remove
        );
    }

    pub fn is_nothing_to_do(&self) -> bool {
        match self {
            IndexToRemoveRange::NothingToDo => true,
            _ => false,
        }
    }

    pub fn unwrap_as_remove_intervals(&self) -> Option<(usize, usize)> {
        match self {
            IndexToRemoveRange::RemoveIntervals { from, to } => (*from, *to).into(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::queue_with_intervals::QueueIndexRange;

    use super::IndexToRemoveRange;

    #[test]
    fn test_all_cases_do_nothing() {
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_remove = QueueIndexRange::restore(5, 7);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        assert_eq!(index.is_nothing_to_do(), true);

        let range_to_remove = QueueIndexRange::restore(21, 29);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        assert_eq!(index.is_nothing_to_do(), true);

        let range_to_remove = QueueIndexRange::restore(42, 48);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        assert_eq!(index.is_nothing_to_do(), true);

        let range_to_remove = QueueIndexRange::restore(42, 49);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        assert_eq!(index.is_nothing_to_do(), true);

        let range_to_remove = QueueIndexRange::restore(81, 85);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        assert_eq!(index.is_nothing_to_do(), true);

        let range_to_remove = QueueIndexRange::restore(82, 85);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        assert_eq!(index.is_nothing_to_do(), true);
    }

    #[test]
    fn test_all_cases_we_go_between_intervals() {
        let a = 5;

        /*
        let intervals = vec![
            QueueIndexRange::restore(10, 20),
            QueueIndexRange::restore(30, 40),
            QueueIndexRange::restore(50, 60),
            QueueIndexRange::restore(70, 80),
        ];

        let range_to_remove = QueueIndexRange::restore(5, 25);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        let index = index.unwrap_as_remove_intervals().unwrap();
        assert_eq!(index.0, 0);
        assert_eq!(index.0, 0);

        let range_to_remove = QueueIndexRange::restore(9, 25);
        let index = IndexToRemoveRange::new(&intervals, &range_to_remove);
        let index = index.unwrap_as_remove_intervals().unwrap();
        assert_eq!(index.0, 0);
        assert_eq!(index.0, 0);
         */
    }
}
