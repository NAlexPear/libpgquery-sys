#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/pg_query.rs"));

// TODO: spin these wrappers out into a separate crate
use prost::Message;
use std::ffi::{CStr, CString, NulError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid statement format: {0}")]
    Conversion(#[from] NulError),
    #[error("Error decoding result: {0}")]
    Decode(#[from] prost::DecodeError),
    #[error("Invalid statement: {0}")]
    Parse(std::string::String),
}

/// Parses the given SQL statement into a JSON-formatted parse tree
pub fn parseToJson(statement: &str) -> Result<std::string::String, Error> {
    let input = CString::new(statement)?;
    let result = unsafe { pg_query_parse(input.as_ptr()) };

    let response = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();

        Err(Error::Parse(message))
    } else {
        let parse_tree = unsafe { CStr::from_ptr(result.parse_tree) }
            .to_string_lossy()
            .to_string();

        Ok(parse_tree)
    };

    unsafe { pg_query_free_parse_result(result) };

    response
}

/// Parses the given SQL statement into a Protobuf-formatted parse tree
pub fn parseToProtobuf(statement: &str) -> Result<ParseResult, Error> {
    let input = CString::new(statement)?;
    let result = unsafe { pg_query_parse_protobuf(input.as_ptr()) };

    let response = if !result.error.is_null() {
        let message = unsafe { CStr::from_ptr((*result.error).message) }
            .to_string_lossy()
            .to_string();

        Err(Error::Parse(message))
    } else {
        let data = unsafe {
            std::slice::from_raw_parts(
                result.parse_tree.data as *const u8,
                result.parse_tree.len as usize,
            )
        };

        ParseResult::decode(data).map_err(Error::Decode)
    };

    unsafe { pg_query_free_protobuf_parse_result(result) };

    response
}

/// wrapper around parseToProtobuf for ease-of-use
pub fn parse(statement: &str) -> Result<ParseResult, Error> {
    parseToProtobuf(statement)
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO:
    // add libpg_query test suite
    // add pg_query_go test suite

    #[test]
    fn parses_to_json() {
        let json =
            parseToJson("select * from items").expect("Error parsing valid statement into JSON");

        let _map: serde_json::Map<std::string::String, serde_json::Value> =
            serde_json::from_str(&json).expect("Error parsing response into JSON Object");
    }

    #[test]
    fn parses_to_proto() {
        let _ = parseToProtobuf("select null")
            .expect("Error parsing statement 'select null' into Protobuf");

        let _ = parseToProtobuf("select ''")
            .expect("Error parsing statement 'select ''' into Protobuf");

        let _ = parseToProtobuf("checkpoint")
            .expect("Error parsing statement 'checkpoint' into Protobuf");

        let _ = parseToProtobuf("select from items")
            .expect("Error parsing statement 'select from items' into Protobuf");

        let proto =
            parseToProtobuf("select *").expect("Error parsing statement 'select *' into Protobuf");

        assert_eq!(
            proto.stmts.len(),
            1,
            "Failed to parse correct number of statements"
        )
    }
}
