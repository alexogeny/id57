use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyLong;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const ALPHABET: &[u8; 57] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
const BASE: u128 = 57;
const TIMESTAMP_WIDTH: usize = 11;
const UUID_WIDTH: usize = 22;
const IDENTIFIER_WIDTH: usize = TIMESTAMP_WIDTH + UUID_WIDTH;
const MAX_ENCODED_LEN: usize = UUID_WIDTH;
const INVALID: u8 = 0xFF;

const fn build_decode_table() -> [u8; 256] {
    let mut table = [INVALID; 256];
    let mut i = 0;
    while i < ALPHABET.len() {
        table[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
}

const DECODE_TABLE: [u8; 256] = build_decode_table();

fn encode_base57_raw(mut value: u128) -> Vec<u8> {
    if value == 0 {
        return vec![ALPHABET[0]];
    }

    let mut buf = Vec::with_capacity(MAX_ENCODED_LEN);
    while value > 0 {
        let remainder = (value % BASE) as usize;
        buf.push(ALPHABET[remainder]);
        value /= BASE;
    }
    buf.reverse();
    buf
}

fn encode_base57(value: u128, pad_to: Option<usize>) -> String {
    let digits = String::from_utf8(encode_base57_raw(value)).expect("alphabet is valid UTF-8");
    if let Some(width) = pad_to {
        if width > digits.len() {
            let mut padded = String::with_capacity(width);
            padded.extend(std::iter::repeat(ALPHABET[0] as char).take(width - digits.len()));
            padded.push_str(&digits);
            return padded;
        }
    }
    digits
}

fn decode_base57(value: &str) -> PyResult<u128> {
    if value.is_empty() {
        return Err(PyValueError::new_err("Value cannot be empty"));
    }

    let mut result: u128 = 0;
    for (index, ch) in value.char_indices() {
        let byte = match ch.is_ascii() {
            true => ch as u8,
            false => {
                return Err(PyValueError::new_err(format!(
                    "Invalid base57 character: {ch:?} at position {index}"
                )))
            }
        };
        let digit = DECODE_TABLE[byte as usize];
        if digit == INVALID {
            return Err(PyValueError::new_err(format!(
                "Invalid base57 character: {ch:?} at position {index}"
            )));
        }
        result = result
            .checked_mul(BASE)
            .and_then(|acc| acc.checked_add(digit as u128))
            .ok_or_else(|| PyValueError::new_err("Decoded value overflowed u128"))?;
    }
    Ok(result)
}

fn extract_uuid(py: Python<'_>, uuid: Option<PyObject>) -> PyResult<u128> {
    match uuid {
        Some(obj) => {
            let any = obj.bind(py);
            if any.is_instance_of::<PyLong>() {
                if any.lt(0)? {
                    return Err(PyValueError::new_err("uuid must be non-negative"));
                }
                return any.extract::<u128>();
            }
            if let Ok(value) = any.extract::<u128>() {
                return Ok(value);
            }
            if let Ok(int_obj) = any.call_method0("__int__") {
                if int_obj.lt(0)? {
                    return Err(PyValueError::new_err("uuid must be non-negative"));
                }
                return int_obj.extract::<u128>();
            }
            Err(PyValueError::new_err(
                "uuid must be an int or expose an __int__ method",
            ))
        }
        None => Ok(Uuid::new_v4().as_u128()),
    }
}

fn extract_timestamp(timestamp: Option<Bound<'_, PyAny>>) -> PyResult<u128> {
    match timestamp {
        Some(value) => {
            if value.lt(0)? {
                return Err(PyValueError::new_err("timestamp must be non-negative"));
            }
            Ok(value.extract::<u128>()?)
        }
        None => {
            let duration = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|err| PyRuntimeError::new_err(err.to_string()))?;
            let micros = (duration.as_secs() as u128) * 1_000_000
                + (duration.subsec_nanos() as u128 / 1_000);
            Ok(micros)
        }
    }
}

#[pyfunction]
#[pyo3(signature = (value, pad_to=None))]
fn base57_encode(value: Bound<'_, PyAny>, pad_to: Option<usize>) -> PyResult<String> {
    if value.lt(0)? {
        return Err(PyValueError::new_err("Value must be non-negative"));
    }
    let number = value.extract::<u128>()?;
    Ok(encode_base57(number, pad_to))
}

#[pyfunction]
fn decode57(value: &str) -> PyResult<u128> {
    decode_base57(value)
}

#[pyfunction]
#[pyo3(signature = (timestamp=None, uuid=None))]
fn generate_id57(
    py: Python<'_>,
    timestamp: Option<Bound<'_, PyAny>>,
    uuid: Option<PyObject>,
) -> PyResult<String> {
    let ts_value = extract_timestamp(timestamp)?;
    let uuid_value = extract_uuid(py, uuid)?;
    let ts_part = encode_base57(ts_value, Some(TIMESTAMP_WIDTH));
    let uuid_part = encode_base57(uuid_value, Some(UUID_WIDTH));
    let mut identifier = String::with_capacity(IDENTIFIER_WIDTH);
    identifier.push_str(&ts_part);
    identifier.push_str(&uuid_part);
    Ok(identifier)
}

#[pymodule]
fn _core(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("ALPHABET", std::str::from_utf8(ALPHABET).unwrap())?;
    m.add_function(wrap_pyfunction!(base57_encode, m)?)?;
    m.add_function(wrap_pyfunction!(decode57, m)?)?;
    m.add_function(wrap_pyfunction!(generate_id57, m)?)?;
    Ok(())
}
