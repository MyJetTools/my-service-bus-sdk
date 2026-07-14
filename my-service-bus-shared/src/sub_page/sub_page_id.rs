use my_service_bus_abstractions::MessageId;

use crate::page_id::PageId;

pub const SUB_PAGE_MESSAGES_AMOUNT: i64 = 1000;
pub const SUB_PAGES_PER_PAGE: i64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct SubPageId(i64);

impl SubPageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }
    pub fn from_message_id(message_id: MessageId) -> Self {
        Self(message_id.get_value() / SUB_PAGE_MESSAGES_AMOUNT)
    }

    pub fn from_page_id(page_id: PageId) -> Self {
        Self(page_id.get_value() * SUB_PAGES_PER_PAGE)
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn get_first_message_id(&self) -> MessageId {
        let result = self.get_value() * SUB_PAGE_MESSAGES_AMOUNT;
        result.into()
    }

    pub fn get_last_message_id(&self) -> MessageId {
        let result = self.get_first_message_id_of_next_sub_page().get_value() - 1;
        result.into()
    }

    pub fn get_first_message_id_of_next_sub_page(&self) -> MessageId {
        let result = self.get_first_message_id().get_value() + SUB_PAGE_MESSAGES_AMOUNT;
        result.into()
    }

    pub fn iterate_message_ids(&self) -> std::ops::Range<i64> {
        let first_message_id = self.get_first_message_id().get_value();
        first_message_id..first_message_id + SUB_PAGE_MESSAGES_AMOUNT
    }

    pub fn is_my_message_id(&self, message_id: MessageId) -> bool {
        let first_message_id = self.get_first_message_id().get_value();
        let last_message_id = self.get_last_message_id().get_value();

        let message_id = message_id.get_value();

        message_id >= first_message_id && message_id <= last_message_id
    }
}

impl std::fmt::Display for SubPageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<i64> for SubPageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Into<SubPageId> for MessageId {
    fn into(self) -> SubPageId {
        SubPageId::from_message_id(self)
    }
}

impl Into<SubPageId> for PageId {
    fn into(self) -> SubPageId {
        SubPageId::from_page_id(self)
    }
}

pub trait AsSubPageId {
    fn as_sub_page_id(&self) -> SubPageId;
}

impl AsSubPageId for i64 {
    fn as_sub_page_id(&self) -> SubPageId {
        SubPageId::new(*self)
    }
}

#[cfg(test)]
mod test {
    use crate::{page_id::PageId, sub_page::*};

    #[test]
    fn test_first_message_id() {
        assert_eq!(0, SubPageId::new(0).get_first_message_id().get_value());
        assert_eq!(1000, SubPageId::new(1).get_first_message_id().get_value());
        assert_eq!(2000, SubPageId::new(2).get_first_message_id().get_value());
    }

    #[test]
    fn test_first_message_id_of_next_page() {
        assert_eq!(
            1000,
            SubPageId::new(0)
                .get_first_message_id_of_next_sub_page()
                .get_value()
        );
        assert_eq!(
            2000,
            SubPageId::new(1)
                .get_first_message_id_of_next_sub_page()
                .get_value()
        );
        assert_eq!(
            3000,
            SubPageId::new(2)
                .get_first_message_id_of_next_sub_page()
                .get_value()
        );
    }

    #[test]
    fn test_creating_from_page_id() {
        assert_eq!(0, SubPageId::from_page_id(PageId::new(0)).get_value());

        assert_eq!(100, SubPageId::from_page_id(PageId::new(1)).get_value());
        assert_eq!(200, SubPageId::from_page_id(PageId::new(2)).get_value());

        //Made cross check from MessageID and From PageID
        let message_id = 100_000.into();
        let page_id = PageId::from_message_id(message_id);

        assert_eq!(
            SubPageId::from_page_id(page_id).get_value(),
            SubPageId::from_message_id(message_id).get_value()
        );
    }
}

#[cfg(test)]
mod audit_conversions {
    // Audit of message_id -> sub_page -> page conversions used by the broker
    // restore-loop. Verifies invariants 1-5 and the exact page boundaries so an
    // off-by-one can never let a restore request miss part of a sub-page.
    use crate::page_id::{PageId, MESSAGES_IN_PAGE};
    use crate::sub_page::{SubPageId, SUB_PAGES_PER_PAGE, SUB_PAGE_MESSAGES_AMOUNT};
    use my_service_bus_abstractions::MessageId;

