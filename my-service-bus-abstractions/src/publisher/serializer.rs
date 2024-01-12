use super::SbMessageHeaders;

pub trait MySbMessageSerializer {
    fn serialize(
        &self,
        headers: Option<SbMessageHeaders>,
    ) -> Result<(Vec<u8>, SbMessageHeaders), String>;
}
