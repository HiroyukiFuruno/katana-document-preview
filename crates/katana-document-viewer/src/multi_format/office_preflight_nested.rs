use super::{
    OfficeDocumentFormat, OfficeDocumentSource, ViewerSourceIdentity,
    office_preflight::{OfficePreflightError, OfficePreflightLimits, OfficePreflightSupport},
    office_preflight_archive::OfficePreflightArchive,
};
use std::io::{Cursor, Read};
use zip::ZipArchive;

pub(super) struct OfficeNestedPackages;

impl OfficeNestedPackages {
    pub(super) fn inspect(
        archive: &mut ZipArchive<Cursor<&[u8]>>,
        parent: &OfficeDocumentSource,
        packages: &[(String, OfficeDocumentFormat)],
        limits: OfficePreflightLimits,
        depth: usize,
    ) -> Result<(), OfficePreflightError> {
        for (name, format) in packages {
            let nested = Self::read(archive, parent, name, *format)?;
            OfficePreflightArchive::inspect(&nested, limits, depth + 1)?;
        }
        Ok(())
    }

    fn read(
        archive: &mut ZipArchive<Cursor<&[u8]>>,
        parent: &OfficeDocumentSource,
        name: &str,
        format: OfficeDocumentFormat,
    ) -> Result<OfficeDocumentSource, OfficePreflightError> {
        let mut entry = archive
            .by_name(name)
            .map_err(OfficePreflightSupport::archive_error)?;
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(OfficePreflightSupport::archive_error)?;
        Ok(Self::source(parent, name, format, bytes))
    }

    fn source(
        parent: &OfficeDocumentSource,
        name: &str,
        format: OfficeDocumentFormat,
        bytes: Vec<u8>,
    ) -> OfficeDocumentSource {
        let identity = ViewerSourceIdentity::new(
            format!("{}#{name}", parent.identity.uri),
            parent.identity.revision.clone(),
        );
        OfficeDocumentSource::new(
            identity,
            format,
            OfficePreflightSupport::expected_mime(format),
            bytes,
        )
    }
}
