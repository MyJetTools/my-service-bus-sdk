use my_service_bus_abstractions::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

use super::PageId;

pub struct SplittedByPageId {
    pub page_id: PageId,
    pub ids: QueueWithIntervals,
}

pub struct SplittedByPageIdIterator {
    intervals: Vec<QueueIndexRange<i64>>,
    index: usize,
}

impl SplittedByPageIdIterator {
    pub fn new(src: &QueueWithIntervals) -> Self {
        Self {
            intervals: src.get_snapshot(),
            index: 0,
        }
    }
}

impl Iterator for SplittedByPageIdIterator {
    type Item = SplittedByPageId;

    fn next(&mut self) -> Option<Self::Item> {
        // Skip empties.
        while self.index < self.intervals.len() && self.intervals[self.index].is_empty() {
            self.index += 1;
        }

        if self.index >= self.intervals.len() {
            return None;
        }

        let page_id = PageId::from_message_id(self.intervals[self.index].from_id.into());
        let mut ids = QueueWithIntervals::new();

        while self.index < self.intervals.len() {
            let el = &mut self.intervals[self.index];

            if el.is_empty() {
                self.index += 1;
                continue;
            }

            let current_page = PageId::from_message_id(el.from_id.into());
            if current_page.get_value() > page_id.get_value() {
                break;
            }

            let page_last_id = page_id.get_last_message_id().get_value();
            let chunk_to = el.to_id.min(page_last_id);

            ids.enqueue_range(QueueIndexRange {
                from_id: el.from_id,
                to_id: chunk_to,
            });

            if chunk_to == el.to_id {
                // interval fully consumed
                self.index += 1;
            } else {
                // interval continues on the next page
                el.from_id = chunk_to + 1;
                break;
            }
        }

        Some(SplittedByPageId { page_id, ids })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_both_on_the_same_page() {
        let src = QueueWithIntervals::from_single_interval(100, 200);

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(1, result.len());
        assert_eq!(0, result[0].page_id.get_value());

        assert_eq!(100, result[0].ids.get_interval(0).unwrap().from_id);
        assert_eq!(200, result[0].ids.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn test_we_are_jumping_behind_the_page() {
        let src = QueueWithIntervals::from_single_interval(99998, 100002);

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(2, result.len());
        assert_eq!(0, result[0].page_id.get_value());
        assert_eq!(1, result[1].page_id.get_value());

        assert_eq!(99998, result[0].ids.get_interval(0).unwrap().from_id);
        assert_eq!(99999, result[0].ids.get_interval(0).unwrap().to_id);

        assert_eq!(100000, result[1].ids.get_interval(0).unwrap().from_id);
        assert_eq!(100002, result[1].ids.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn test_we_are_jumping_behind_the_page_2() {
        let mut src = QueueWithIntervals::from_single_interval(99_998, 100_002);

        src.enqueue_range(QueueIndexRange {
            from_id: 100_010,
            to_id: 100_020,
        });

        src.enqueue_range(QueueIndexRange {
            from_id: 199_990,
            to_id: 200_020,
        });

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(3, result.len());
        assert_eq!(0, result[0].page_id.get_value());
        assert_eq!(1, result[1].page_id.get_value());
        assert_eq!(2, result[2].page_id.get_value());

        assert_eq!(99_998, result[0].ids.get_interval(0).unwrap().from_id);
        assert_eq!(99_999, result[0].ids.get_interval(0).unwrap().to_id);

        assert_eq!(100_000, result[1].ids.get_interval(0).unwrap().from_id);
        assert_eq!(100_002, result[1].ids.get_interval(0).unwrap().to_id);

        assert_eq!(100_010, result[1].ids.get_interval(1).unwrap().from_id);
        assert_eq!(100_020, result[1].ids.get_interval(1).unwrap().to_id);

        assert_eq!(199_990, result[1].ids.get_interval(2).unwrap().from_id);
        assert_eq!(199_999, result[1].ids.get_interval(2).unwrap().to_id);

        assert_eq!(200_000, result[2].ids.get_interval(0).unwrap().from_id);
        assert_eq!(200_020, result[2].ids.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn test_leading_empty_interval_is_skipped() {
        let mut src = QueueWithIntervals::new();
        // Empty interval (from > to) should be ignored.
        src.enqueue_range(QueueIndexRange {
            from_id: 10,
            to_id: 9,
        });
        src.enqueue_range(QueueIndexRange {
            from_id: 100,
            to_id: 110,
        });

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(1, result.len());
        assert_eq!(0, result[0].page_id.get_value());
        assert_eq!(100, result[0].ids.get_interval(0).unwrap().from_id);
        assert_eq!(110, result[0].ids.get_interval(0).unwrap().to_id);
    }

    #[test]
    fn test_middle_empty_interval_is_skipped() {
        let mut src = QueueWithIntervals::new();
        src.enqueue_range(QueueIndexRange {
            from_id: 50,
            to_id: 60,
        });
        // Empty interval (from > to) in the middle.
        src.enqueue_range(QueueIndexRange {
            from_id: 20,
            to_id: 10,
        });
        src.enqueue_range(QueueIndexRange {
            from_id: 70,
            to_id: 72,
        });

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(1, result.len());
        assert_eq!(0, result[0].page_id.get_value());
        assert_eq!(50, result[0].ids.get_interval(0).unwrap().from_id);
        assert_eq!(60, result[0].ids.get_interval(0).unwrap().to_id);
        assert_eq!(70, result[0].ids.get_interval(1).unwrap().from_id);
        assert_eq!(72, result[0].ids.get_interval(1).unwrap().to_id);
    }

    #[test]
    fn test_all_intervals_empty_results_in_none() {
        let mut src = QueueWithIntervals::new();
        src.enqueue_range(QueueIndexRange {
            from_id: 5,
            to_id: 4,
        });
        src.enqueue_range(QueueIndexRange {
            from_id: 9,
            to_id: 8,
        });

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        assert_eq!(0, result.len());
    }

    #[test]
    fn test_empty_remainder_after_split_is_skipped() {
        // Crosses page boundary at exactly the page end; remainder becomes empty and must be skipped.
        let mut src = QueueWithIntervals::from_single_interval(99_998, 100_000);
        // Add another interval to ensure we continue past the emptied remainder.
        src.enqueue_range(QueueIndexRange {
            from_id: 200_000,
            to_id: 200_001,
        });

        let result: Vec<SplittedByPageId> = SplittedByPageIdIterator::new(&src).collect();

        // Page 0 chunk
        assert_eq!(0, result[0].page_id.get_value());
        assert_eq!(99_998, result[0].ids.get_interval(0).unwrap().from_id);
        assert_eq!(99_999, result[0].ids.get_interval(0).unwrap().to_id);

        // Remainder on next page
        assert_eq!(1, result[1].page_id.get_value());
        assert_eq!(100_000, result[1].ids.get_interval(0).unwrap().from_id);
        assert_eq!(100_000, result[1].ids.get_interval(0).unwrap().to_id);

        // Next non-empty interval still emitted
        assert_eq!(2, result[2].page_id.get_value());
        assert_eq!(200_000, result[2].ids.get_interval(0).unwrap().from_id);
        assert_eq!(200_001, result[2].ids.get_interval(0).unwrap().to_id);
    }
}
