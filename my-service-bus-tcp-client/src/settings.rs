#[async_trait::async_trait]
pub trait MyServiceBusSettings {
    /// MyServiceBus connection string.
    ///
    /// Both formats are accepted: the legacy bare `host:port` and the `;` separated
    /// `host=127.0.0.1:6421;ns=alpha` one. See
    /// [`my_service_bus_shared::connection_string::parse_connection_string`].
    async fn get_host_port(&self) -> String;
}
