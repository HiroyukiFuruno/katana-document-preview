use super::{PdfOutlineBuilder, decode_title};
use hayro::hayro_syntax::Pdf;
use hayro::hayro_syntax::object::{FromBytes, String as PdfString};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn outline_preserves_hierarchy_direct_destinations_and_actions() -> TestResult {
    let pdf = Pdf::new(pdf_with_outline()).map_err(|_| "test PDF should load")?;

    let items = PdfOutlineBuilder::build(&pdf);

    assert_eq!(items.len(), 3);
    assert_eq!((items[0].title.as_str(), items[0].level), ("Chapter", 1));
    assert_eq!((items[1].title.as_str(), items[1].level), ("Section", 2));
    assert_eq!((items[2].title.as_str(), items[2].level), ("日本", 1));
    assert!(items.iter().all(|item| item.page_index == Some(0)));
    Ok(())
}

#[test]
fn title_decoder_accepts_utf8_and_utf16be() -> TestResult {
    let ascii = PdfString::from_bytes(b"(Chapter)").ok_or("literal PDF string")?;
    let utf16 = PdfString::from_bytes(b"<FEFF65E5672C>").ok_or("hex PDF string")?;

    assert_eq!(decode_title(ascii), "Chapter");
    assert_eq!(decode_title(utf16), "日本");
    Ok(())
}

#[test]
fn outline_traversal_stops_at_cycles_and_depth_limit() -> TestResult {
    let cyclic = Pdf::new(pdf_with_cyclic_outline()).map_err(|_| "cyclic PDF should load")?;
    assert_eq!(PdfOutlineBuilder::build(&cyclic).len(), 2);

    let deep = Pdf::new(pdf_with_deep_outline()).map_err(|_| "deep PDF should load")?;
    assert_eq!(PdfOutlineBuilder::build(&deep).len(), 64);
    Ok(())
}

fn pdf_with_outline() -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R /Outlines 5 0 R >>".as_slice(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".as_slice(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources <<>> /Contents 4 0 R >>".as_slice(),
        b"<< /Length 0 >>\nstream\n\nendstream".as_slice(),
        b"<< /Type /Outlines /First 6 0 R /Last 7 0 R /Count 3 >>".as_slice(),
        b"<< /Title (Chapter) /Parent 5 0 R /Dest [3 0 R /Fit] /First 8 0 R /Last 8 0 R /Next 7 0 R >>".as_slice(),
        b"<< /Title <FEFF65E5672C> /Parent 5 0 R /A << /S /GoTo /D [3 0 R /Fit] >> >>".as_slice(),
        b"<< /Title (Section) /Parent 6 0 R /Dest [3 0 R /Fit] >>".as_slice(),
    ];
    let (mut bytes, offsets) = serialize_objects(&objects);
    let xref = bytes.len();
    bytes.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    bytes.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.into_iter().skip(1) {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}

fn pdf_with_cyclic_outline() -> Vec<u8> {
    let objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R /Outlines 5 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources <<>> /Contents 4 0 R >>"
            .to_vec(),
        b"<< /Length 0 >>\nstream\n\nendstream".to_vec(),
        b"<< /Type /Outlines /First 6 0 R /Last 7 0 R /Count 2 >>".to_vec(),
        b"<< /Title (Cycle A) /Parent 5 0 R /First 7 0 R /Next 7 0 R >>".to_vec(),
        b"<< /Title (Cycle B) /Parent 5 0 R /Next 6 0 R >>".to_vec(),
    ];
    serialize_owned_objects(&objects)
}

fn pdf_with_deep_outline() -> Vec<u8> {
    let mut objects = vec![
        b"<< /Type /Catalog /Pages 2 0 R /Outlines 5 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources <<>> /Contents 4 0 R >>"
            .to_vec(),
        b"<< /Length 0 >>\nstream\n\nendstream".to_vec(),
        b"<< /Type /Outlines /First 6 0 R /Last 6 0 R /Count 65 >>".to_vec(),
    ];
    for index in 0..65 {
        let object_id = index + 6;
        let child = if index < 64 {
            format!(" /First {} 0 R", object_id + 1)
        } else {
            String::new()
        };
        objects.push(format!("<< /Title (Level {index}) /Parent 5 0 R{child} >>").into_bytes());
    }
    serialize_owned_objects(&objects)
}

fn serialize_owned_objects(objects: &[Vec<u8>]) -> Vec<u8> {
    let borrowed = objects.iter().map(Vec::as_slice).collect::<Vec<_>>();
    let (mut bytes, offsets) = serialize_objects(&borrowed);
    let xref = bytes.len();
    bytes.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    bytes.extend_from_slice(b"0000000000 65535 f \n");
    for offset in offsets.into_iter().skip(1) {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}

fn serialize_objects(objects: &[&[u8]]) -> (Vec<u8>, Vec<usize>) {
    let mut bytes = b"%PDF-1.7\n%KDV\n".to_vec();
    let mut offsets = vec![0];
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        bytes.extend_from_slice(object);
        bytes.extend_from_slice(b"\nendobj\n");
    }
    (bytes, offsets)
}
