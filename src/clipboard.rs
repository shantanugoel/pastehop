use std::{env, fs, path::PathBuf};

use image::{ColorType, ImageFormat};
use tempfile::NamedTempFile;

use crate::errors::PasteHopError;

pub struct ClipboardImage {
    pub file: NamedTempFile,
    pub size_bytes: u64,
}

pub fn read_clipboard_image() -> Result<ClipboardImage, PasteHopError> {
    if env::var_os("PH_FAKE_CLIPBOARD_TEXT").is_some() {
        return Err(PasteHopError::ClipboardNotImage);
    }

    if let Some(fake_path) = env::var_os("PH_FAKE_CLIPBOARD_IMAGE") {
        return materialize_png_from_file(PathBuf::from(fake_path));
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|source| PasteHopError::ClipboardUnavailable {
            message: source.to_string(),
        })?;
    let image = clipboard
        .get_image()
        .map_err(|_| PasteHopError::ClipboardNotImage)?;
    let bytes = image.bytes.into_owned();

    let file = NamedTempFile::new().map_err(|source| PasteHopError::ClipboardIo { source })?;
    image::save_buffer_with_format(
        file.path(),
        &bytes,
        image.width as u32,
        image.height as u32,
        ColorType::Rgba8,
        ImageFormat::Png,
    )
    .map_err(|source| PasteHopError::ClipboardImageEncoding { source })?;

    let size_bytes = fs::metadata(file.path())
        .map_err(|source| PasteHopError::ClipboardIo { source })?
        .len();

    Ok(ClipboardImage { file, size_bytes })
}

pub fn write_clipboard_text(text: &str) -> Result<(), PasteHopError> {
    if let Some(fake_path) = env::var_os("PH_FAKE_CLIPBOARD_WRITE_PATH") {
        fs::write(fake_path, text).map_err(|source| PasteHopError::ClipboardIo { source })?;
        return Ok(());
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|source| PasteHopError::ClipboardUnavailable {
            message: source.to_string(),
        })?;
    clipboard
        .set_text(text.to_owned())
        .map_err(|source| PasteHopError::ClipboardUnavailable {
            message: source.to_string(),
        })
}

fn materialize_png_from_file(path: PathBuf) -> Result<ClipboardImage, PasteHopError> {
    let file = NamedTempFile::new().map_err(|source| PasteHopError::ClipboardIo { source })?;
    let image =
        image::open(&path).map_err(|source| PasteHopError::ClipboardImageEncoding { source })?;
    image
        .save_with_format(file.path(), ImageFormat::Png)
        .map_err(|source| PasteHopError::ClipboardImageEncoding { source })?;

    let size_bytes = fs::metadata(file.path())
        .map_err(|source| PasteHopError::ClipboardIo { source })?
        .len();

    Ok(ClipboardImage { file, size_bytes })
}

#[cfg(test)]
mod tests {
    use std::{env, fs};

    use image::{ImageBuffer, ImageFormat, Rgba};
    use tempfile::TempDir;

    use super::{materialize_png_from_file, write_clipboard_text};

    #[test]
    fn materializes_png_from_source_image() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let source = temp_dir.path().join("clipboard.png");
        let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(2, 2, |_, _| Rgba([10, 20, 30, 255]));
        buffer.save(&source).expect("source image should save");

        let clipboard = materialize_png_from_file(source).expect("png should materialize");

        assert!(clipboard.size_bytes > 0);
        assert!(clipboard.file.path().exists());
        assert_eq!(
            image::guess_format(
                &std::fs::read(clipboard.file.path()).expect("clipboard output should exist")
            )
            .expect("format should be detected"),
            ImageFormat::Png
        );
    }

    #[test]
    fn writes_text_to_fake_clipboard_sink() {
        let temp_dir = TempDir::new().expect("temp dir should exist");
        let sink = temp_dir.path().join("clipboard.txt");

        unsafe {
            env::set_var("PH_FAKE_CLIPBOARD_WRITE_PATH", &sink);
        }
        let result = write_clipboard_text("~/remote/path.png");
        unsafe {
            env::remove_var("PH_FAKE_CLIPBOARD_WRITE_PATH");
        }

        result.expect("clipboard write should succeed");
        assert_eq!(
            fs::read_to_string(&sink).expect("clipboard sink should be written"),
            "~/remote/path.png"
        );
    }
}
