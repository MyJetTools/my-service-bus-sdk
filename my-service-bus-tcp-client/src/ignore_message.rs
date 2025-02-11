use my_service_bus_abstractions::MessageId;

pub struct IgnoreMessage {
    pub topic_id: String,
    pub queue_id: String,
    pub message_id: MessageId,
}

const SKIP_MESSAGE_ID_ENV_NAME: &'static str = "SB_IGNORE_MESSAGE";

impl IgnoreMessage {
    pub fn new() -> Option<Self> {
        let data = std::env::var(SKIP_MESSAGE_ID_ENV_NAME);
        if data.is_err() {
            return None;
        }

        let data = data.unwrap();

        let mut topic_id = None;
        let mut queue_id = None;
        let mut message_id: Option<i64> = None;

        for key_value in data.split(";") {
            let mut key_value = key_value.split("=");
            let key = key_value.next().unwrap();

            if let Some(value) = key_value.next() {
                match key {
                    "TOPIC_ID" => {
                        topic_id = Some(value.to_string());
                    }

                    "QUEUE_ID" => {
                        queue_id = Some(value.to_string());
                    }
                    "MESSAGE_ID" => message_id = Some(value.parse().unwrap()),
                    _ => {
                        panic!(
                            "Invalid line [{data}] for ENV_VARIABLE {}",
                            SKIP_MESSAGE_ID_ENV_NAME
                        );
                    }
                }
            } else {
                panic!(
                    "Invalid line [{data}] for ENV_VARIABLE {}",
                    SKIP_MESSAGE_ID_ENV_NAME
                );
            }
        }

        if topic_id.is_none() {
            panic!(
                "Invalid line [{data}] for ENV_VARIABLE {}. TOPIC_ID is missing",
                SKIP_MESSAGE_ID_ENV_NAME
            );
        }

        if queue_id.is_none() {
            panic!(
                "Invalid line [{data}] for ENV_VARIABLE {}. QUEUE_ID is missing",
                SKIP_MESSAGE_ID_ENV_NAME
            );
        }

        if message_id.is_none() {
            panic!(
                "Invalid line [{data}] for ENV_VARIABLE {}. MESSAGE_ID is missing",
                SKIP_MESSAGE_ID_ENV_NAME
            );
        }

        let result = Self {
            topic_id: topic_id.unwrap(),
            queue_id: queue_id.unwrap(),
            message_id: message_id.unwrap().into(),
        };

        Some(result)
    }
}
