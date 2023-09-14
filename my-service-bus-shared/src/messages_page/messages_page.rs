use std::collections::BTreeMap;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    messages::{MySbMessage, MySbMessageContent},
    page_id::PageId,
    protobuf_models::{MessageMetaDataProtobufModel, MessageProtobufModel},
    queue_with_intervals::QueueWithIntervals,
    MessageId,
};

pub enum MessageSize {
    MessageIsReady(usize),
    NotLoaded,
    Missing,
}

pub struct MessagesPage {
    pub to_be_persisted: QueueWithIntervals,
    pub being_persisted: QueueWithIntervals,
    full_loaded_messages: QueueWithIntervals,
    pub messages: BTreeMap<MessageId, MySbMessage>,
    pub size: usize,
    pub is_being_persisted: bool,
    pub page_id: PageId,
    pub last_accessed: DateTimeAsMicroseconds,
}

impl MessagesPage {
    pub fn create_empty(page_id: PageId) -> MessagesPage {
        MessagesPage {
            messages: BTreeMap::new(),
            size: 0,
            to_be_persisted: QueueWithIntervals::new(),
            being_persisted: QueueWithIntervals::new(),
            is_being_persisted: false,
            full_loaded_messages: QueueWithIntervals::new(),
            page_id,
            last_accessed: DateTimeAsMicroseconds::now(),
        }
    }

    /*
       pub fn restore(snapshot: MessagesPageRestoreSnapshot) -> Self {
           let mut result = MessagesPage::create_empty(snapshot.page_id);

           for msg in snapshot {
               result.update_message(msg);
           }

           result
       }
    */
    pub fn add_message(&mut self, msg: MySbMessageContent) {
        self.size += msg.content.len();

        self.to_be_persisted.enqueue(msg.id);
        self.full_loaded_messages.enqueue(msg.id);

        let old = self.messages.insert(msg.id, MySbMessage::Loaded(msg));

        if let Some(old) = old {
            self.size -= old.content_size();
        }
    }

    fn update_message(&mut self, msg: MySbMessage) {
        self.size += msg.content_size();

        if let MySbMessage::Loaded(full_message) = &msg {
            self.full_loaded_messages.enqueue(full_message.id);
        }

        let message_id = msg.get_id();

        let old = self.messages.insert(message_id, msg);

        if let Some(old) = old {
            self.size -= old.content_size();

            if let MySbMessage::Loaded(full_message) = &old {
                self.full_loaded_messages.remove(full_message.id);
            }
        }
    }

    fn update_messages(&mut self, msgs: Vec<MySbMessage>) -> Option<MessageId> {
        let mut min_message_id = None;
        for msg in msgs {
            let message_id = msg.get_id();

            self.update_message(msg);

            if let Some(current_min_message_id) = min_message_id {
                if current_min_message_id > message_id {
                    min_message_id = Some(message_id);
                }
            } else {
                min_message_id = Some(message_id);
            }
        }

        min_message_id
    }

    pub fn get_message_size(&self, message_id: &MessageId) -> MessageSize {
        let msg = self.messages.get(message_id);

        if msg.is_none() {
            return MessageSize::Missing;
        }

        match msg.unwrap() {
            MySbMessage::Loaded(msg) => MessageSize::MessageIsReady(msg.content.len()),
            MySbMessage::Missing { id: _ } => MessageSize::Missing,
        }
    }

    fn gc(&mut self, messages_to_gc: Vec<MySbMessage>) {
        for msg_to_gc in &messages_to_gc {
            self.full_loaded_messages.remove(msg_to_gc.get_id());
        }

        self.update_messages(messages_to_gc);
    }

    /*
       pub fn gc_messages(&mut self, up_to_message_id: MessageId) {
           let mut messages_to_gc_result = None;

           for msg_id in &self.full_loaded_messages {
               if msg_id >= up_to_message_id {
                   break;
               }

               if self.to_be_persisted.has_message(msg_id) {
                   break;
               }

               if self.being_persisted.has_message(msg_id) {
                   break;
               }

               if messages_to_gc_result.is_none() {
                   messages_to_gc_result = Some(Vec::new())
               }

               if let Some(vec) = &mut messages_to_gc_result {
                   vec.remove(&msg_id);
               }
           }

           if let Some(messages_to_gc) = messages_to_gc_result {
               self.gc(messages_to_gc);
           }
       }
    */

