use num::traits::AsPrimitive;
use toml::value::*;

pub fn as_integer<T>(map: &Table, key: &str, default: T) -> T
where
    T: 'static + Copy,
    i64: AsPrimitive<T>,
{
    map.get(key)
        .map(|x| {
            x.as_integer()
                .unwrap_or_else(|| panic!("Invalid arguments in field {}", key))
                .as_()
        })
        .unwrap_or(default)
}

pub fn as_bool(map: &Table, key: &str, default: bool) -> bool {
    map.get(key)
        .map(|x| {
            x.as_bool()
                .unwrap_or_else(|| panic!("Invalid arguments in field {}", key))
        })
        .unwrap_or(default)
}

pub fn as_str<'a>(map: &'a Table, key: &str, default: &'a str) -> &'a str {
    map.get(key)
        .map(|x| {
            x.as_str()
                .unwrap_or_else(|| panic!("Invalid arguments in field {}", key))
        })
        .unwrap_or(default)
}

pub fn as_list<'a>(map: &'a Table, key: &str) -> Vec<&'a Value> {
    map.get(key)
        .map(|x| {
            x.as_array()
                .unwrap_or_else(|| panic!("Invalid arguments in field {}", key))
                .iter()
                .collect()
        })
        .unwrap_or_default()
}
