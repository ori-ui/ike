use crate::android::{Event, context::Proxy};

pub fn register_callbacks(activity: &mut ndk_sys::ANativeActivity) {
    let callbacks = unsafe { &mut *activity.callbacks };
    callbacks.onNativeWindowCreated = Some(on_window_created);
    callbacks.onNativeWindowDestroyed = Some(on_window_destroyed);
    callbacks.onNativeWindowRedrawNeeded = Some(on_window_redraw_needed);
    callbacks.onNativeWindowResized = Some(on_window_resized);
    callbacks.onInputQueueCreated = Some(on_input_queue_created);
    callbacks.onInputQueueDestroyed = Some(on_input_queue_destroyed);
    callbacks.onWindowFocusChanged = Some(on_window_focus_changed);
}

fn get_proxy(activity: &ndk_sys::ANativeActivity) -> &Proxy {
    while activity.instance.is_null() {
        std::thread::yield_now();
    }

    unsafe { &*activity.instance.cast() }
}

unsafe extern "C" fn on_window_created(
    activity: *mut ndk_sys::ANativeActivity,
    window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::WindowCreated(window));
    };
}

unsafe extern "C" fn on_window_destroyed(
    activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::WindowDestroyed);
    };
}

unsafe extern "C" fn on_window_redraw_needed(
    activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::WindowRedraw);
    };
}

unsafe extern "C" fn on_window_resized(
    activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::WindowResized);
    };
}

unsafe extern "C" fn on_window_focus_changed(
    activity: *mut ndk_sys::ANativeActivity,
    focused: i32,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::WindowFocusChanged(focused > 0));
    };
}

unsafe extern "C" fn on_input_queue_created(
    activity: *mut ndk_sys::ANativeActivity,
    queue: *mut ndk_sys::AInputQueue,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::InputQueueCreated(queue));
    }
}

unsafe extern "C" fn on_input_queue_destroyed(
    activity: *mut ndk_sys::ANativeActivity,
    queue: *mut ndk_sys::AInputQueue,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::InputQueueDestroyed(queue));
    }
}
