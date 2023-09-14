use std::collections::HashMap;

use my_telemetry::MyTelemetryContext;

pub fn apply_publish_telemetry(
    headers: &mut Option<HashMap<String, String>>,
    my_telemetry: &MyTelemetryContext,
) {
    if headers.is_none() {
        *headers = Some(HashMap::new());
    }

    headers.as_mut().unwrap().insert(
        crate::MY_TELEMETRY_HEADER.to_string(),
        my_telemetry.as_string(),
    );
}
