use ::image::{DynamicImage, ImageOutputFormat};
use std::io::Cursor;

pub trait PngBytes {
    fn png_bytes(&self) -> anyhow::Result<Vec<u8>>;
}

impl PngBytes for DynamicImage {
    fn png_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut image_bytes = Cursor::new(Vec::<u8>::new());
        self.write_to(&mut image_bytes, ImageOutputFormat::Png)?;
        Ok(image_bytes.into_inner())
    }
}
