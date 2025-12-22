use std::{ffi, ops::Range, ptr::NonNull};

use ike_core::{ImeSignal, Key, NamedKey};
use jni::{
    JNIEnv, JavaVM,
    objects::{JClass, JObject, JString},
};
use parking_lot::Mutex;

use crate::{Event, EventLoop, WindowState, context::Proxy};

#[derive(Debug)]
pub(super) enum ImeEvent {
    CommitText(String, usize),
    DeleteSurrounding(usize, usize),
    SendKeyEvent { key: Key, pressed: bool },
    SetSelection(usize, usize),
}

pub(super) struct Ime {
    proxy: Proxy,
    state: Mutex<ImeState>,
}

impl Ime {
    pub fn selection(&self) -> Range<usize> {
        self.state.lock().selection.clone()
    }

    pub fn composing(&self) -> Option<Range<usize>> {
        self.state.lock().composing.clone()
    }

    pub fn set_text(&self, text: String) {
        let mut state = self.state.lock();

        state.selection.start = state.selection.start.min(text.len());
        state.selection.end = state.selection.end.min(text.len());
        state.text = text;
    }

    pub fn set_selection(&self, mut selection: Range<usize>) {
        let mut state = self.state.lock();

        selection.start = selection.start.min(state.text.len());
        selection.end = selection.end.min(state.text.len());
        state.selection = selection;
    }

    pub fn set_composing(&self, composing: Option<Range<usize>>) {
        let mut state = self.state.lock();

        state.composing = composing;
    }

    pub fn index_n_chars_before(&self, n: usize) -> usize {
        let state = self.state.lock();

        let mut index = state.selection.start;

        for c in state.text[..index].chars().rev().take(n) {
            index -= c.len_utf8();
        }

        index
    }

    pub fn index_n_chars_after(&self, n: usize) -> usize {
        let state = self.state.lock();

        let mut index = state.selection.end;

        for c in state.text[index..].chars().take(n) {
            index += c.len_utf8();
        }

        index
    }
}

struct ImeState {
    text:      String,
    selection: Range<usize>,
    composing: Option<Range<usize>>,
}

impl<'a, T> EventLoop<'a, T> {
    pub fn handle_ime_event(&mut self, event: ImeEvent) {
        match event {
            ImeEvent::CommitText(text, new_cursor_position) => {
                tracing::trace!(
                    text,
                    new_cursor_position,
                    "ime commit text",
                );

                if let WindowState::Open(ref window) = self.window {
                    let Some(id) = window.id else {
                        return;
                    };

                    self.context.ime_commit_text(id, text);
                }
            }

            ImeEvent::DeleteSurrounding(before, after) => {
                tracing::trace!(before, after, "delete surrounding");

                if let WindowState::Open(ref window) = self.window {
                    let Some(id) = window.id else {
                        return;
                    };

                    let selection = self.ime.selection();

                    if selection.start != selection.end {
                        self.context.ime_commit_text(id, String::new());
                        return;
                    }

                    if before != 0 {
                        let start = self.ime.index_n_chars_before(before);
                        self.context.ime_select(id, start..selection.start);
                        self.context.ime_commit_text(id, String::new());
                    }

                    if after != 0 {
                        let end = self.ime.index_n_chars_after(after);
                        self.context.ime_select(id, selection.end..end);
                        self.context.ime_commit_text(id, String::new());
                    }
                }
            }

            ImeEvent::SendKeyEvent { key, pressed } => {
                tracing::trace!(?key, pressed, "ime send key event");

                if let WindowState::Open(ref window) = self.window {
                    let Some(id) = window.id else {
                        return;
                    };

                    (self.context).key_pressed(id, key, false, None, pressed);
                }
            }

            ImeEvent::SetSelection(start, end) => {
                tracing::trace!(start, end, "set selection");

                if let WindowState::Open(ref window) = self.window {
                    let Some(id) = window.id else {
                        return;
                    };

                    self.ime.set_selection(start..end);
                    self.context.ime_select(id, start..end);
                }
            }
        }
    }

