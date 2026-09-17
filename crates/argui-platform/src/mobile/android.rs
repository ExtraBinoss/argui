use std::sync::{
    Arc, Mutex, MutexGuard, OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use android_activity::AndroidApp;
use argui_core::ColorScheme;
use jni::{
    Env, JValue, JavaVM, jni_sig, jni_str,
    objects::{Global, JClass, JObject},
};

use super::{MobileActivityError, PhysicalInsets};

struct AndroidContext {
    app: AndroidApp,
    vm: JavaVM,
    activity: Global<JObject<'static>>,
    helper: Mutex<Option<Global<JClass<'static>>>>,
    soft_input_visible: AtomicBool,
    system_bar_scheme: AtomicU8,
}

static CONTEXT: OnceLock<Mutex<Option<Arc<AndroidContext>>>> = OnceLock::new();

/// Returns the process-wide lock protecting the retained Android JNI context.
fn context_lock() -> &'static Mutex<Option<Arc<AndroidContext>>> {
    CONTEXT.get_or_init(|| Mutex::new(None))
}

/// Locks the Android context and recovers it if another caller panicked while holding the lock.
fn lock_context() -> MutexGuard<'static, Option<Arc<AndroidContext>>> {
    context_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Initializes JNI state and retains the active NativeActivity without requiring app Java code.
