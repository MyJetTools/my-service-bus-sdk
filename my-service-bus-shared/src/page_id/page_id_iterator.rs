use my_service_bus_abstractions::MessageId;

use super::{PageId, MESSAGES_IN_PAGE};

pub struct PageIdIterator {
    from_id: i64,
    to_id: i64,
}

impl PageIdIterator {
    pub fn new(page_id: PageId) -> Self {
        let from_id = page_id.get_first_message_id().get_value();
        Self {
            from_id,
            to_id: from_id + MESSAGES_IN_PAGE,
        }
    }
}

impl Iterator for PageIdIterator {
    type Item = MessageId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.from_id >= self.to_id {
            None
        } else {
            let result = self.from_id;
            self.from_id = self.from_id + 1;
            Some(result.into())
        }
    }
}
