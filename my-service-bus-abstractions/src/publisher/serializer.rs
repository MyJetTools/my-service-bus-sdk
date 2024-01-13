pub trait MySbMessageSerializer {
    fn serialize(
        &self,
        headers: Option<crate::SbMessageHeaders>,
    ) -> Result<(Vec<u8>, crate::SbMessageHeaders), String>;
}
