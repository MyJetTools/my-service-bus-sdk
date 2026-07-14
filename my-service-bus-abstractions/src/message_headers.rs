use rust_extensions::StrOrString;

#[derive(Debug, Clone)]
pub struct SbMessageHeaders {
    headers: Vec<(String, String)>,
}

impl SbMessageHeaders {
    pub fn new() -> Self {
        Self { headers: vec![] }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            headers: Vec::with_capacity(capacity),
        }
    }

    pub fn from_iterator(
        capacity: Option<usize>,
        src: impl Iterator<Item = (String, String)>,
    ) -> Self {
        let mut result = if let Some(capacity) = capacity {
            Self::with_capacity(capacity)
        } else {
            Self::new()
        };

        for itm in src {
            result = result.add(itm.0, itm.1);
        }

        result
    }

    pub fn add<'k, 'v>(
        mut self,
        key: impl Into<StrOrString<'k>>,
        value: impl Into<StrOrString<'v>>,
    ) -> Self {
        let key: StrOrString<'k> = key.into();
        let value: StrOrString<'v> = value.into();
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn add_header<'k, 'v>(
        &mut self,
        key: impl Into<StrOrString<'k>>,
        value: impl Into<StrOrString<'v>>,
    ) {
        let key: StrOrString<'k> = key.into();
        let value: StrOrString<'v> = value.into();
        self.headers.push((key.to_string(), value.to_string()));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        for itm in self.headers.iter() {
            if itm.0 == key {
                return Some(itm.1.as_str());
            }
        }

        None
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, String)> {
        self.headers.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, String)> {
        self.headers.into_iter()
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        let index = self.headers.iter().position(|(k, _)| k == key)?;

        let (_, value) = self.headers.remove(index);

        Some(value)
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }
}

impl Into<SbMessageHeaders> for &'_ [(String, String)] {
    fn into(self) -> SbMessageHeaders {
        let mut result = SbMessageHeaders::with_capacity(self.len());

        for itm in self {
            result = result.add(itm.0.to_string(), itm.1.to_string());
        }

        result
    }
}

impl Into<SbMessageHeaders> for Vec<(String, String)> {
    fn into(self) -> SbMessageHeaders {
        let mut result = SbMessageHeaders::with_capacity(self.len());

        for itm in self {
            result = result.add(itm.0, itm.1);
        }

        result
    }
}
