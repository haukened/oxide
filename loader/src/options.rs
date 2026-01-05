use uefi::{
    boot::{OpenProtocolAttributes, OpenProtocolParams, image_handle, open_protocol},
    proto::loaded_image::LoadedImage,
};

use crate::writer::FixedBufWriter;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Boolean boot options parsed from the loader command line. Kept minimal for handoff.
pub struct BootOptions {
    pub debug: bool,
    pub quiet: bool,
}

/// Inspect the UEFI load options and extract simple boolean boot options.
///
/// Returns `BootOptions::default()` if options are absent or malformed so the
/// loader stays resilient to firmware quirks.
pub fn get_boot_options() -> BootOptions {
    let image_handle = image_handle();
    let loaded_image = unsafe {
        open_protocol::<LoadedImage>(
            OpenProtocolParams {
                handle: image_handle,
                agent: image_handle,
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .unwrap()
    };
    let opts16 = match loaded_image.load_options_as_cstr16() {
        Ok(opts) => opts,
        Err(_) => {
            // no load options provided
            return BootOptions::default();
        }
    };

    let mut buf = [0u8; 256];
    let mut writer = FixedBufWriter::new(&mut buf);

    if opts16.as_str_in_buf(&mut writer).is_err() {
        // truncated or failed conversion; ignore to avoid parsing partial tokens
        return BootOptions::default();
    }
    let len = writer.len();

    let cmdline = core::str::from_utf8(&buf[..len]).unwrap_or("");

    let mut options = BootOptions::default();

    for token in cmdline.split_whitespace() {
        match token {
            "debug" => options.debug = true,
            "quiet" => options.quiet = true,
            _ => {
                // ignore unknown flags
            }
        }
    }

    options
}