    // PAGE must be an exact multiple of SUB, otherwise a sub-page could straddle
    // two 100k pages and the single page_no sent to persistence would be wrong.
    #[test]
    fn constants_are_consistent() {
        assert_eq!(SUB_PAGE_MESSAGES_AMOUNT, 1000);
        assert_eq!(SUB_PAGES_PER_PAGE, 100);
        assert_eq!(MESSAGES_IN_PAGE, 100_000);
        assert_eq!(
            MESSAGES_IN_PAGE,
            SUB_PAGE_MESSAGES_AMOUNT * SUB_PAGES_PER_PAGE
        );
        assert_eq!(MESSAGES_IN_PAGE % SUB_PAGE_MESSAGES_AMOUNT, 0);
    }

    // Checks every invariant for one concrete message id M.
    fn assert_message_id_invariants(
        m: i64,
        expected_sub: i64,
        expected_first: i64,
        expected_last: i64,
        expected_page: i64,
    ) {
        let message_id: MessageId = m.into();
        let sub_page = SubPageId::from_message_id(message_id);

        // Invariant 1: SubPageId::from(M).get_value() == M / SUB
        assert_eq!(sub_page.get_value(), expected_sub, "sub_page for M={}", m);
        assert_eq!(
            sub_page.get_value(),
            m / SUB_PAGE_MESSAGES_AMOUNT,
            "sub == M/SUB for M={}",
            m
        );

        // The `.into()` path (used by the node) must agree with from_message_id.
        let via_into: SubPageId = message_id.into();
        assert_eq!(via_into.get_value(), expected_sub);

        // Invariant 2: first == sub*SUB, last == sub*SUB + (SUB-1)
        assert_eq!(
            sub_page.get_first_message_id().get_value(),
            expected_first,
            "first for M={}",
            m
        );
        assert_eq!(
            sub_page.get_last_message_id().get_value(),
            expected_last,
            "last for M={}",
            m
        );
        assert_eq!(
            sub_page.get_first_message_id().get_value(),
            expected_sub * SUB_PAGE_MESSAGES_AMOUNT
        );
        assert_eq!(
            sub_page.get_last_message_id().get_value(),
            expected_sub * SUB_PAGE_MESSAGES_AMOUNT + (SUB_PAGE_MESSAGES_AMOUNT - 1)
        );

        // The probed M must fall inside its own [first..last].
        assert!(sub_page.is_my_message_id(message_id), "M={} in range", m);

        // Invariant 4: PageId::from(SubPageId) == first/PAGE == sub/(PAGE/SUB)
        let page: PageId = sub_page.into();
        assert_eq!(page.get_value(), expected_page, "page for M={}", m);
        assert_eq!(page.get_value(), expected_first / MESSAGES_IN_PAGE);
        assert_eq!(page.get_value(), expected_sub / SUB_PAGES_PER_PAGE);

        // Invariant 5 / consistency: from and to land in the SAME 100k page, so the
        // node's single page_no covers the whole sub-page range [from..to].
        assert_eq!(
            expected_first / MESSAGES_IN_PAGE,
            expected_last / MESSAGES_IN_PAGE,
            "from/to share page for M={}",
            m
        );
        assert_eq!(
            PageId::from_message_id(expected_first.into()).get_value(),
            PageId::from_message_id(expected_last.into()).get_value()
        );
    }

    // Boundary cases enumerated in the audit (point 5).
    #[test]
    fn boundary_cases() {
        //                            M            sub      first        last         page
        assert_message_id_invariants(0, 0, 0, 999, 0);
        assert_message_id_invariants(999, 0, 0, 999, 0);
        assert_message_id_invariants(1000, 1, 1000, 1999, 0);
        assert_message_id_invariants(99_999, 99, 99_000, 99_999, 0);
        assert_message_id_invariants(100_000, 100, 100_000, 100_999, 1);
        assert_message_id_invariants(356_237_000, 356_237, 356_237_000, 356_237_999, 3562);
    }

    // Invariant 3: iterate_message_ids() yields exactly [first..last], no gaps, len == SUB.
    #[test]
    fn iterate_message_ids_covers_full_sub_page() {
        for sub in [0_i64, 1, 99, 100, 356_237] {
            let sub_page = SubPageId::new(sub);
            let first = sub_page.get_first_message_id().get_value();
            let last = sub_page.get_last_message_id().get_value();

            let ids: Vec<i64> = sub_page.iterate_message_ids().collect();

            assert_eq!(
                ids.len() as i64,
                SUB_PAGE_MESSAGES_AMOUNT,
                "length for sub={}",
                sub
            );
            assert_eq!(*ids.first().unwrap(), first, "first id for sub={}", sub);
            assert_eq!(*ids.last().unwrap(), last, "last id for sub={}", sub);

            // strictly contiguous: no gaps, no overshoot past `last`.
            for (i, id) in ids.iter().enumerate() {
                assert_eq!(*id, first + i as i64);
            }
        }
    }

