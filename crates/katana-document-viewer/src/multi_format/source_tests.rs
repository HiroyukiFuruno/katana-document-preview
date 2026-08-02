use super::{
    BinaryDocumentSource, OfficeDocumentFormat, OfficeDocumentSource, ViewerSource,
    ViewerSourceIdentity,
};

#[test]
fn owned_source_values_roundtrip_through_every_accessor() {
    let pdf_identity = ViewerSourceIdentity::new(
        "file:///owned.pdf".to_owned(),
        "sha256:owned-pdf".to_owned(),
    );
    let pdf = ViewerSource::Pdf(BinaryDocumentSource::new(
        pdf_identity.clone(),
        "application/pdf".to_owned(),
        vec![1, 2],
    ));
    assert_eq!(&pdf_identity, pdf.identity());
    assert_eq!("application/pdf", pdf.mime());
    assert_eq!(&[1, 2], pdf.bytes());
    assert_eq!(None, pdf.office_format());

    let office = ViewerSource::Office(OfficeDocumentSource::new(
        ViewerSourceIdentity::new("file:///owned.xlsx", "sha256:owned-xlsx"),
        OfficeDocumentFormat::Xlsx,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_owned(),
        vec![3, 4],
    ));
    assert_eq!("file:///owned.xlsx", office.identity().uri);
    assert_eq!(
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        office.mime()
    );
    assert_eq!(&[3, 4], office.bytes());
    assert_eq!(Some(OfficeDocumentFormat::Xlsx), office.office_format());
}