    pub fn handle_ime_signal(&mut self, signal: ImeSignal) {
        match signal {
            ImeSignal::Start => {
                if let Ok(mut env) = self.jvm.attach_current_thread() {
                    if !self
                        .show_soft_input(&mut env, 0)
                        .is_ok_and(|success| success)
                    {
                        tracing::warn!("show soft input failed");
                    } else {
                        tracing::trace!("show soft input");
                    }
                }
            }

            ImeSignal::End => {
                if let Ok(mut env) = self.jvm.attach_current_thread() {
                    let _ = self.hide_soft_input(&mut env, 0);
                    tracing::trace!("hide soft input");
                }
            }

            ImeSignal::Area(..) => {}

            ImeSignal::Text(text) => {
                self.ime.set_text(text);

                if let Ok(mut env) = self.jvm.attach_current_thread() {
                    let selection = self.ime.selection();
                    let compose = self.ime.composing();

                    let _ = self.update_selection(
                        &mut env,
                        selection.start as i32,
                        selection.end as i32,
                        compose.as_ref().map_or(-1, |range| range.start as i32),
                        compose.as_ref().map_or(-1, |range| range.end as i32),
                    );
                }
            }

            ImeSignal::Selection {
                selection,
                composing: compose,
            } => {
                if self.ime.selection() == selection && self.ime.composing() == compose {
                    return;
                }

                if let Ok(mut env) = self.jvm.attach_current_thread() {
                    self.ime.set_selection(selection.clone());
                    self.ime.set_composing(compose.clone());

                    let _ = self.update_selection(
                        &mut env,
                        selection.start as i32,
                        selection.end as i32,
                        compose.as_ref().map_or(-1, |range| range.start as i32),
                        compose.as_ref().map_or(-1, |range| range.end as i32),
                    );

                    tracing::trace!(?selection, "update selection");
                }
            }
        }
    }

    fn show_soft_input(&self, env: &mut JNIEnv<'_>, flags: i32) -> jni::errors::Result<bool> {
        let activity = unsafe { native_activity(self.native_activity) };
        let view = rust_view(env, &activity)?;
        let imm = self.input_method_manager(env, &view)?;

        env.call_method(
            &imm,
            "showSoftInput",
            "(Landroid/view/View;I)Z",
            &[(&view).into(), flags.into()],
        )?
        .z()
    }

    fn hide_soft_input(&self, env: &mut JNIEnv<'_>, flags: i32) -> jni::errors::Result<bool> {
        let activity = unsafe { native_activity(self.native_activity) };
        let view = rust_view(env, &activity)?;
        let window = self.window_token(env, &view)?;
        let imm = self.input_method_manager(env, &view)?;

        env.call_method(
            imm,
            "hideSoftInputFromWindow",
            "(Landroid/os/IBinder;I)Z",
            &[(&window).into(), flags.into()],
        )?
        .z()
    }

    fn update_selection(
        &self,
        env: &mut JNIEnv<'_>,
        selection_start: i32,
        selection_end: i32,
        compose_start: i32,
        compose_end: i32,
    ) -> jni::errors::Result<()> {
        let activity = unsafe { native_activity(self.native_activity) };
        let view = rust_view(env, &activity)?;
        let imm = self.input_method_manager(env, &view)?;

        env.call_method(
            imm,
            "updateSelection",
            "(Landroid/view/View;IIII)V",
            &[
                (&view).into(),
                selection_start.into(),
                selection_end.into(),
                compose_start.into(),
                compose_end.into(),
            ],
        )?
        .v()
    }

    fn input_method_manager<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        rust_view: &JObject<'local>,
    ) -> jni::errors::Result<JObject<'local>> {
        env.get_field(
            rust_view,
            "inputMethodManager",
            "Landroid/view/inputmethod/InputMethodManager;",
        )?
        .l()
    }

    fn window_token<'local>(
        &self,
        env: &mut JNIEnv<'local>,
        rust_view: &JObject<'local>,
    ) -> jni::errors::Result<JObject<'local>> {
        env.call_method(
            rust_view,
            "getWindowToken",
            "()Landroid/os/IBinder;",
            &[],
        )?
        .l()
    }
}

