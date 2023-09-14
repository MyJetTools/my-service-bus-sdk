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
