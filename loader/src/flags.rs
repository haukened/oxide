use uefi::{
    boot::{OpenProtocolAttributes, OpenProtocolParams, image_handle, open_protocol},
    proto::loaded_image::LoadedImage,
};

use crate::writer::FixedBufWriter;

#[allow(dead_code)]
pub struct BootFlags {
    pub debug: bool,
    pub quiet: bool,
}

impl Default for BootFlags {
    fn default() -> Self {
        BootFlags {
            debug: false,
            quiet: false,
        }
    }
}

pub fn get_boot_flags() -> BootFlags {
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
            return BootFlags::default();
        }
    };

    let mut buf = [0u8; 256];
    let mut writer = FixedBufWriter::new(&mut buf);

    let _ = opts16.as_str_in_buf(&mut writer);
    let len = writer.len();

    let cmdline = core::str::from_utf8(&buf[..len]).unwrap_or("");

    let mut flags = BootFlags::default();

    for token in cmdline.split_whitespace() {
        match token {
            "debug" => flags.debug = true,
            "quiet" => flags.quiet = true,
            _ => {
                // ignore unknown flags
            }
        }
    }

    flags
}
