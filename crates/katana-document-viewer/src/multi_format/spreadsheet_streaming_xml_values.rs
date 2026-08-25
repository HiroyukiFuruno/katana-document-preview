use super::spreadsheet_engine::SpreadsheetEngineError;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

pub(super) fn parse_shared_strings(xml: &[u8]) -> Result<Vec<String>, SpreadsheetEngineError> {
    let mut reader = Reader::from_reader(xml);
    let mut strings = Vec::new();
    let mut current = None::<String>;
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) if event.local_name().as_ref() == "si" => {
                current = Some(String::new());
            }
            Ok(Event::Text(text)) => {
                if let Some(value) = current.as_mut() {
                    let decoded = decode_text(text.as_ref().as_bytes())?;
                    value.push_str(&decoded);
                }
            }
            Ok(Event::End(event)) if event.local_name().as_ref() == "si" => {
                if let Some(value) = current.take() {
                    strings.push(value);
                }
            }
            Ok(Event::Eof) => return Ok(strings),
            Ok(_) => {}
            Err(error) => return Err(xml_error(error)),
        }
    }
}

pub(super) fn required_attribute(
    reader: &Reader<impl std::io::BufRead>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<String, SpreadsheetEngineError> {
    attribute(reader, event, name)?.ok_or_else(|| {
        SpreadsheetEngineError::Import(format!(
            "required spreadsheet XML attribute `{}` is missing",
            String::from_utf8_lossy(name)
        ))
    })
}

pub(super) fn decode_text(bytes: &[u8]) -> Result<String, SpreadsheetEngineError> {
    let value = std::str::from_utf8(bytes)
        .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
    quick_xml::escape::unescape(value)
        .map(|value| value.into_owned())
        .map_err(|error| SpreadsheetEngineError::Import(error.to_string()))
}

pub(super) fn attribute(
    _reader: &Reader<impl std::io::BufRead>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, SpreadsheetEngineError> {
    for attribute in event.attributes() {
        let attribute =
            attribute.map_err(|error| SpreadsheetEngineError::Import(error.to_string()))?;
        if attribute.key.local_name().as_ref().as_bytes() == name {
            return attribute
                .normalized_value(XmlVersion::Implicit1_0)
                .map(|value| Some(value.into_owned()))
                .map_err(|error| SpreadsheetEngineError::Import(error.to_string()));
        }
    }
    Ok(None)
}

pub(super) fn attribute_f32(
    reader: &Reader<impl std::io::BufRead>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<f32>, SpreadsheetEngineError> {
    attribute(reader, event, name)?
        .map(|value| {
            value.parse().map_err(|error: std::num::ParseFloatError| {
                SpreadsheetEngineError::Import(error.to_string())
            })
        })
        .transpose()
}

pub(super) fn attribute_usize(
    reader: &Reader<impl std::io::BufRead>,
    event: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<usize>, SpreadsheetEngineError> {
    attribute(reader, event, name)?
        .map(|value| {
            value.parse().map_err(|error: std::num::ParseIntError| {
                SpreadsheetEngineError::Import(error.to_string())
            })
        })
        .transpose()
}

pub(super) fn xml_error(error: quick_xml::Error) -> SpreadsheetEngineError {
    SpreadsheetEngineError::Import(error.to_string())
}
