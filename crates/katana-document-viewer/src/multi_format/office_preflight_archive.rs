use super::{
    OfficeDocumentFormat, OfficeDocumentSource,
    office_preflight::{
        MAX_NESTED_PACKAGE_DEPTH, OfficePreflightError, OfficePreflightLimits,
        OfficePreflightReport, OfficePreflightSupport, OfficeResourceLimitKind,
    },
    office_preflight_nested::OfficeNestedPackages,
    office_preflight_policy::OfficePreflightPolicy,
    office_preflight_relationships::OfficePreflightRelationships,
    office_preflight_zip_entries::OfficeZipEntries,
};
use std::collections::HashSet;
use std::io::Cursor;
use zip::ZipArchive;
struct ArchiveScan {
    names: HashSet<String>,
    relationships: Vec<String>,
    nested: Vec<(String, OfficeDocumentFormat)>,
    compressed_bytes: u64,
    uncompressed_bytes: u64,
    has_main_part: bool,
    diagnostics: Vec<super::ViewerDiagnostic>,
}
impl ArchiveScan {
    fn new(entry_count: usize) -> Self {
        Self {
            names: HashSet::with_capacity(entry_count),
            relationships: Vec::new(),
            nested: Vec::new(),
            compressed_bytes: 0,
            uncompressed_bytes: 0,
            has_main_part: false,
            diagnostics: Vec::new(),
        }
    }
}

pub(crate) struct OfficePreflightArchive;

impl OfficePreflightArchive {
    pub(crate) fn inspect(
        source: &OfficeDocumentSource,
        limits: OfficePreflightLimits,
        depth: usize,
    ) -> Result<(OfficePreflightReport, Vec<super::ViewerDiagnostic>), OfficePreflightError> {
        validate_depth(depth)?;
        OfficePreflightPolicy::validate_source(source, limits)?;
        let mut archive = open_archive(source)?;
        OfficePreflightPolicy::validate_entry_count(archive.len(), limits)?;
        let scan = scan_archive(&mut archive, source.format, limits)?;
        OfficeZipEntries::validate(source.bytes.as_slice(), &mut archive)?;
        validate_main_part(source.format, scan.has_main_part)?;
        let external_relationship_count =
            inspect_relationships(&mut archive, &scan.relationships, limits)?;
        OfficeNestedPackages::inspect(&mut archive, source, &scan.nested, limits, depth)?;
        Ok((
            OfficePreflightReport {
                entry_count: archive.len(),
                total_compressed_bytes: scan.compressed_bytes,
                total_uncompressed_bytes: scan.uncompressed_bytes,
                external_relationship_count,
            },
            scan.diagnostics,
        ))
    }
}

fn validate_depth(depth: usize) -> Result<(), OfficePreflightError> {
    if depth > MAX_NESTED_PACKAGE_DEPTH {
        return Err(OfficePreflightSupport::resource_limit(
            OfficeResourceLimitKind::EntryCount,
            depth as u64,
            MAX_NESTED_PACKAGE_DEPTH as u64,
            None,
        ));
    }
    Ok(())
}

fn open_archive(
    source: &OfficeDocumentSource,
) -> Result<ZipArchive<Cursor<&[u8]>>, OfficePreflightError> {
    ZipArchive::new(Cursor::new(source.bytes.as_slice()))
        .map_err(OfficePreflightSupport::archive_error)
}

fn scan_archive(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    format: OfficeDocumentFormat,
    limits: OfficePreflightLimits,
) -> Result<ArchiveScan, OfficePreflightError> {
    let mut scan = ArchiveScan::new(archive.len());
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(OfficePreflightSupport::archive_error)?;
        record_entry(
            &mut scan,
            entry.name(),
            entry.compressed_size(),
            entry.size(),
            format,
            limits,
        )?;
    }
    Ok(scan)
}

fn record_entry(
    scan: &mut ArchiveScan,
    name: &str,
    compressed: u64,
    uncompressed: u64,
    format: OfficeDocumentFormat,
    limits: OfficePreflightLimits,
) -> Result<(), OfficePreflightError> {
    OfficePreflightPolicy::validate_entry(name, compressed, uncompressed, limits)?;
    if OfficePreflightPolicy::active_content_entry(name) {
        let entry = name.to_owned();
        let error = OfficePreflightError::ActiveContentBlocked { entry };
        scan.diagnostics.push(error.diagnostic());
    }
    if !scan.names.insert(name.to_owned()) {
        return Err(OfficePreflightSupport::invalid_archive(format!(
            "duplicate entry `{name}`"
        )));
    }
    update_totals(scan, compressed, uncompressed, limits)?;
    scan.has_main_part |= name == OfficePreflightPolicy::main_part(format);
    collect_auxiliary_entry(scan, name);
    Ok(())
}

fn update_totals(
    scan: &mut ArchiveScan,
    compressed: u64,
    uncompressed: u64,
    limits: OfficePreflightLimits,
) -> Result<(), OfficePreflightError> {
    scan.compressed_bytes = OfficePreflightPolicy::checked_total(
        scan.compressed_bytes,
        compressed,
        OfficeResourceLimitKind::SourceBytes,
        limits.max_source_bytes,
    )?;
    scan.uncompressed_bytes = OfficePreflightPolicy::checked_total(
        scan.uncompressed_bytes,
        uncompressed,
        OfficeResourceLimitKind::TotalUncompressedBytes,
        limits.max_total_uncompressed_bytes,
    )?;
    Ok(())
}

fn collect_auxiliary_entry(scan: &mut ArchiveScan, name: &str) {
    if OfficePreflightPolicy::relationship_entry(name) {
        scan.relationships.push(name.to_owned());
    }
    if let Some(format) = OfficePreflightPolicy::nested_package_format(name) {
        scan.nested.push((name.to_owned(), format));
    }
}

fn validate_main_part(
    format: OfficeDocumentFormat,
    found: bool,
) -> Result<(), OfficePreflightError> {
    if !found {
        return Err(OfficePreflightSupport::invalid_archive(format!(
            "required main part `{}` is missing",
            OfficePreflightPolicy::main_part(format)
        )));
    }
    Ok(())
}

fn inspect_relationships(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    names: &[String],
    limits: OfficePreflightLimits,
) -> Result<usize, OfficePreflightError> {
    let mut external_relationship_count = 0;
    for name in names {
        external_relationship_count +=
            OfficePreflightRelationships::inspect(archive, name, limits)?;
    }
    Ok(external_relationship_count)
}

#[cfg(test)]
#[path = "office_preflight_archive_tests.rs"]
mod tests;
