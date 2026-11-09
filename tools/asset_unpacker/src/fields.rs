//! Strict access to source TypeTree fields, shared by mesh/material/scene extraction.

use unity_asset_binary::object::UnityObject;
use unity_asset_core::UnityValue;

use crate::unity::Error;

pub fn invalid(message: impl Into<String>) -> Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message.into()).into()
}

pub fn property<'a>(object: &'a UnityObject, name: &str) -> Result<&'a UnityValue, Error> {
    object.class.properties().get(name).ok_or_else(|| invalid(format!("missing {name}")))
}

pub fn field<'a>(value: &'a UnityValue, name: &str) -> Result<&'a UnityValue, Error> {
    value.as_object().and_then(|fields| fields.get(name))
        .ok_or_else(|| invalid(format!("missing {name}")))
}

pub fn integer(value: &UnityValue) -> Result<i64, Error> {
    value.as_i64().ok_or_else(|| invalid("expected integer"))
}

pub fn boolean(value: &UnityValue) -> Result<bool, Error> {
    value.as_bool().ok_or_else(|| invalid("expected boolean"))
}

pub fn number(value: &UnityValue) -> Result<f32, Error> {
    let number = value.as_f64().ok_or_else(|| invalid("expected number"))? as f32;
    if !number.is_finite() { return Err(invalid("non-finite source number")); }
    Ok(number)
}

pub fn text(value: &UnityValue) -> Result<&str, Error> {
    value.as_str().ok_or_else(|| invalid("expected string"))
}

pub fn array(value: &UnityValue) -> Result<&[UnityValue], Error> {
    value.as_array().map(Vec::as_slice).ok_or_else(|| invalid("expected array"))
}

pub fn vector<const N: usize>(value: &UnityValue, names: [&str; N]) -> Result<[f32; N], Error> {
    let mut result = [0.0; N];
    for (index, name) in names.into_iter().enumerate() { result[index] = number(field(value, name)?)?; }
    Ok(result)
}

pub fn bytes(value: &UnityValue) -> Result<Vec<u8>, Error> {
    if let Some(bytes) = value.as_bytes() { return Ok(bytes.to_vec()); }
    array(value)?.iter().map(|item| {
        u8::try_from(integer(item)?).map_err(|_| invalid("byte outside 0..255"))
    }).collect()
}

pub fn pairs(value: &UnityValue) -> Result<Vec<(&str, &UnityValue)>, Error> {
    array(value)?.iter().map(|item| {
        let pair = array(item)?;
        if pair.len() != 2 { return Err(invalid("malformed material property pair")); }
        Ok((text(&pair[0])?, &pair[1]))
    }).collect()
}
