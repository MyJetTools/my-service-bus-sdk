use my_telemetry::{EventDurationTracker, MyTelemetryContext};
use rust_extensions::StrOrString;

use crate::{publisher::SbMessageHeaders, MessageId};

pub struct DeliveredMessageTelemetry {
    telemetry_event_name: Option<String>,
    ctx: MyTelemetryContext,
    pub event_duration_tracker: Option<EventDurationTracker>,
    pub ignore_this_event: bool,
    pub message_id: MessageId,
}

impl DeliveredMessageTelemetry {
    pub fn new(
        topic_id: &str,
        queue_id: &str,
        message_id: MessageId,
        headers: &SbMessageHeaders,
    ) -> Self {
        use crate::MY_TELEMETRY_HEADER;
        let telemetry_event_name = format!("Handling event {}/{}", topic_id, queue_id,);

        if let Some(telemetry_value) = headers.get(MY_TELEMETRY_HEADER) {
            if let Ok(ctx) = MyTelemetryContext::parse_from_string(telemetry_value) {
                return Self {
                    ctx,
                    event_duration_tracker: None,
                    ignore_this_event: false,
                    message_id,
                    telemetry_event_name: Some(telemetry_event_name),
                }
                .into();
            }
        }

        Self::create_brand_new_telemetry(message_id, telemetry_event_name)
    }

    fn create_brand_new_telemetry(message_id: MessageId, telemetry_event_name: String) -> Self {
        Self {
            telemetry_event_name: Some(telemetry_event_name),
            ctx: MyTelemetryContext::new(),
            event_duration_tracker: None,
            ignore_this_event: true,

            message_id,
        }
    }

    pub fn ignore_this_event(&mut self) {
        self.ignore_this_event = true;
    }

    pub fn enabled_duration_tracking_on_confirmation(&mut self) {
        if !self.ignore_this_event {
            if let Some(event_duration_tracker) = &mut self.event_duration_tracker {
                event_duration_tracker.do_not_ignore_this_event();
            }
        }
    }

    pub fn engage_telemetry(&mut self) -> MyTelemetryContext {
        if self.event_duration_tracker.is_none() {
            if let Some(telemetry_event_name) = self.telemetry_event_name.take() {
                let mut event_duration_tracker =
                    self.ctx.start_event_tracking(telemetry_event_name);

                event_duration_tracker.add_tag(
                    "msg_id".to_string(),
                    self.message_id.get_value().to_string(),
                );

                event_duration_tracker.ignore_this_event();

                self.event_duration_tracker = Some(event_duration_tracker);
            }
        }

        self.ctx.clone()
    }

    pub fn add_tag(
        &mut self,
        key: impl Into<StrOrString<'static>>,
        value: impl Into<StrOrString<'static>>,
    ) {
        match self.event_duration_tracker.as_mut() {
            Some(event_duration_tracker) => {
                let key: StrOrString<'static> = key.into();
                let value: StrOrString<'static> = value.into();

                event_duration_tracker.add_tag(key.to_string(), value.to_string());
            }
            None => {
                panic!("Telemetry is not engaged. Please call engage_telemetry before adding tags")
            }
        }
    }
}
