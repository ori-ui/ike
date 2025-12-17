use crate::android::{Event, InputQueueEvent, WindowEvent, context::Proxy};

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

fn get_proxy(activity: &ndk_sys::ANativeActivity) -> &Proxy {
    while activity.instance.is_null() {
        std::thread::yield_now();
    }

    unsafe { &*activity.instance.cast() }
}

unsafe extern "C" fn on_resume(activity: *mut ndk_sys::ANativeActivity) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::Resumed);
    };
}

unsafe extern "C" fn on_window_created(
    activity: *mut ndk_sys::ANativeActivity,
    window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::Window(WindowEvent::Created(
            window,
        )));
    };
}

unsafe extern "C" fn on_window_destroyed(
    activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::Window(WindowEvent::Destroyed));
    };
}

unsafe extern "C" fn on_window_redraw_needed(
    activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::Window(WindowEvent::Redraw));
    };
}

unsafe extern "C" fn on_window_resized(
    activity: *mut ndk_sys::ANativeActivity,
    _window: *mut ndk_sys::ANativeWindow,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::Window(WindowEvent::Resized));
    };
}

unsafe extern "C" fn on_window_focus_changed(
    activity: *mut ndk_sys::ANativeActivity,
    focused: i32,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::Window(
            WindowEvent::FocusChanged(focused > 0),
        ));
    };
}

unsafe extern "C" fn on_input_queue_created(
    activity: *mut ndk_sys::ANativeActivity,
    queue: *mut ndk_sys::AInputQueue,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::InputQueue(
            InputQueueEvent::Created(queue),
        ));
    }
}

unsafe extern "C" fn on_input_queue_destroyed(
    activity: *mut ndk_sys::ANativeActivity,
    queue: *mut ndk_sys::AInputQueue,
) {
    unsafe {
        let proxy = get_proxy(&*activity);
        proxy.send(Event::InputQueue(
            InputQueueEvent::Destroyed(queue),
        ));
    }
}
