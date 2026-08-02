use super::office_preflight::{
    OfficePreflightError, OfficePreflightLimits, OfficePreflightSupport, OfficeResourceLimitKind,
};
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub(crate) struct OfficePreflightRelationships;

impl OfficePreflightRelationships {
    pub(crate) fn inspect(
        archive: &mut ZipArchive<Cursor<&[u8]>>,
        name: &str,
        limits: OfficePreflightLimits,
    ) -> Result<(), OfficePreflightError> {
        let mut entry = archive.by_name(name).map_err(zip_error)?;
        if entry.size() > limits.max_relationship_bytes {
            return Err(OfficePreflightSupport::resource_limit(
                OfficeResourceLimitKind::RelationshipBytes,
                entry.size(),
                limits.max_relationship_bytes,
                Some(name.to_owned()),
            ));
        }
        let mut xml = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut xml).map_err(io_error)?;
        inspect_xml(name, &xml)
    }
}

fn zip_error(error: zip::result::ZipError) -> OfficePreflightError {
    OfficePreflightSupport::invalid_archive(error.to_string())
}

fn io_error(error: std::io::Error) -> OfficePreflightError {
    OfficePreflightSupport::invalid_archive(error.to_string())
}

fn inspect_xml(name: &str, xml: &[u8]) -> Result<(), OfficePreflightError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) | Ok(Event::Empty(event))
                if event.local_name().as_ref() == b"Relationship" =>
            {
                inspect_element(name, &reader, &event)?;
            }
            Ok(Event::Eof) => return Ok(()),
            Ok(_) => {}
            Err(error) => {
                return Err(OfficePreflightSupport::invalid_archive(error.to_string()));
            }
        }
    }
}

fn inspect_element(
    name: &str,
    reader: &Reader<&[u8]>,
    event: &BytesStart<'_>,
) -> Result<(), OfficePreflightError> {
    let mut external = false;
    let mut target = String::new();
    for attribute in event.attributes() {
        let attribute = attribute
            .map_err(|error| OfficePreflightSupport::invalid_archive(error.to_string()))?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| OfficePreflightSupport::invalid_archive(error.to_string()))?
            .into_owned();
        match attribute.key.local_name().as_ref() {
            b"TargetMode" => external = value.eq_ignore_ascii_case("external"),
            b"Target" => target = value,
            _ => {}
        }
    }
    if external {
        return Err(OfficePreflightError::ExternalResourceBlocked {
            entry: name.to_owned(),
            target,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{OfficePreflightRelationships, inspect_xml, io_error, zip_error};
    use crate::multi_format::{
        OfficePreflightError, OfficePreflightLimits, OfficeResourceLimitKind,
    };
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;

    #[test]
    fn malformed_relationship_xml_is_an_invalid_archive() {
        assert!(matches!(
            zip_error(zip::result::ZipError::FileNotFound),
            OfficePreflightError::InvalidArchive { .. }
        ));
        assert!(matches!(
            io_error(std::io::Error::other("read failed")),
            OfficePreflightError::InvalidArchive { .. }
        ));
        assert!(matches!(
            inspect_xml("_rels/.rels", b"<Relationships><Relationship"),
            Err(OfficePreflightError::InvalidArchive { .. })
        ));
    }

    #[test]
    fn relationship_bytes_are_bounded_before_xml_parsing() -> Result<(), Box<dyn std::error::Error>>
    {
        let cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);
        writer.start_file("_rels/.rels", SimpleFileOptions::default())?;
        writer.write_all(b"<Relationships/>")?;
        let bytes = writer.finish()?.into_inner();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes.as_slice()))?;
        let mut limits = OfficePreflightLimits::strict();
        limits.max_relationship_bytes = 1;
        assert!(matches!(
            OfficePreflightRelationships::inspect(&mut archive, "_rels/.rels", limits),
            Err(OfficePreflightError::ResourceLimitExceeded {
                kind: OfficeResourceLimitKind::RelationshipBytes,
                ..
            })
        ));
        Ok(())
    }
}
