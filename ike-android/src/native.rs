use std::{ffi, ptr::NonNull};

use jni::{
    JNIEnv, JavaVM,
    objects::{JClass, JObject},
};

use crate::{ime, window};

pub unsafe fn init(
    jvm: &JavaVM,
    activity: NonNull<ndk_sys::ANativeActivity>,
) -> jni::errors::Result<()> {
    let activity = unsafe { native_activity(activity) };

    let mut env = jvm.attach_current_thread()?;

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
                name:   "onApplyWindowInsetsNative".into(),
                sig:    "(IIIIIIIIIIII)V".into(),
                fn_ptr: window::on_apply_window_insets as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "getTextBeforeCursorNative".into(),
                sig:    "(II)Ljava/lang/String;".into(),
                fn_ptr: ime::get_text_before_cursor as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "getTextAfterCursorNative".into(),
                sig:    "(II)Ljava/lang/String;".into(),
                fn_ptr: ime::get_text_after_cursor as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "getSelectedTextNative".into(),
                sig:    "(I)Ljava/lang/String;".into(),
                fn_ptr: ime::get_selected_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "commitTextNative".into(),
                sig:    "(Ljava/lang/String;I)Z".into(),
                fn_ptr: ime::commit_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "deleteSurroundingTextNative".into(),
                sig:    "(II)Z".into(),
                fn_ptr: ime::delete_surrounding_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "deleteSurroundingTextInCodePointsNative".into(),
                sig:    "(II)Z".into(),
                fn_ptr: ime::delete_surrounding_in_code_points_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "setComposingTextNative".into(),
                sig:    "(Ljava/lang/String;I)Z".into(),
                fn_ptr: ime::set_composing_text as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "sendKeyEventNative".into(),
                sig:    "(Landroid/view/KeyEvent;)Z".into(),
                fn_ptr: ime::send_key_event as *mut ffi::c_void,
            },
            jni::NativeMethod {
                name:   "setSelectionNative".into(),
                sig:    "(II)Z".into(),
                fn_ptr: ime::set_selection as *mut ffi::c_void,
            },
        ],
    )?;

    Ok(())
}

pub fn rust_view<'local>(
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

pub unsafe fn native_activity<'local>(
    native_activity: NonNull<ndk_sys::ANativeActivity>,
) -> JObject<'local> {
    unsafe { JObject::from_raw(native_activity.as_ref().clazz) }
}
