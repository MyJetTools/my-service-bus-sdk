#[derive(Clone, Copy, Debug)]
pub struct TcpProtocolVersion(i32);

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
