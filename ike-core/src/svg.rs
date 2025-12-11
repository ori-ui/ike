use std::{
    borrow::Cow,
    fmt,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, Weak},
};

#[macro_export]
macro_rules! include_svg {
    ($path:literal) => {
        $crate::SvgData::from_bytes_static(::std::include_bytes!($path))
    };
}

#[derive(Clone, Debug, PartialEq)]
pub struct Svg {
    data: Arc<SvgData>,
}

impl Svg {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let data = SvgData {
            bytes: bytes.to_vec().into(),
        };

        Self {
            data: Arc::new(data),
        }
    }

    pub fn downgrade(this: &Self) -> WeakSvg {
        WeakSvg {
            data: Arc::downgrade(&this.data),
        }
    }
}

impl Deref for Svg {
    type Target = SvgData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Svg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.data)
    }
}

impl From<SvgData> for Svg {
    fn from(data: SvgData) -> Self {
        Svg {
            data: Arc::new(data),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeakSvg {
    data: Weak<SvgData>,
}

impl WeakSvg {
    pub fn upgrade(&self) -> Option<Svg> {
        Some(Svg {
            data: self.data.upgrade()?,
        })
    }

    pub fn strong_count(&self) -> usize {
        self.data.strong_count()
    }
}

impl PartialEq for WeakSvg {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for WeakSvg {}

impl Hash for WeakSvg {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.as_ptr().hash(state);
    }
}

#[derive(Clone, PartialEq)]
pub struct SvgData {
    bytes: Cow<'static, [u8]>,
}

impl SvgData {
    pub const fn from_bytes_static(bytes: &'static [u8]) -> SvgData {
        SvgData {
            bytes: Cow::Borrowed(bytes),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Debug for SvgData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SvgData").finish_non_exhaustive()
    }
}
