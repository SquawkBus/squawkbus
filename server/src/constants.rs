pub const LEVEL_SEPARATOR: &str = ".";
pub const MULTI_LEVEL_WILDCARD: &str = "*";
pub const SINGLE_LEVEL_WILDCARD: &str = "?";

pub const SYSTEM_WORD: &str = "~";
pub const SUBSCRIPTIONS_CATEGORY: &str = "subscriptions";

pub const SUBSCRIPTION_TOPIC: &str =
    const_str::join!(&[SYSTEM_WORD, SUBSCRIPTIONS_CATEGORY], LEVEL_SEPARATOR);

pub const SQUAWKBUS_CONTENT_TYPE: &str = "application/x-squawkbus";