pub unsafe fn init(
    jvm: &JavaVM,
    activity: NonNull<ndk_sys::ANativeActivity>,
    proxy: Proxy,
) -> jni::errors::Result<&'static Ime> {
    let activity = unsafe { native_activity(activity) };

    let mut env = jvm.attach_current_thread()?;
    let rust_view = rust_view(&mut env, &activity)?;

    let context = Ime {
        proxy,
        state: Mutex::new(ImeState {
            text:      String::new(),
            selection: 0..0,
            composing: None,
        }),
    };

    let context: &Ime = Box::leak(Box::new(context));

    env.set_field(
        &rust_view,
        "context",
        "J",
        (context as *const Ime as i64).into(),
    )?;

    let class_name = env.new_string("org.ori.RustView")?;
    let class_loader = env
        .call_method(
            activity,
            "getClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )?
        .l()?;

    let class = env
        .call_method(
            class_loader,
            "loadClass",
            "(Ljava/lang/String;)Ljava/lang/Class;",
            &[(&class_name).into()],
        )?
        .l()?;

    env.register_native_methods(
        JClass::from(class),
        &[
            jni::NativeMethod {
                name:   "getTextBeforeCursorNative".into(),
                sig:    "(II)Ljava/lang/String;".into(),
                fn_ptr: get_text_before_cursor as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "getTextAfterCursorNative".into(),
                sig:    "(II)Ljava/lang/String;".into(),
                fn_ptr: get_text_after_cursor as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "getSelectedTextNative".into(),
                sig:    "(I)Ljava/lang/String;".into(),
                fn_ptr: get_selected_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "commitTextNative".into(),
                sig:    "(Ljava/lang/String;I)Z".into(),
                fn_ptr: commit_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "deleteSurroundingTextNative".into(),
                sig:    "(II)Z".into(),
                fn_ptr: delete_surrounding_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "deleteSurroundingTextInCodePointsNative".into(),
                sig:    "(II)Z".into(),
                fn_ptr: delete_surrounding_in_code_points_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "setComposingTextNative".into(),
                sig:    "(Ljava/lang/String;I)Z".into(),
                fn_ptr: set_composing_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "sendKeyEventNative".into(),
                sig:    "(Landroid/view/KeyEvent;)Z".into(),
                fn_ptr: send_key_event as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "setSelectionNative".into(),
                sig:    "(II)Z".into(),
                fn_ptr: set_selection as *mut ffi::c_void,
            },
        ],
    )?;

    Ok(context)
}

fn rust_view<'local>(
    env: &mut JNIEnv<'local>,
    activity: &JObject<'local>,
) -> jni::errors::Result<JObject<'local>> {
    env.get_field(
        activity,
        "rustView",
        "Lorg/ori/RustView;",
    )?
    .l()
}

unsafe fn native_activity<'local>(
    native_activity: NonNull<ndk_sys::ANativeActivity>,
) -> JObject<'local> {
    unsafe { JObject::from_raw(native_activity.as_ref().clazz) }
}

unsafe fn get_ime<'local>(
    env: &mut JNIEnv<'local>,
    rust_view: &JObject<'local>,
) -> jni::errors::Result<&'local Ime> {
    let ptr = env.get_field(rust_view, "context", "J")?.j()?;

    unsafe { Ok(&*(ptr as usize as *const Ime)) }
}

unsafe fn get_proxy<'local>(
    env: &mut JNIEnv<'local>,
    rust_view: &JObject<'local>,
) -> jni::errors::Result<&'local Proxy> {
    unsafe { get_ime(env, rust_view).map(|cx| &cx.proxy) }
}

unsafe extern "C" fn get_text_before_cursor<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    n: i32,
    flags: i32,
) -> JString<'local> {
    tracing::trace!(n, flags, "get text before cursor");

    if let Ok(context) = unsafe { get_ime(&mut env, &rust_view) } {
        let start = context.index_n_chars_before(n as usize);
        let state = context.state.lock();
        let text = &state.text[start..state.selection.start];

        env.new_string(text)
            .unwrap_or_else(|_| JObject::null().into())
    } else {
        JObject::null().into()
    }
}