    pub fn get_messages_to_persist(&mut self) -> Option<Vec<MessageProtobufModel>> {
        let mut result = None;

        while let Some(msg_id) = self.to_be_persisted.dequeue() {
            if let Some(message) = self.messages.get(&msg_id) {
                if let MySbMessage::Loaded(content) = message {
                    self.being_persisted.enqueue(msg_id);

                    if result.is_none() {
                        result = Some(Vec::new());
                    }

                    let mut headers = Vec::new();

                    if let Some(c_headers) = &content.headers {
                        for (key, value) in c_headers {
                            headers.push(MessageMetaDataProtobufModel {
                                key: key.to_string(),
                                value: value.to_string(),
                            })
                        }
                    }

                    result.as_mut().unwrap().push(MessageProtobufModel {
                        created: content.time.unix_microseconds,
                        message_id: content.id,
                        data: content.content.clone(),
                        headers,
                    });
                }
            }
        }

        result
    }

    pub fn persisted(&mut self) {
        self.being_persisted.clean();
    }

    pub fn not_persisted(&mut self) {
        for range in &self.being_persisted.intervals {
            self.to_be_persisted.enqueue_range(range);
        }

        self.being_persisted.clean();
    }
}

//TODO - Restore or Delete
/*
#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::*;

       #[test]
       pub fn test_gc_messages() {
           let mut msgs_to_restore = HashMap::new();

           msgs_to_restore.insert(
               5,
               MySbMessageContent {
                   id: 5,
                   time: DateTimeAsMicroseconds::now(),
                   content: vec![5u8, 5u8, 5u8],
                   headers: None,
               },
           );

           msgs_to_restore.insert(
               6,
               MySbMessageContent {
                   id: 6,
                   time: DateTimeAsMicroseconds::now(),
                   content: vec![6u8, 6u8, 6u8],
                   headers: None,
               },
           );

           msgs_to_restore.insert(
               7,
               MySbMessageContent {
                   id: 7,
                   time: DateTimeAsMicroseconds::now(),
                   content: vec![7u8, 7u8, 7u8],
                   headers: None,
               },
           );

           msgs_to_restore.insert(
               8,
               MySbMessageContent {
                   id: 8,
                   time: DateTimeAsMicroseconds::now(),
                   content: vec![7u8, 7u8, 7u8],
                   headers: None,
               },
           );

           let mut restore_snapshot = MessagesPageRestoreSnapshot::new(0, 5, 8);
           restore_snapshot.messages = Some(msgs_to_restore);

           let mut page_data = MessagesPage::restore(restore_snapshot);

           assert_eq!(4, page_data.full_loaded_messages.len());

           assert_eq!(true, page_data.full_loaded_messages.has_message(5));
           assert_eq!(true, page_data.full_loaded_messages.has_message(6));
           assert_eq!(true, page_data.full_loaded_messages.has_message(7));
           assert_eq!(true, page_data.full_loaded_messages.has_message(8));

           page_data.gc_messages(7);

           assert_eq!(2, page_data.full_loaded_messages.len());

           assert_eq!(false, page_data.full_loaded_messages.has_message(5));
           assert_eq!(false, page_data.full_loaded_messages.has_message(6));
           assert_eq!(true, page_data.full_loaded_messages.has_message(7));
           assert_eq!(true, page_data.full_loaded_messages.has_message(8));
       }

    #[test]
    fn test_new_with_all_missing_and_loaded() {
        let restore_snapshot = MessagesPageRestoreSnapshot::new(0, 5, 10);
        let page_data = MessagesPage::restore(restore_snapshot);

        assert_eq!(11, page_data.messages.len());

        for msg_id in 5..11 {
            let msg = page_data.messages.get(&msg_id).unwrap();

            if let MySbMessage::Missing { id } = msg {
                assert_eq!(*id, msg_id);
            } else {
                panic!("We should not be here");
            }
        }

        for msg_id in 0..5 {
            let msg = page_data.messages.get(&msg_id);

            if msg.is_none() {
            } else {
                panic!("We should not be here");
            }
        }
    }

    #[test]
    fn test_being_persisted_and_back() {
        let mut page = MessagesPage::create_empty(0);

        page.new_message(MySbMessageContent {
            id: 1,
            content: vec![0u8, 1u8, 2u8],
            time: DateTimeAsMicroseconds::now(),
            headers: None,
        });

        if let Some(messages_to_persist) = page.get_messages_to_persist() {
            assert_eq!(1, messages_to_persist.len());
            assert_eq!(1, page.being_persisted.len());
            assert_eq!(0, page.to_be_persisted.len());
        } else {
            panic!("Should not be here");
        }

        page.not_persisted();
        assert_eq!(0, page.being_persisted.len());
        assert_eq!(1, page.to_be_persisted.len());
    }

}
*/
