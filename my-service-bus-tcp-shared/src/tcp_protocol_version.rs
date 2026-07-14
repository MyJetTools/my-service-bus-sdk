pub const DEFAULT_TCP_PROTOCOL_VERSION: i32 = 3;
#[derive(Clone, Copy, Debug)]
pub struct TcpProtocolVersion(i32);

impl Default for TcpProtocolVersion {
    fn default() -> Self {
        Self(DEFAULT_TCP_PROTOCOL_VERSION)
    }
}

impl TcpProtocolVersion {
    pub fn get_value(&self) -> i32 {
        self.0
    }
}

impl Into<TcpProtocolVersion> for i32 {
    fn into(self) -> TcpProtocolVersion {
        TcpProtocolVersion(self)
    }
}
