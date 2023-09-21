use std::collections::{vec_deque::IterMut, VecDeque};

use super::{MySbDeliveredMessage, MySbMessageDeserializer};

pub enum CurrentMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    Single(MySbDeliveredMessage<TMessageModel>),
    Multiple(VecDeque<MySbDeliveredMessage<TMessageModel>>),
    None,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> CurrentMessage<TMessageModel> {
    pub fn unwrap_as_single_message_mut<'s>(
        &'s mut self,
    ) -> &'s mut MySbDeliveredMessage<TMessageModel> {
        match self {
            CurrentMessage::Single(msg) => msg,
            CurrentMessage::Multiple(_) => panic!("CurrentMessage is not a single message"),
            CurrentMessage::None => panic!("CurrentMessage is none"),
        }
    }

    pub fn unwrap_as_iterator<'s>(
        &'s mut self,
    ) -> IterMut<'s, MySbDeliveredMessage<TMessageModel>> {
        match self {
            CurrentMessage::Single(_) => panic!("CurrentMessage is  a single message"),
            CurrentMessage::Multiple(result) => {
                let result = result.iter_mut();
                result
            }
            CurrentMessage::None => panic!("CurrentMessage is none"),
        }
    }

    pub fn take(&mut self) -> CurrentMessage<TMessageModel> {
        std::mem::replace(self, CurrentMessage::None)
    }
}