#[allow(unsafe_code)]
pub(super) fn initialize(android_app: &AndroidApp) -> Result<(), String> {
    // SAFETY: android-activity returns the process JVM pointer and guarantees it remains valid
    // for the NativeActivity lifetime; this context only uses it while the app process is alive.
    let vm = unsafe { JavaVM::from_raw(android_app.vm_as_ptr().cast()) };
    let app = android_app.clone();
    let created = vm
        .attach_current_thread_for_scope(|env| -> jni::errors::Result<AndroidContext> {
            let raw_activity = android_app.activity_as_ptr().cast();
            // SAFETY: android-activity documents this as an unowned global reference valid while
            // `android_app` is alive. `new_global_ref` below creates our own retained reference.
            let borrowed_activity =
                unsafe { env.as_cast_raw::<Global<JObject<'static>>>(&raw_activity)? };
            let activity = env.new_global_ref(borrowed_activity)?;
            Ok(AndroidContext {
                app,
                vm: vm.clone(),
                activity,
                helper: Mutex::new(None),
                soft_input_visible: AtomicBool::new(false),
                system_bar_scheme: AtomicU8::new(0),
            })
        })
        .map_err(|error| error.to_string())?;

    *lock_context() = Some(Arc::new(created));
    configure_edge_to_edge()
}

/// Configures the retained NativeActivity for transparent edge-to-edge system bars.
fn configure_edge_to_edge() -> Result<(), String> {
    with_context(|env, context| {
        let window = env
            .call_method(
                context.activity.as_ref(),
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        env.call_method(
            &window,
            jni_str!("setStatusBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(0)],
        )?;
        env.call_method(
            &window,
            jni_str!("setNavigationBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(0)],
        )?;
        if android_sdk_version(env)? >= 30 {
            env.call_method(
                &window,
                jni_str!("setDecorFitsSystemWindows"),
                jni_sig!("(Z)V"),
                &[JValue::Bool(false)],
            )?;
        } else {
            let decor = env
                .call_method(
                    &window,
                    jni_str!("getDecorView"),
                    jni_sig!("()Landroid/view/View;"),
                    &[],
                )?
                .l()?;
            let current = env
                .call_method(
                    &decor,
                    jni_str!("getSystemUiVisibility"),
                    jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            let layout = android_view_flag(env, "SYSTEM_UI_FLAG_LAYOUT_STABLE")?
                | android_view_flag(env, "SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN")?
                | android_view_flag(env, "SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION")?;
            env.call_method(
                &decor,
                jni_str!("setSystemUiVisibility"),
                jni_sig!("(I)V"),
                &[JValue::Int(current | layout)],
            )?;
        }
        Ok(())
    })
}

/// Updates Android's transparent system-bar icon contrast when the resolved theme changes.
pub(super) fn set_system_bar_color_scheme(scheme: ColorScheme) {
    let context = {
        let guard = lock_context();
        guard.as_ref().cloned()
    };
    let Some(context) = context else {
        return;
    };
    let encoded = match scheme {
        ColorScheme::Light => 1,
        ColorScheme::Dark => 2,
    };
    if context.system_bar_scheme.swap(encoded, Ordering::AcqRel) == encoded {
        return;
    }
    let _ = with_context(|env, context| {
        let window = env
            .call_method(
                context.activity.as_ref(),
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;
        let light_background = scheme == ColorScheme::Light;
        let sdk_version = android_sdk_version(env)?;
        if sdk_version >= 30 {
            let controller = env
                .call_method(
                    &window,
                    jni_str!("getInsetsController"),
                    jni_sig!("()Landroid/view/WindowInsetsController;"),
                    &[],
                )?
                .l()?;
            if controller.is_null() {
                return Ok(());
            }
            let mask = env
                .get_static_field(
                    jni_str!("android/view/WindowInsetsController"),
                    jni_str!("APPEARANCE_LIGHT_STATUS_BARS"),
                    jni_sig!("I"),
                )?
                .i()?
                | env
                    .get_static_field(
                        jni_str!("android/view/WindowInsetsController"),
                        jni_str!("APPEARANCE_LIGHT_NAVIGATION_BARS"),
                        jni_sig!("I"),
                    )?
                    .i()?;
            env.call_method(
                &controller,
                jni_str!("setSystemBarsAppearance"),
                jni_sig!("(II)V"),
                &[
                    JValue::Int(if light_background { mask } else { 0 }),
                    JValue::Int(mask),
                ],
            )?;
            return Ok(());
        }
        let decor = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let current = env
            .call_method(
                &decor,
                jni_str!("getSystemUiVisibility"),
                jni_sig!("()I"),
                &[],
            )?
            .i()?;
        let mut mask = 0;
        if sdk_version >= 23 {
            mask |= android_view_flag(env, "SYSTEM_UI_FLAG_LIGHT_STATUS_BAR")?;
        }
        if sdk_version >= 26 {
            mask |= android_view_flag(env, "SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR")?;
        }
        env.call_method(
            &decor,
            jni_str!("setSystemUiVisibility"),
            jni_sig!("(I)V"),
            &[JValue::Int(if light_background {
                current | mask
            } else {
                current & !mask
            })],
        )?;
        Ok(())
    });
}

/// Reads the device API level used to select compatible window APIs.
fn android_sdk_version(env: &mut Env<'_>) -> jni::errors::Result<i32> {
    env.get_static_field(
        jni_str!("android/os/Build$VERSION"),
        jni_str!("SDK_INT"),
        jni_sig!("I"),
    )?
    .i()
}

/// Reads one public `android.view.View` system-UI flag by field name.
fn android_view_flag(env: &mut Env<'_>, name: &'static str) -> jni::errors::Result<i32> {
    let field = match name {
        "SYSTEM_UI_FLAG_LAYOUT_STABLE" => jni_str!("SYSTEM_UI_FLAG_LAYOUT_STABLE"),
        "SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN" => jni_str!("SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN"),
        "SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION" => {
            jni_str!("SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION")
        }
        "SYSTEM_UI_FLAG_LIGHT_STATUS_BAR" => jni_str!("SYSTEM_UI_FLAG_LIGHT_STATUS_BAR"),
        _ => jni_str!("SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR"),
    };
    env.get_static_field(jni_str!("android/view/View"), field, jni_sig!("I"))?
        .i()
}

/// Runs one JNI operation with an attached thread and retained app references.
fn with_context<T>(
    operation: impl FnOnce(&mut Env<'_>, &AndroidContext) -> jni::errors::Result<T>,
) -> Result<T, String> {
    let context = lock_context()
        .as_ref()
        .cloned()
        .ok_or_else(|| "Android mobile context is not initialized".to_owned())?;
    context
        .vm
        .attach_current_thread_for_scope(|env| operation(env, &context))
        .map_err(|error| error.to_string())
}

/// Runs one mobile-activity JNI operation with its lazily loaded Java helper class.
fn with_helper<T>(
    env: &mut Env<'_>,
    context: &AndroidContext,
    operation: impl FnOnce(&mut Env<'_>, &JClass<'_>) -> jni::errors::Result<T>,
) -> jni::errors::Result<T> {
    let mut helper = context
        .helper
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if helper.is_none() {
        let class_loader = env
            .call_method(
                context.activity.as_ref(),
                jni_str!("getClassLoader"),
                jni_sig!("()Ljava/lang/ClassLoader;"),
                &[],
            )?
            .l()?;
        let helper_name = env.new_string("dev.argui.android.MobileActivityHost")?;
        let helper_object = env
            .call_method(
                &class_loader,
                jni_str!("loadClass"),
                jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
                &[JValue::Object(&helper_name)],
            )?
            .l()?;
        *helper = Some(env.new_cast_global_ref::<JClass<'static>>(helper_object)?);
    }
    let helper = helper
        .as_ref()
        .ok_or(jni::errors::Error::NullPtr("MobileActivityHost class"))?;
    operation(env, helper)
}

/// Requests Android notification permission and begins the foreground service when allowed.
pub(super) fn start(title: &str, message: &str) -> Result<(), MobileActivityError> {
    let permission_granted = with_context(|env, context| {
        with_helper(env, context, |env, helper| {
            env.call_static_method(
                helper,
                jni_str!("prepareNotificationPermission"),
                jni_sig!("(Landroid/app/Activity;)Z"),
                &[JValue::Object(context.activity.as_ref())],
            )?
            .z()
        })
    })
    .map_err(MobileActivityError::Native)?;
    if !permission_granted {
        return Err(MobileActivityError::NotificationPermissionRequired);
    }

    with_context(|env, context| {
        let title = env.new_string(title)?;
        let message = env.new_string(message)?;
        with_helper(env, context, |env, helper| {
            env.call_static_method(
                helper,
                jni_str!("start"),
                jni_sig!("(Landroid/app/Activity;Ljava/lang/String;Ljava/lang/String;)V"),
                &[
                    JValue::Object(context.activity.as_ref()),
                    JValue::Object(title.as_ref()),
                    JValue::Object(message.as_ref()),
                ],
            )?;
            Ok(())
        })
    })
    .map_err(MobileActivityError::Native)
}

/// Updates the foreground notification using the supplied percentage and message.
pub(super) fn update(percent: u8, message: &str) -> Result<(), MobileActivityError> {
    with_context(|env, context| {
        let message = env.new_string(message)?;
        with_helper(env, context, |env, helper| {
            env.call_static_method(
                helper,
                jni_str!("update"),
                jni_sig!("(Landroid/app/Activity;ILjava/lang/String;)V"),
                &[
                    JValue::Object(context.activity.as_ref()),
                    JValue::Int(i32::from(percent)),
                    JValue::Object(message.as_ref()),
                ],
            )?;
            Ok(())
        })
    })
    .map_err(MobileActivityError::Native)
}

/// Stops the foreground service and removes its notification.
pub(super) fn finish() -> Result<(), MobileActivityError> {
    with_context(|env, context| {
        with_helper(env, context, |env, helper| {
            env.call_static_method(
                helper,
                jni_str!("finish"),
                jni_sig!("(Landroid/app/Activity;)V"),
                &[JValue::Object(context.activity.as_ref())],
            )?;
            Ok(())
        })
    })
    .map_err(MobileActivityError::Native)
}

/// Reads the current root-window insets from Android's decor view.
pub(super) fn safe_area_insets() -> Option<PhysicalInsets> {
    with_context(read_window_insets).ok().flatten()
}

/// Explicitly shows or hides Android's soft keyboard through the retained activity.
pub(super) fn set_soft_input_visible(visible: bool) {
    let context = {
        let guard = lock_context();
        guard.as_ref().cloned()
    };
    if let Some(context) = context {
        if context.soft_input_visible.swap(visible, Ordering::AcqRel) == visible {
            return;
        }
        if visible {
            context.app.show_soft_input(false);
        } else {
            context.app.hide_soft_input(true);
        }
    }
}

/// Reads system bars and display cutouts using the Android API available on this device.
fn read_window_insets(
    env: &mut Env<'_>,
    context: &AndroidContext,
) -> jni::errors::Result<Option<PhysicalInsets>> {
    let window = env
        .call_method(
            context.activity.as_ref(),
            jni_str!("getWindow"),
            jni_sig!("()Landroid/view/Window;"),
            &[],
        )?
        .l()?;
    let decor = env
        .call_method(
            &window,
            jni_str!("getDecorView"),
            jni_sig!("()Landroid/view/View;"),
            &[],
        )?
        .l()?;
    let root = env
        .call_method(
            &decor,
            jni_str!("getRootWindowInsets"),
            jni_sig!("()Landroid/view/WindowInsets;"),
            &[],
        )?
        .l()?;
    if root.is_null() {
        return Ok(None);
    }

    let sdk_version = env
        .get_static_field(
            jni_str!("android/os/Build$VERSION"),
            jni_str!("SDK_INT"),
            jni_sig!("I"),
        )?
        .i()?;
    let insets = if sdk_version >= 30 {
        read_modern_insets(env, &root)?
    } else {
        read_pre_android_11_insets(env, &root, sdk_version)?
    };
    Ok(Some(insets))
}

/// Reads Android 11+ system-bar and cutout values through the typed inset API.
fn read_modern_insets(
    env: &mut Env<'_>,
    root: &JObject<'_>,
) -> jni::errors::Result<PhysicalInsets> {
    let inset_types = env.find_class(jni_str!("android/view/WindowInsets$Type"))?;
    let system_bars = env
        .call_static_method(&inset_types, jni_str!("systemBars"), jni_sig!("()I"), &[])?
        .i()?;
    let display_cutout = env
        .call_static_method(
            &inset_types,
            jni_str!("displayCutout"),
            jni_sig!("()I"),
            &[],
        )?
        .i()?;
    let values = env
        .call_method(
            root,
            jni_str!("getInsets"),
            jni_sig!("(I)Landroid/graphics/Insets;"),
            &[JValue::Int(system_bars | display_cutout)],
        )?
        .l()?;
    read_insets_fields(env, &values)
}

/// Reads Android 10-and-earlier system-window values and includes available display cutouts.
fn read_pre_android_11_insets(
    env: &mut Env<'_>,
    root: &JObject<'_>,
    sdk_version: i32,
) -> jni::errors::Result<PhysicalInsets> {
    let mut result = PhysicalInsets {
        top: non_negative(read_int_method(env, root, "getSystemWindowInsetTop")?),
        right: non_negative(read_int_method(env, root, "getSystemWindowInsetRight")?),
        bottom: non_negative(read_int_method(env, root, "getSystemWindowInsetBottom")?),
        left: non_negative(read_int_method(env, root, "getSystemWindowInsetLeft")?),
    };
    if sdk_version >= 28 {
        let cutout = env
            .call_method(
                root,
                jni_str!("getDisplayCutout"),
                jni_sig!("()Landroid/view/DisplayCutout;"),
                &[],
            )?
            .l()?;
        if !cutout.is_null() {
            result.top = result.top.max(non_negative(read_int_method(
                env,
                &cutout,
                "getSafeInsetTop",
            )?));
            result.right = result.right.max(non_negative(read_int_method(
                env,
                &cutout,
                "getSafeInsetRight",
            )?));
            result.bottom = result.bottom.max(non_negative(read_int_method(
                env,
                &cutout,
                "getSafeInsetBottom",
            )?));
            result.left = result.left.max(non_negative(read_int_method(
                env,
                &cutout,
                "getSafeInsetLeft",
            )?));
        }
    }
    Ok(result)
}

/// Reads one Android 10-and-earlier inset method from a Java object.
fn read_int_method(
    env: &mut Env<'_>,
    object: &JObject<'_>,
    name: &'static str,
) -> jni::errors::Result<i32> {
    let method = match name {
        "getSystemWindowInsetTop" => jni_str!("getSystemWindowInsetTop"),
        "getSystemWindowInsetRight" => jni_str!("getSystemWindowInsetRight"),
        "getSystemWindowInsetBottom" => jni_str!("getSystemWindowInsetBottom"),
        "getSystemWindowInsetLeft" => jni_str!("getSystemWindowInsetLeft"),
        "getSafeInsetTop" => jni_str!("getSafeInsetTop"),
        "getSafeInsetRight" => jni_str!("getSafeInsetRight"),
        "getSafeInsetBottom" => jni_str!("getSafeInsetBottom"),
        _ => jni_str!("getSafeInsetLeft"),
    };
    env.call_method(object, method, jni_sig!("()I"), &[])?.i()
}

/// Reads the four public fields returned by `WindowInsets.getInsets`.
fn read_insets_fields(
    env: &mut Env<'_>,
    values: &JObject<'_>,
) -> jni::errors::Result<PhysicalInsets> {
    Ok(PhysicalInsets {
        top: non_negative(env.get_field(values, jni_str!("top"), jni_sig!("I"))?.i()?),
        right: non_negative(
            env.get_field(values, jni_str!("right"), jni_sig!("I"))?
                .i()?,
        ),
        bottom: non_negative(
            env.get_field(values, jni_str!("bottom"), jni_sig!("I"))?
                .i()?,
        ),
        left: non_negative(
            env.get_field(values, jni_str!("left"), jni_sig!("I"))?
                .i()?,
        ),
    })
}

/// Converts a signed Android inset to a non-negative physical-pixel distance.
const fn non_negative(value: i32) -> u32 {
    if value > 0 { value as u32 } else { 0 }
}
