use crate::{
    BorderWidth, Canvas, Clip, Color, CornerRadius, FontStretch, FontStyle, FontWeight, Offset,
    Paint, Paragraph, TextAlign, TextStyle, TextWrap, WidgetRef, WindowId, World,
    record::DisplayMemorySize,
};

pub(crate) fn recorder_overlay_window(world: &World, window: WindowId, canvas: &mut dyn Canvas) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers() {
        if let Ok(widget) = world.widget(layer.widget) {
            recorder_overlay_widget(&widget, canvas);
        }
    }

    let mut paragraph = Paragraph::new(1.0, TextAlign::Start, TextWrap::Word);
    paragraph.push(
        format!(
            "{} / {}",
            DisplayMemorySize(world.recorder().memory_usage()),
            DisplayMemorySize(world.settings().record.max_memory_usage),
        ),
        TextStyle {
            font_size:    16.0,
            font_family:  String::from("Inter Variable"),
            font_weight:  FontWeight::NORMAL,
            font_stretch: FontStretch::Normal,
            font_style:   FontStyle::Normal,
            paint:        Paint::from(Color::RED),
        },
    );

    canvas.draw_text(
        &paragraph,
        f32::INFINITY,
        Offset::all(4.0),
    );
}

pub(crate) fn recorder_overlay_widget(widget: &WidgetRef<'_>, canvas: &mut dyn Canvas) {
    for child in widget.cx.iter_children().flatten() {
        recorder_overlay_widget(&child, canvas);
    }

    if let Some(recording) = widget.cx.world.recorder.get_recording_unmarked(widget.id())
        && let Some(frames_unused) = widget.cx.world.recorder.get_frames_unused(widget.id())
        && let Some(cost_estimate) = widget.cx.world.recorder.get_cost_estimate(widget.id())
    {
        canvas.transform(
            widget.cx.global_transform(),
            &mut |canvas| {
                let spoilage = frames_unused as f32 // compute the stale the recording is
                    / widget.cx.world.settings.record.max_frames_unused as f32;

                let color = Color::GREEN.mix(Color::RED, spoilage);

                canvas.draw_rect(
                    widget.cx.rect().shrink(1.0),
                    CornerRadius::all(0.0),
                    &Paint::from(color.fade(0.2)),
                );

                canvas.draw_border(
                    widget.cx.rect(),
                    BorderWidth::all(1.0),
                    CornerRadius::all(0.0),
                    &Paint::from(Color::BLUE.fade(0.8)),
                );

                let mut paragraph = Paragraph::new(1.0, TextAlign::Start, TextWrap::Word);
                paragraph.push(
                    format!(
                        "{}x{} - {:.2}% mem - {:.2} cost",
                        recording.width,
                        recording.height,
                        recording.memory as f32 / widget.cx.world.recorder.memory_usage() as f32
                            * 100.0,
                        cost_estimate,
                    ),
                    TextStyle {
                        font_size:    12.0,
                        font_family:  String::from("Inter Variable"),
                        font_weight:  FontWeight::NORMAL,
                        font_stretch: FontStretch::Normal,
                        font_style:   FontStyle::Normal,
                        paint:        Paint::from(Color::BLUE.fade(0.8)),
                    },
                );

                canvas.clip(
                    &Clip::Rect(
                        widget.cx.rect().shrink(2.0),
                        CornerRadius::all(0.0),
                    ),
                    &mut |canvas| {
                        canvas.draw_text(
                            &paragraph,
                            widget.cx.width() - 4.0,
                            Offset::new(2.0, 0.0),
                        );
                    },
                );
            },
        );
    }
}