unsafe extern "C" fn get_text_after_cursor<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    n: i32,
    flags: i32,
) -> JString<'local> {
    tracing::trace!(n, flags, "get text after cursor");

    if let Ok(context) = unsafe { get_ime(&mut env, &rust_view) } {
        let end = context.index_n_chars_after(n as usize);
        let state = context.state.lock();
        let text = &state.text[state.selection.end..end];

        env.new_string(text)
            .unwrap_or_else(|_| JObject::null().into())
    } else {
        JObject::null().into()
    }
}

unsafe extern "C" fn get_selected_text<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    flags: i32,
) -> JString<'local> {
    tracing::trace!(flags, "get selected text");

    if let Ok(context) = unsafe { get_ime(&mut env, &rust_view) } {
        let state = context.state.lock();
        let start = state.selection.start.min(state.selection.len());
        let end = state.selection.end.min(state.selection.len());
        let text = &state.text[start..end];

        env.new_string(text)
            .unwrap_or_else(|_| JObject::null().into())
    } else {
        JObject::null().into()
    }
}

unsafe extern "C" fn commit_text<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    text: JString<'local>,
    new_cursor_position: i32,
) -> bool {
    if let Ok(proxy) = unsafe { get_proxy(&mut env, &rust_view) }
        && let Ok(text) = env.get_string(&text)
    {
        proxy.send(Event::Ime(ImeEvent::CommitText(
            text.to_string_lossy().to_string(),
            new_cursor_position as usize,
        )));

        true
    } else {
        false
    }
}

unsafe extern "C" fn delete_surrounding_text<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    before: i32,
    after: i32,
) -> bool {
    if let Ok(proxy) = unsafe { get_proxy(&mut env, &rust_view) } {
        proxy.send(Event::Ime(ImeEvent::DeleteSurrounding(
            before as usize,
            after as usize,
        )));

        true
    } else {
        false
    }
}

unsafe extern "C" fn delete_surrounding_in_code_points_text<'local>(
    _env: JNIEnv<'local>,
    _rust_view: JObject<'local>,
    _before: i32,
    _after: i32,
) -> bool {
    false
}

unsafe extern "C" fn set_composing_text<'local>(
    _env: JNIEnv<'local>,
    _rust_view: JObject<'local>,
    _text: JString<'local>,
    _new_cursor_position: i32,
) -> bool {
    tracing::info!("set composing text");

    false
}

unsafe extern "C" fn send_key_event<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    event: JObject<'local>,
) -> bool {
    if let Ok(proxy) = unsafe { get_proxy(&mut env, &rust_view) } {
        let keycode = env
            .call_method(&event, "getKeyCode", "()I", &[])
            .and_then(|v| v.i())
            .unwrap_or(ndk_sys::AKEYCODE_UNKNOWN as i32);

        let action = env
            .call_method(&event, "getAction", "()I", &[])
            .and_then(|v| v.i())
            .unwrap_or(ndk_sys::AKEY_EVENT_ACTION_DOWN as i32);

        let key = match keycode as u32 {
            ndk_sys::AKEYCODE_DEL => Key::Named(NamedKey::Backspace),
            ndk_sys::AKEYCODE_ENTER => Key::Named(NamedKey::Enter),
            _ => Key::Named(NamedKey::Unidentified),
        };

        let pressed = action as u32 == ndk_sys::AKEY_EVENT_ACTION_DOWN;

        proxy.send(Event::Ime(ImeEvent::SendKeyEvent {
            key,
            pressed,
        }));

        true
    } else {
        false
    }
}

unsafe extern "C" fn set_selection<'local>(
    mut env: JNIEnv<'local>,
    rust_view: JObject<'local>,
    start: i32,
    end: i32,
) -> bool {
    if let Ok(proxy) = unsafe { get_proxy(&mut env, &rust_view) } {
        proxy.send(Event::Ime(ImeEvent::SetSelection(
            start as usize,
            end as usize,
        )));

        true
    } else {
        false
    }
}
