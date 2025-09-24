use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyLong};
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

fn pad_digits(digits: String, pad_to: Option<usize>) -> String {
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

fn encode_base57(value: u128, pad_to: Option<usize>) -> String {
    let digits = String::from_utf8(encode_base57_raw(value)).expect("alphabet is valid UTF-8");
    pad_digits(digits, pad_to)
}

fn encode_big_base57(value: &BigUint, pad_to: Option<usize>) -> String {
    let digits = if value.is_zero() {
        vec![ALPHABET[0]]
    } else {
        value
            .to_radix_be(BASE as u32)
            .into_iter()
            .map(|digit| ALPHABET[digit as usize])
            .collect::<Vec<_>>()
    };
    let digits = String::from_utf8(digits).expect("alphabet is valid UTF-8");
    pad_digits(digits, pad_to)
}

fn decode_base57(value: &str) -> PyResult<BigUint> {
    if value.is_empty() {
        return Err(PyValueError::new_err("Value cannot be empty"));
    }

    let mut result = BigUint::from(0u8);
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
        result *= BASE as u32;
        result += BigUint::from(digit as u32);
    }
    Ok(result)
}

fn extract_biguint(value: Bound<'_, PyAny>, negative_message: &'static str) -> PyResult<BigUint> {
    if let Ok(long) = value.downcast::<PyLong>() {
        return pylong_to_biguint(&long, negative_message);
    }

    let int_obj = value
        .call_method0("__int__")
        .map_err(|_| PyValueError::new_err("Value must be an int or expose an __int__ method"))?;
    let long = int_obj
        .downcast::<PyLong>()
        .map_err(|_| PyValueError::new_err("Value must be an int or expose an __int__ method"))?;
    pylong_to_biguint(&long, negative_message)
}

fn pylong_to_biguint(
    long: &Bound<'_, PyLong>,
    negative_message: &'static str,
) -> PyResult<BigUint> {
    if long.lt(0)? {
        return Err(PyValueError::new_err(negative_message));
    }
    let bit_length: usize = long.call_method0("bit_length")?.extract()?;
    if bit_length == 0 {
        return Ok(BigUint::from(0u8));
    }
    let byte_length = (bit_length + 7) / 8;
    let bytes: Bound<'_, PyBytes> = long
        .call_method("to_bytes", (byte_length, "little"), None)?
        .downcast_into()?;
    Ok(BigUint::from_bytes_le(bytes.as_bytes()))
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
    let number = extract_biguint(value, "Value must be non-negative")?;
    if let Some(small) = number.to_u128() {
        Ok(encode_base57(small, pad_to))
    } else {
        Ok(encode_big_base57(&number, pad_to))
    }
}

#[pyfunction]
fn decode57(value: &str) -> PyResult<BigUint> {
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
