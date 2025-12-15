use std::{ffi::CString, io};

use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

pub struct MakeAndroidWriter;

impl MakeWriter<'_> for MakeAndroidWriter {
    type Writer = AndroidWriter;

    fn make_writer(&'_ self) -> Self::Writer {
        AndroidWriter {
            level: ndk_sys::android_LogPriority::ANDROID_LOG_UNKNOWN,
        }
    }

    fn make_writer_for(&'_ self, meta: &Metadata<'_>) -> Self::Writer {
        AndroidWriter {
            level: match *meta.level() {
                Level::TRACE => ndk_sys::android_LogPriority::ANDROID_LOG_SILENT,
                Level::INFO => ndk_sys::android_LogPriority::ANDROID_LOG_INFO,
                Level::DEBUG => ndk_sys::android_LogPriority::ANDROID_LOG_DEBUG,
                Level::WARN => ndk_sys::android_LogPriority::ANDROID_LOG_WARN,
                Level::ERROR => ndk_sys::android_LogPriority::ANDROID_LOG_ERROR,
            },
        }
    }
}

pub struct AndroidWriter {
    level: ndk_sys::android_LogPriority,
}

impl io::Write for AndroidWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let message = CString::new(buf.to_vec())?;

        unsafe {
            ndk_sys::__android_log_print(
                self.level.0 as i32,
                c"rust".as_ptr(),
                message.as_ptr(),
            );
        };

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