    // The exact restore range the broker probe relies on:
    // trade topic, sub_page 356220 -> 356220000..356220999 (1000 messages, page 3562).
    #[test]
    fn restore_range_matches_broker_probe() {
        let sub_page = SubPageId::new(356_220);
        assert_eq!(sub_page.get_first_message_id().get_value(), 356_220_000);
        assert_eq!(sub_page.get_last_message_id().get_value(), 356_220_999);

        let ids: Vec<i64> = sub_page.iterate_message_ids().collect();
        assert_eq!(ids.len(), 1000);

        let page: PageId = sub_page.into();
        assert_eq!(page.get_value(), 3562);
    }

    // A sub-page must never cross a 100k page boundary: sub 99 is the last sub-page
    // of page 0, sub 100 is the first of page 1, and the ids butt up exactly.
    #[test]
    fn sub_page_does_not_cross_page_boundary() {
        let last_in_page_0 = SubPageId::new(SUB_PAGES_PER_PAGE - 1);
        let p0: PageId = last_in_page_0.into();
        assert_eq!(p0.get_value(), 0);
        assert_eq!(
            last_in_page_0.get_last_message_id().get_value(),
            MESSAGES_IN_PAGE - 1
        );

        let first_in_page_1 = SubPageId::new(SUB_PAGES_PER_PAGE);
        let p1: PageId = first_in_page_1.into();
        assert_eq!(p1.get_value(), 1);
        assert_eq!(
            first_in_page_1.get_first_message_id().get_value(),
            MESSAGES_IN_PAGE
        );

        // No gap and no overlap between the two adjacent sub-pages.
        assert_eq!(
            last_in_page_0.get_last_message_id().get_value() + 1,
            first_in_page_1.get_first_message_id().get_value()
        );
    }

    // from_page_id (node uses it as page -> first sub-page of that page) must be the
    // inverse of SubPageId -> PageId at the page's first sub-page.
    #[test]
    fn page_id_round_trip() {
        for page_no in [0_i64, 1, 2, 3562] {
            let page = PageId::new(page_no);
            let first_sub_page = SubPageId::from_page_id(page);
            assert_eq!(first_sub_page.get_value(), page_no * SUB_PAGES_PER_PAGE);

            // back to page
            let back: PageId = first_sub_page.into();
            assert_eq!(back.get_value(), page_no);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SubPageId;

    #[test]
    fn test_b_tree_map() {
        let mut map = std::collections::BTreeMap::new();

        map.insert(SubPageId::new(1), 1);
        map.insert(SubPageId::new(2), 2);
        map.insert(SubPageId::new(4), 4);
        map.insert(SubPageId::new(3), 3);

        assert_eq!(1, map[&SubPageId::new(1)]);
        assert_eq!(2, map[&SubPageId::new(2)]);
        assert_eq!(3, map[&SubPageId::new(3)]);
        assert_eq!(4, map[&SubPageId::new(4)]);
    }

    #[test]
    fn test_hash_map() {
        let mut map = std::collections::HashMap::new();

        map.insert(SubPageId::new(1), 1);
        map.insert(SubPageId::new(2), 2);
        map.insert(SubPageId::new(4), 4);
        map.insert(SubPageId::new(3), 3);

        assert_eq!(1, map[&SubPageId::new(1)]);
        assert_eq!(2, map[&SubPageId::new(2)]);
        assert_eq!(3, map[&SubPageId::new(3)]);
        assert_eq!(4, map[&SubPageId::new(4)]);
    }

    #[test]
    fn test_my_message_id() {
        let sub_page = SubPageId::new(0);

        assert!(sub_page.is_my_message_id(0.into()));
        assert!(sub_page.is_my_message_id(999.into()));
        assert!(!sub_page.is_my_message_id(1000.into()));

        let sub_page = SubPageId::new(1);
        assert!(sub_page.is_my_message_id(1000.into()));
        assert!(sub_page.is_my_message_id(1999.into()));
        assert!(!sub_page.is_my_message_id(2000.into()));
    }

    #[test]
    fn test_first_message_id_of_the_next_page() {
        let sub_page = SubPageId::new(0);

        assert_eq!(
            1000,
            sub_page.get_first_message_id_of_next_sub_page().get_value()
        );

        let sub_page = SubPageId::new(1);

        assert_eq!(
            2000,
            sub_page.get_first_message_id_of_next_sub_page().get_value()
        );

        let sub_page = SubPageId::new(2);

        assert_eq!(
            3000,
            sub_page.get_first_message_id_of_next_sub_page().get_value()
        );
    }
}
