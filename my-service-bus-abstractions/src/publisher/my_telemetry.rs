use my_telemetry::MyTelemetryContext;

use crate::SbMessageHeaders;

pub fn apply_publish_telemetry(headers: &mut SbMessageHeaders, my_telemetry: &MyTelemetryContext) {
    headers.add_header(
        crate::MY_TELEMETRY_HEADER.to_string(),
        my_telemetry.as_string(),
    );
}
