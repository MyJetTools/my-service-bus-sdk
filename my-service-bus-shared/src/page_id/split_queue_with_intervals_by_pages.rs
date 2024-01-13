use my_service_bus_abstractions::queue_with_intervals::{QueueIndexRange, QueueWithIntervals};

use super::PageId;

pub struct SplittedByPageId {
    pub page_id: PageId,
    pub ids: QueueWithIntervals,
}

pub struct SplittedByPageIdIterator {
    intervals: Vec<QueueIndexRange>,
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
        let el = self.intervals.get_mut(self.index)?;

        if el.is_empty() {
            return None;
        }

        let mut ids = QueueWithIntervals::new();
        let mut page_id = None;

        while let Some(el) = self.intervals.get_mut(self.index) {
            if page_id.is_none() {
                page_id = Some(PageId::from_message_id(el.from_id.into()));
            }

            let page_id = page_id.unwrap();

            let from_page_id = PageId::from_message_id(el.from_id.into());

            if from_page_id.get_value() > page_id.get_value() {
                return Some(SplittedByPageId { page_id, ids });
            }

            let to_page_id = PageId::from_message_id(el.to_id.into());

            if to_page_id.get_value() > page_id.get_value() {
                let to_id = page_id.get_last_message_id();

                ids.push_interval(QueueIndexRange {
                    from_id: el.from_id,
                    to_id: to_id.get_value(),
                });

                el.from_id = to_id.get_value() + 1;

                return Some(SplittedByPageId { page_id, ids });
            }

            ids.push_interval(QueueIndexRange {
                from_id: el.from_id,
                to_id: el.to_id,
            });

            self.index += 1;
        }

        return Some(SplittedByPageId {
            page_id: page_id.unwrap(),
            ids,
        });
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

        src.push_interval(QueueIndexRange {
            from_id: 100_010,
            to_id: 100_020,
        });

        src.push_interval(QueueIndexRange {
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
}
