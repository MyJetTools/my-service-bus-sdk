pub const DEFAULT_NAMESPACE: &str = "default";

#[derive(Debug, Clone)]
pub enum InvalidTopicName {
    InvalidNameFormat(String),
    NameIsReserved,
}

pub fn validate_topic_name(name: &str) -> Result<(), InvalidTopicName> {
    if name == "topics" {
        return Err(InvalidTopicName::NameIsReserved);
    }

    if name.len() < 3 {
        return Err(InvalidTopicName::InvalidNameFormat(
            "Table name must contain at least 3 symbols".to_string(),
        ));
    }

    if name.len() > 63 {
        return Err(InvalidTopicName::InvalidNameFormat(
            "Table name must contain 3-63 symbols".to_string(),
        ));
    }

    let mut i = 0;

    let mut prev_char: Option<char> = None;

    let as_bytes = name.as_bytes();

    for s in as_bytes {
        let c = *s as char;

        if i == 0 {
            if c == '-' {
                return Err(InvalidTopicName::InvalidNameFormat(format!(
                    "Table can not be started from '-' symbol",
                )));
            }
        }

        if i == as_bytes.len() - 1 {
            if c == '-' {
                return Err(InvalidTopicName::InvalidNameFormat(format!(
                    "Table can not be ended with '-' symbol",
                )));
            }
        }

        if !symbol_is_allowed(c) {
            return Err(InvalidTopicName::InvalidNameFormat(format!(
                "Symbol {} is not allowed which stays at position {}",
                c, i
            )));
        }

        if c == '-' {
            if let Some(prev_char) = prev_char {
                if prev_char == '-' {
                    return Err(InvalidTopicName::InvalidNameFormat(format!(
                        "Two following '-' symbols are not allowed. Check please position {}",
                        i
                    )));
                }
            }
        }

        prev_char = Some(c);
        i += 1;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidNamespaceName {
    InvalidNameFormat(String),
}

impl InvalidNamespaceName {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidNameFormat(reason) => reason.as_str(),
        }
    }
}

impl std::fmt::Display for InvalidNamespaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::error::Error for InvalidNamespaceName {}

/// Namespace name allows only `[a-z0-9-]` and does not start with `-`; the length is 1..=63.
///
/// Upper case is an error and is never silently lower-cased: a namespace is auto-created by
/// the server, so a typo has to fail instead of bringing a garbage namespace to life.
pub fn validate_namespace_name(name: &str) -> Result<(), InvalidNamespaceName> {
    if name.len() < 1 {
        return Err(InvalidNamespaceName::InvalidNameFormat(
            "Namespace name must contain at least 1 symbol".to_string(),
        ));
    }

    if name.len() > 63 {
        return Err(InvalidNamespaceName::InvalidNameFormat(
            "Namespace name must contain 1-63 symbols".to_string(),
        ));
    }

    let mut i = 0;

    for s in name.as_bytes() {
        let c = *s as char;

        if i == 0 {
            if c == '-' {
                return Err(InvalidNamespaceName::InvalidNameFormat(
                    "Namespace can not be started from '-' symbol".to_string(),
                ));
            }
        }

        if !symbol_is_allowed(c) {
            return Err(InvalidNamespaceName::InvalidNameFormat(format!(
                "Symbol {} is not allowed which stays at position {}",
                c, i
            )));
        }

        i += 1;
    }

    Ok(())
}

fn symbol_is_allowed(c: char) -> bool {
    c == '-' || is_digit(c) || is_lower_case_latin_letter(c)
}

fn is_digit(c: char) -> bool {
    return c >= '0' && c <= '9';
}

fn is_lower_case_latin_letter(c: char) -> bool {
    return c >= 'a' && c <= 'z';
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lower_cases_and_dashes_ok() {
        let test_table_name = "my-test-name-5";

        let result = validate_topic_name(test_table_name);

        assert_eq!(true, result.is_ok());
    }

    #[test]
    fn test_lower_cases_and_two_dashes() {
        let test_table_name = "my-test--name";

        let result = validate_topic_name(test_table_name);

        assert_eq!(false, result.is_ok());

        if let Err(err) = result {
            if let InvalidTopicName::InvalidNameFormat(name) = err {
                println!("{}", name);
            } else {
                panic!("Should not be here");
            }
        }
    }

    #[test]
    fn test_lower_cases_and_start_with_dash() {
        let test_table_name = "-my-test-name";

        let result = validate_topic_name(test_table_name);

        assert_eq!(false, result.is_ok());

        if let Err(err) = result {
            if let InvalidTopicName::InvalidNameFormat(name) = err {
                println!("{}", name);
            } else {
                panic!("Should not be here");
            }
        }
    }

    #[test]
    fn test_lower_cases_and_ended_with_dash() {
        let test_table_name = "my-test-name-";

        let result = validate_topic_name(test_table_name);

        assert_eq!(false, result.is_ok());

        if let Err(err) = result {
            if let InvalidTopicName::InvalidNameFormat(name) = err {
                println!("{}", name);
            } else {
                panic!("Should not be here");
            }
        }
    }

    #[test]
    fn test_upper_cases_and_ended_with_dash() {
        let test_table_name = "my-test-Name";

        let result = validate_topic_name(test_table_name);

        assert_eq!(false, result.is_ok());

        if let Err(err) = result {
            if let InvalidTopicName::InvalidNameFormat(name) = err {
                println!("{}", name);
            } else {
                panic!("Should not be here");
            }
        }
    }

    #[test]
    fn test_we_handle_reserved_name() {
        let test_table_name = "topics";

        let result = validate_topic_name(test_table_name);

        assert_eq!(false, result.is_ok());

        if let Err(err) = result {
            if let InvalidTopicName::NameIsReserved = err {
            } else {
                panic!("Should not be here");
            }
        }
    }

    #[test]
    fn test_namespace_lower_cases_digits_and_dashes_ok() {
        assert_eq!(true, validate_namespace_name("my-namespace-5").is_ok());
        assert_eq!(true, validate_namespace_name(DEFAULT_NAMESPACE).is_ok());
        assert_eq!(true, validate_namespace_name("a").is_ok());
        assert_eq!(true, validate_namespace_name("my--namespace").is_ok());
        assert_eq!(true, validate_namespace_name("my-namespace-").is_ok());
    }

    #[test]
    fn test_namespace_can_not_be_started_with_dash() {
        assert_eq!(false, validate_namespace_name("-my-namespace").is_ok());
    }

    #[test]
    fn test_namespace_with_not_allowed_symbols() {
        assert_eq!(false, validate_namespace_name("my-Namespace").is_ok());
        assert_eq!(false, validate_namespace_name("my_namespace").is_ok());
        assert_eq!(false, validate_namespace_name("my namespace").is_ok());
    }

    #[test]
    fn test_namespace_length_boundaries() {
        assert_eq!(false, validate_namespace_name("").is_ok());

        let max_len = "a".repeat(63);
        assert_eq!(true, validate_namespace_name(max_len.as_str()).is_ok());

        let too_long = "a".repeat(64);
        assert_eq!(false, validate_namespace_name(too_long.as_str()).is_ok());
    }
}
