use std::collections::HashMap;

pub trait MySbMessageSerializer {
    fn serialize(
        &self,
        headers: Option<HashMap<String, String>>,
    ) -> Result<(Vec<u8>, Option<HashMap<String, String>>), String>;
}
