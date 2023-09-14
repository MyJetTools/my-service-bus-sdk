use crate::sub_page::SubPageId;

pub struct SubPagesIterator {
    pub now_id: i64,
    pub to_id: i64,
}

impl Iterator for SubPagesIterator {
    type Item = SubPageId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.now_id >= self.to_id {
            return None;
        }

        let result = Self::Item::new(self.now_id);

        self.now_id += 1;

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::{page_id::PageId, sub_page::SUB_PAGES_PER_PAGE};

    #[test]
    fn test_iterator_with_page_0() {
        let page_id = PageId::new(0);

        let mut amount = 0;

        let mut test_sub_page_id = 0;

        for sub_page_id in page_id.iterate_sub_page_ids() {
            assert_eq!(test_sub_page_id, sub_page_id.get_value());

            amount += 1;
            test_sub_page_id += 1;
        }

        assert_eq!(amount, SUB_PAGES_PER_PAGE);
    }

    #[test]
    fn test_iterator_with_page_1() {
        let page_id = PageId::new(1);

        let mut amount = 0;

        let mut test_sub_page_id = SUB_PAGES_PER_PAGE;

        for sub_page_id in page_id.iterate_sub_page_ids() {
            assert_eq!(test_sub_page_id, sub_page_id.get_value());

            amount += 1;
            test_sub_page_id += 1;
        }

        assert_eq!(amount, SUB_PAGES_PER_PAGE);
    }

    #[test]
    fn test_iterator_with_page_2() {
        let page_id = PageId::new(2);

        let mut amount = 0;

        let mut test_sub_page_id = SUB_PAGES_PER_PAGE * 2;

        for sub_page_id in page_id.iterate_sub_page_ids() {
            assert_eq!(test_sub_page_id, sub_page_id.get_value());

            amount += 1;
            test_sub_page_id += 1;
        }

        assert_eq!(amount, SUB_PAGES_PER_PAGE);
    }
}
