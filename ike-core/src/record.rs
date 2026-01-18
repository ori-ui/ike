use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    hash::{BuildHasherDefault, Hash, Hasher},
    ops::Deref,
    sync::{Arc, Weak},
};

use seahash::SeaHasher;

use crate::{Size, WidgetId, world::Widgets};

/// Settings that pertain to recording.
#[derive(Debug)]
pub struct RecordSettings {
    /// Threshold for when a widget should be recorded, common values `50..200`.
    ///
    /// Default is `65.0`.
    pub cost_threshold: f32,

    /// Threshold for the number of frame a recording can go unused.
    ///
    /// Default is `30`.
    pub max_frames_unused: u64,

    /// The maximum total memory used for recordings.
    ///
    /// Default is `96 MiB`.
    pub max_memory_usage: u64,
}

impl Default for RecordSettings {
    fn default() -> Self {
        Self {
            cost_threshold:    65.0,
            max_frames_unused: 30,
            max_memory_usage:  96 * 1024u64.pow(2),
        }
    }
}

#[derive(Debug)]
pub struct Recorder {
    memory_usage: u64,
    frame_count:  u64,
    entries:      HashMap<WidgetId, RecorderEntry, BuildSeaHasher>,
}

#[derive(Debug)]
struct RecorderEntry {
    recording:       Recording,
    frame_last_used: u64,
    cost_estimate:   f32,
}

type BuildSeaHasher = BuildHasherDefault<SeaHasher>;

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            memory_usage: 0,
            frame_count:  0,
            entries:      HashMap::default(),
        }
    }

    pub fn memory_usage(&self) -> u64 {
        self.memory_usage
    }

    pub fn insert(&mut self, widget: WidgetId, cost_estimate: f32, recording: Recording) {
        self.memory_usage += recording.memory;

        let entry = RecorderEntry {
            recording,
            frame_last_used: self.frame_count,
            cost_estimate,
        };

        self.entries.insert(widget, entry);
    }

    pub fn get_recording_unmarked(&self, widget: WidgetId) -> Option<&Recording> {
        let entry = self.entries.get(&widget)?;
        Some(&entry.recording)
    }

    pub fn get_last_frame_used(&self, widget: WidgetId) -> Option<u64> {
        let entry = self.entries.get(&widget)?;
        Some(entry.frame_last_used)
    }

    pub fn get_frames_unused(&self, widget: WidgetId) -> Option<u64> {
        let entry = self.entries.get(&widget)?;
        Some(self.frame_count - entry.frame_last_used)
    }

    pub fn get_cost_estimate(&self, widget: WidgetId) -> Option<f32> {
        let entry = self.entries.get(&widget)?;
        Some(entry.cost_estimate)
    }

    pub fn get_recording_marked(&mut self, widget: WidgetId) -> Option<Recording> {
        let entry = self.entries.get_mut(&widget)?;
        entry.frame_last_used = self.frame_count;
        Some(entry.recording.clone())
    }

    pub fn remove(&mut self, widget: WidgetId) {
        if let Some(entry) = self.entries.remove(&widget) {
            self.memory_usage -= entry.recording.memory;
        }
    }

    pub fn contains(&self, widget: WidgetId) -> bool {
        self.entries.contains_key(&widget)
    }

    pub fn clear(&mut self) {
        self.memory_usage = 0;
        self.frame_count = 0;
        self.entries.clear();
    }

    /// Remove recordings of destroyed widgets.
    pub(crate) fn cleanup(&mut self, widgets: &Widgets) {
        self.entries.retain(|id, _| widgets.contains(*id));
    }

    pub(crate) fn frame(&mut self, settings: &RecordSettings, widgets: &Widgets) {
        self.cleanup(widgets);
        self.frame_count += 1;

        self.entries.retain(|_widget, entry| {
            let frames_unused = self.frame_count - entry.frame_last_used;
            let should_remove = frames_unused >= settings.max_frames_unused;

            if should_remove {
                self.memory_usage -= entry.recording.memory;
            }

            !should_remove
        });

        self.cull_memory(settings);
    }

    /// Cull recordings if memory usage exceeds `3/4` of `max_memory_usage`.
    fn cull_memory(&mut self, settings: &RecordSettings) {
        let cull_threshold = settings.max_memory_usage * 3 / 4;

        if self.memory_usage <= cull_threshold {
            return;
        }

        tracing::warn!(
            "recorder memory usage exceeds 75% ({}/{}), consider increasing",
            DisplayMemorySize(self.memory_usage),
            DisplayMemorySize(settings.max_memory_usage)
        );

        let mut widgets = Vec::new();

        for (widget, entry) in self.entries.iter() {
            let weighted_cost = entry.recording.memory as f32 / entry.cost_estimate;

            widgets.push((*widget, weighted_cost));
        }

        widgets.sort_unstable_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(Ordering::Equal));

        while self.memory_usage > cull_threshold {
            let Some((widget, _)) = widgets.pop() else {
                break;
            };

            let entry = self
                .entries
                .remove(&widget)
                .expect("widget gotten from entries just above");

            self.memory_usage -= entry.recording.memory;
        }
    }
}

/// Helper type for displaying memory sizes, e.g. `32.3 MiB`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayMemorySize(pub u64);

impl fmt::Display for DisplayMemorySize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for DisplayMemorySize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(size) = *self;

        if size > 1024u64.pow(3) {
            let gibs = size as f32 / 1024f32.powi(3);
            write!(f, "{:.1} GiB", gibs)
        } else if size > 1024u64.pow(2) {
            let mibs = size as f32 / 1024f32.powi(2);
            write!(f, "{:.1} MiB", mibs)
        } else if size > 1024 {
            write!(f, "{:.1} KiB", size as f32 / 1024.0)
        } else {
            write!(f, "{} B", size)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recording {
    data: Arc<RecordingData>,
}

/// The data of a [`Recording`].
#[derive(Clone, Debug, PartialEq)]
pub struct RecordingData {
    /// Size in logical pixels.
    pub size: Size,

    /// Width in device pixels.
    pub width: u32,

    /// Height in device pixels.
    pub height: u32,

    /// Estimated memory usage, usually `width * height * 4`.
    pub memory: u64,
}

impl Recording {
    pub fn new(data: RecordingData) -> Self {
        Self {
            data: Arc::new(data),
        }
    }

    pub fn downgrade(this: &Self) -> WeakRecording {
        WeakRecording {
            data: Arc::downgrade(&this.data),
        }
    }
}

impl Deref for Recording {
    type Target = RecordingData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Clone, Debug)]
pub struct WeakRecording {
    data: Weak<RecordingData>,
}

impl WeakRecording {
    pub fn upgrade(&self) -> Option<Recording> {
        Some(Recording {
            data: self.data.upgrade()?,
        })
    }

    pub fn strong_count(&self) -> usize {
        self.data.strong_count()
    }
}

impl PartialEq for WeakRecording {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for WeakRecording {}

impl Hash for WeakRecording {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.as_ptr().hash(state);
    }
}
