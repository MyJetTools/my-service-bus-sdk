use std::fmt::Display;

use my_service_bus_abstractions::MessageId;

use crate::sub_page::SubPageId;

use super::{PageIdIterator, SubPagesIterator};

pub const MESSAGES_IN_PAGE: i64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct PageId(i64);

impl PageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }
    pub fn from_message_id(message_id: MessageId) -> Self {
        let result = message_id.get_value() / MESSAGES_IN_PAGE;
        Self(result.into())
    }

    pub fn get_first_message_id(&self) -> MessageId {
        let result = self.0 * MESSAGES_IN_PAGE;
        result.into()
    }

    pub fn get_last_message_id(&self) -> MessageId {
        let result = (self.0 + 1) * MESSAGES_IN_PAGE - 1;
        result.into()
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn iterate_messages(&self) -> PageIdIterator {
        return PageIdIterator::new(*self);
    }

    pub fn iterate_sub_page_ids(&self) -> SubPagesIterator {
        let message_id = self.get_first_message_id();
        let first_sub_page_id = SubPageId::from_message_id(message_id);

        SubPagesIterator {
            now_id: first_sub_page_id.get_value(),
            to_id: first_sub_page_id.get_value() + crate::sub_page::SUB_PAGES_PER_PAGE,
        }
    }
}

impl Into<PageId> for MessageId {
    fn into(self) -> PageId {
        PageId::from_message_id(self)
    }
}

impl Into<PageId> for SubPageId {
    fn into(self) -> PageId {
        PageId::from_message_id(self.get_first_message_id())
    }
}

impl AsRef<i64> for PageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl std::ops::Sub<i64> for PageId {
    type Output = Self;

    fn sub(self, rhs: i64) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl std::ops::Add<i64> for PageId {
    type Output = Self;

    fn add(self, rhs: i64) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Into<i64> for PageId {
    fn into(self) -> i64 {
        self.0
    }
}

impl<'s> Into<i64> for &'s PageId {
    fn into(self) -> i64 {
        self.0
    }
}

pub trait AsPageId {
    fn as_page_id(&self) -> PageId;
}

impl AsPageId for i64 {
    fn as_page_id(&self) -> PageId {
        PageId::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_message_id_of_the_page() {
        assert_eq!(099_999, PageId(0).get_last_message_id().get_value());

        assert_eq!(199_999, PageId(1).get_last_message_id().get_value());

        assert_eq!(299_999, PageId(2).get_last_message_id().get_value());
    }
}
