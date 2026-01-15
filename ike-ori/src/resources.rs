use std::any::{Any, TypeId};

use ori::Provider;

pub struct Resources {
    entries: Vec<Entry>,
}

struct Entry {
    value:   Box<dyn Any>,
    type_id: TypeId,
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

impl Resources {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl Provider for Resources {
    fn push<T: Any>(&mut self, context: Box<T>) {
        self.entries.push(Entry {
            value:   context,
            type_id: TypeId::of::<T>(),
        })
    }

    fn pop<T: Any>(&mut self) -> Option<Box<T>> {
        self.entries.pop()?.value.downcast().ok()
    }

    fn get<T: Any>(&self) -> Option<&T> {
        let entry = self
            .entries
            .iter()
            .rfind(|e| e.type_id == TypeId::of::<T>())?;

        Some(unsafe { &*(entry.value.as_ref() as *const _ as *const T) })
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let entry = self
            .entries
            .iter_mut()
            .rfind(|e| e.type_id == TypeId::of::<T>())?;

        Some(unsafe { &mut *(entry.value.as_mut() as *mut _ as *mut T) })
    }
}
