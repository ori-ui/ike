use crate::{Event, GLOBAL_STATE, InputQueueEvent, WindowEvent};

pub fn register_callbacks(activity: &mut ndk_sys::ANativeActivity) {
    let callbacks = unsafe { &mut *activity.callbacks };

    callbacks.onResume = Some(on_resume);
    callbacks.onNativeWindowCreated = Some(on_window_created);
    callbacks.onNativeWindowDestroyed = Some(on_window_destroyed);
    callbacks.onNativeWindowRedrawNeeded = Some(on_window_redraw_needed);
    callbacks.onNativeWindowResized = Some(on_window_resized);
    callbacks.onWindowFocusChanged = Some(on_window_focus_changed);
    callbacks.onInputQueueCreated = Some(on_input_queue_created);
    callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);
}

fn send(event: Event) {
    let global_state = GLOBAL_STATE.get().expect("GLOBAL_STATE has not been set");
    let _ = global_state.sender.send(event);

    if let Some(ref wake) = *global_state.waker.lock() {
        wake();
    }
}

unsafe extern "C" fn on_resume(_activity: *mut ndk_sys::ANativeActivity) {
    send(Event::Resumed);
}

unsafe extern "C" fn on_window_created(
    _activity: *mut ndk_sys::ANativeActivity,
    window: *mut ndk_sys::ANativeWindow,
) {
    send(Event::Window(WindowEvent::Created(
        window,
    )));
}

unsafe extern "C" fn on_window_destroyed(
    _activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    send(Event::Window(WindowEvent::Destroyed));
}

unsafe extern "C" fn on_window_redraw_needed(
    _activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    send(Event::Window(WindowEvent::Redraw));
}

unsafe extern "C" fn on_window_resized(
    _activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    send(Event::Window(WindowEvent::Resized));
}

unsafe extern "C" fn on_window_focus_changed(
    _activity: *mut ndk_sys::ANativeActivity,
    focused: i32,
) {
    send(Event::Window(
        WindowEvent::FocusChanged(focused > 0),
    ));
}

unsafe extern "C" fn on_input_queue_created(
    _activity: *mut ndk_sys::ANativeActivity,
    queue: *mut ndk_sys::AInputQueue,
) {
    send(Event::InputQueue(
        InputQueueEvent::Created(queue),
    ));
}

unsafe extern "C" fn on_input_queue_destroyed(
    _activity: *mut ndk_sys::ANativeActivity,
    queue: *mut ndk_sys::AInputQueue,
) {
    send(Event::InputQueue(
        InputQueueEvent::Destroyed(queue),
    ));
}
