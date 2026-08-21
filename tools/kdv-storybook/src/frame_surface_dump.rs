use std::io::{BufWriter, Write};
use std::path::Path;

const RGBA_CHANNELS: usize = 4;
const RGB_CHANNELS: usize = 3;

pub(super) struct SurfaceDump;

pub(super) struct SurfaceDumpImage<'a> {
    rgba: &'a [u8],
    width: usize,
    height: usize,
}

impl<'a> SurfaceDumpImage<'a> {
    pub(super) const fn new(rgba: &'a [u8], width: usize, height: usize) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }
}

impl SurfaceDump {
    pub(super) fn write_pair(
        directory: &Path,
        fixture: &str,
        reference: SurfaceDumpImage<'_>,
        preview: SurfaceDumpImage<'_>,
    ) -> std::io::Result<()> {
        std::fs::create_dir_all(directory)?;
        let label = fixture.replace(['/', '.'], "_");
        Self::write_ppm(
            &directory.join(format!("{label}_reference.ppm")),
            &reference,
        )?;
        Self::write_ppm(&directory.join(format!("{label}_preview.ppm")), &preview)
    }

    fn write_ppm(path: &Path, image: &SurfaceDumpImage<'_>) -> std::io::Result<()> {
        let expected_len = image
            .width
            .checked_mul(image.height)
            .and_then(|pixels| pixels.checked_mul(RGBA_CHANNELS))
            .ok_or_else(|| invalid_input("surface dimensions overflow"))?;
        if image.rgba.len() != expected_len {
            return Err(invalid_input(format!(
                "surface byte length {} does not match {}x{} RGBA",
                image.rgba.len(),
                image.width,
                image.height
            )));
        }
        let file = std::fs::File::create(path)?;
        let mut file = BufWriter::new(file);
        writeln!(file, "P6\n{} {}\n255", image.width, image.height)?;
        let mut rgb = Vec::with_capacity(image.width * image.height * RGB_CHANNELS);
        for pixel in image.rgba.as_chunks::<RGBA_CHANNELS>().0 {
            rgb.extend_from_slice(&pixel[..RGB_CHANNELS]);
        }
        file.write_all(&rgb)?;
        Ok(())
    }
}

fn invalid_input(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into())
}
