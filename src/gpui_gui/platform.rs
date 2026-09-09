//! Host-loop and GPUI platform plumbing for the embedded editor.

#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use anyhow::Result;
use futures::channel::oneshot;
#[cfg(target_os = "windows")]
use gpui::MouseButton;
use gpui::{
    AnyWindowHandle, AtlasKey, AtlasTextureId, AtlasTextureKind, AtlasTile, BackgroundExecutor,
    Bounds, Capslock, ClipboardItem, CursorStyle, DevicePixels, DispatchEventResult,
    DummyKeyboardMapper, ForegroundExecutor, GpuSpecs, KeyDownEvent, KeyUpEvent, Keymap, Keystroke,
    Modifiers, PathPromptOptions, Pixels, Platform, PlatformAtlas, PlatformDisplay, PlatformInput,
    PlatformInputHandler, PlatformKeyboardLayout, PlatformKeyboardMapper, PlatformTextSystem,
    PlatformWindow, Point, Priority, PromptButton, RequestFrameOptions, Scene, Size, Task,
    ThermalState, TileId, WindowAppearance, WindowBackgroundAppearance, WindowBounds,
    WindowControlArea, WindowParams, point, px, size,
};
use gpui_wgpu::CosmicTextSystem;
use raw_window_handle_06::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};

pub(crate) type VisibilityCallback = Rc<RefCell<Option<Box<dyn FnMut(bool)>>>>;
pub(crate) type PointerCancelCallback = Rc<RefCell<Option<Box<dyn FnMut()>>>>;

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[path = "native.rs"]
mod native;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use native::NativeChild;

pub(crate) fn parent_scale_factor(parent: raw_window_handle::RawWindowHandle) -> f32 {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        native::parent_scale_factor(parent).max(0.01)
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = parent;
        1.0
    }
}

pub(crate) fn logical_to_host_size(width: u32, height: u32, scale: f32) -> (u32, u32) {
    let scale = scale.max(0.01);
    (
        ((width.max(1) as f32 * scale).round() as u32).max(1),
        ((height.max(1) as f32 * scale).round() as u32).max(1),
    )
}

pub(crate) fn host_to_logical_size(width: u32, height: u32, scale: f32) -> (u32, u32) {
    let scale = scale.max(0.01);
    (
        ((width.max(1) as f32 / scale).round() as u32).max(1),
        ((height.max(1) as f32 / scale).round() as u32).max(1),
    )
}

/// A foreground/background dispatcher driven by the host's UI thread.
///
/// GPUI never starts an application-owned event loop here. Main-thread tasks
/// are queued until [`EmbeddedPlatform::pump`] is called by the native child
/// view or by the plugin host's UI callback.
pub(crate) struct EmbeddedDispatcher {
    state: Arc<DispatcherState>,
}

struct DispatcherState {
    main_thread: ThreadId,
    stopped: std::sync::atomic::AtomicBool,
    failed: std::sync::atomic::AtomicBool,
    main_queue: Mutex<VecDeque<gpui::RunnableVariant>>,
    background_queue: Mutex<VecDeque<WorkItem>>,
    timers: Mutex<VecDeque<(Instant, gpui::RunnableVariant)>>,
    worker_wakeup: Condvar,
    worker: Mutex<Option<thread::JoinHandle<()>>>,
}

enum WorkItem {
    Runnable(gpui::RunnableVariant),
    Realtime(Box<dyn FnOnce() + Send>),
}

impl EmbeddedDispatcher {
    pub(crate) fn new() -> Arc<Self> {
        let state = Arc::new(DispatcherState {
            main_thread: thread::current().id(),
            stopped: std::sync::atomic::AtomicBool::new(false),
            failed: std::sync::atomic::AtomicBool::new(false),
            main_queue: Mutex::new(VecDeque::new()),
            background_queue: Mutex::new(VecDeque::new()),
            timers: Mutex::new(VecDeque::new()),
            worker_wakeup: Condvar::new(),
            worker: Mutex::new(None),
        });
        let weak_state = Arc::downgrade(&state);
        let worker = thread::Builder::new()
            .name("toybox-gpui-worker".to_string())
            .spawn(move || {
                loop {
                    let Some(state) = weak_state.upgrade() else {
                        break;
                    };
                    let runnable = {
                        let mut queue = match state.background_queue.lock() {
                            Ok(queue) => queue,
                            Err(_) => {
                                state
                                    .failed
                                    .store(true, std::sync::atomic::Ordering::Release);
                                state
                                    .stopped
                                    .store(true, std::sync::atomic::Ordering::Release);
                                state.worker_wakeup.notify_all();
                                return;
                            }
                        };
                        loop {
                            if state.stopped.load(std::sync::atomic::Ordering::Acquire) {
                                return;
                            }
                            if let Some(runnable) = queue.pop_front() {
                                break runnable;
                            }
                            queue = match state.worker_wakeup.wait(queue) {
                                Ok(queue) => queue,
                                Err(_) => {
                                    state
                                        .failed
                                        .store(true, std::sync::atomic::Ordering::Release);
                                    state
                                        .stopped
                                        .store(true, std::sync::atomic::Ordering::Release);
                                    state.worker_wakeup.notify_all();
                                    return;
                                }
                            };
                        }
                    };
                    if state.stopped.load(std::sync::atomic::Ordering::Acquire) {
                        drop(runnable);
                        continue;
                    }
                    let result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match runnable {
                            WorkItem::Runnable(runnable) => {
                                let _ = runnable.run();
                            }
                            WorkItem::Realtime(callback) => {
                                callback();
                            }
                        }));
                    if result.is_err() {
                        state
                            .failed
                            .store(true, std::sync::atomic::Ordering::Release);
                        state
                            .stopped
                            .store(true, std::sync::atomic::Ordering::Release);
                        state.worker_wakeup.notify_all();
                        return;
                    }
                }
            })
            .expect("failed to start GPUI dispatcher worker");
        *state.worker.lock().expect("GPUI worker lock poisoned") = Some(worker);
        Arc::new(Self { state })
    }

    pub(crate) fn pump(&self) {
        if thread::current().id() != self.state.main_thread
            || self
                .state
                .stopped
                .load(std::sync::atomic::Ordering::Acquire)
        {
            return;
        }
        let now = Instant::now();
        let mut ready = Vec::new();
        match self.state.timers.lock() {
            Ok(mut timers) => {
                let mut pending = VecDeque::new();
                while let Some((deadline, runnable)) = timers.pop_front() {
                    if deadline <= now {
                        ready.push(runnable);
                    } else {
                        pending.push_back((deadline, runnable));
                    }
                }
                *timers = pending;
            }
            Err(_) => {
                self.fail_runtime();
                return;
            }
        }
        if !ready.is_empty() {
            let mut accepted = false;
            match self.state.main_queue.lock() {
                Ok(mut queue)
                    if !self
                        .state
                        .stopped
                        .load(std::sync::atomic::Ordering::Acquire)
                        && queue.len().saturating_add(ready.len()) <= 1024 =>
                {
                    queue.extend(ready.drain(..));
                    accepted = true;
                }
                Ok(_) => {}
                Err(_) => {
                    self.fail_runtime();
                    return;
                }
            }
            if !accepted {
                self.fail_overflow();
            }
        }
        for _ in 0..64 {
            let runnable = match self.state.main_queue.lock() {
                Ok(mut queue) => queue.pop_front(),
                Err(_) => {
                    self.fail_runtime();
                    return;
                }
            };
            let Some(runnable) = runnable else {
                break;
            };
            if self
                .state
                .stopped
                .load(std::sync::atomic::Ordering::Acquire)
            {
                drop(runnable);
                continue;
            }
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| runnable.run())).is_err() {
                self.fail_runtime();
                return;
            }
        }
    }

    pub(crate) fn stop(&self) {
        self.state
            .stopped
            .store(true, std::sync::atomic::Ordering::Release);
        self.state.worker_wakeup.notify_all();
        let worker = self
            .state
            .worker
            .lock()
            .ok()
            .and_then(|mut worker| worker.take());
        if let Some(worker) = worker {
            if worker.thread().id() != thread::current().id() {
                let _ = worker.join();
            } else if let Ok(mut slot) = self.state.worker.lock() {
                // A realtime callback may request shutdown from the worker.
                // Keep the handle owned until a host-thread stop can join it;
                // dropping it here would detach code from an unloading DLL.
                *slot = Some(worker);
            }
        }

        // Take queued captures out of their locks before dropping them. A
        // GPUI task destructor can schedule more work, and dropping it while
        // holding one of these locks would deadlock that reentrant enqueue.
        let main_queue = match self.state.main_queue.lock() {
            Ok(mut queue) => Some(std::mem::take(&mut *queue)),
            Err(_) => {
                self.fail_runtime();
                None
            }
        };
        let background_queue = match self.state.background_queue.lock() {
            Ok(mut queue) => Some(std::mem::take(&mut *queue)),
            Err(_) => {
                self.fail_runtime();
                None
            }
        };
        let timers = match self.state.timers.lock() {
            Ok(mut timers) => Some(std::mem::take(&mut *timers)),
            Err(_) => {
                self.fail_runtime();
                None
            }
        };
        drop(main_queue);
        drop(background_queue);
        drop(timers);
    }

    pub(crate) fn failed(&self) -> bool {
        self.state.failed.load(std::sync::atomic::Ordering::Acquire)
    }

    fn fail_overflow(&self) {
        self.state
            .failed
            .store(true, std::sync::atomic::Ordering::Release);
        self.state
            .stopped
            .store(true, std::sync::atomic::Ordering::Release);
        self.state.worker_wakeup.notify_all();
    }

    fn fail_runtime(&self) {
        self.state
            .failed
            .store(true, std::sync::atomic::Ordering::Release);
        self.state
            .stopped
            .store(true, std::sync::atomic::Ordering::Release);
        self.state.worker_wakeup.notify_all();
    }

    fn enqueue_background(&self, item: WorkItem) {
        let mut item = Some(item);
        let mut overflow = false;
        let mut poisoned = false;
        match self.state.background_queue.lock() {
            Ok(mut queue)
                if !self
                    .state
                    .stopped
                    .load(std::sync::atomic::Ordering::Acquire) =>
            {
                if queue.len() < 1024 {
                    queue.push_back(item.take().expect("work item present"));
                } else {
                    overflow = true;
                }
            }
            Ok(_) => {}
            Err(_) => poisoned = true,
        }
        let accepted = item.is_none();
        drop(item);
        if poisoned {
            self.fail_runtime();
        } else if overflow {
            self.fail_overflow();
        } else if accepted {
            self.state.worker_wakeup.notify_one();
        }
    }
}

impl Drop for EmbeddedDispatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

impl gpui::PlatformDispatcher for EmbeddedDispatcher {
    fn is_main_thread(&self) -> bool {
        thread::current().id() == self.state.main_thread
    }

    fn dispatch(&self, runnable: gpui::RunnableVariant, _priority: Priority) {
        self.enqueue_background(WorkItem::Runnable(runnable));
    }

    fn dispatch_on_main_thread(&self, runnable: gpui::RunnableVariant, _priority: Priority) {
        let mut runnable = Some(runnable);
        let mut overflow = false;
        let mut poisoned = false;
        match self.state.main_queue.lock() {
            Ok(mut queue)
                if !self
                    .state
                    .stopped
                    .load(std::sync::atomic::Ordering::Acquire) =>
            {
                if queue.len() < 1024 {
                    queue.push_back(runnable.take().expect("runnable present"));
                } else {
                    overflow = true;
                }
            }
            Ok(_) => {}
            Err(_) => poisoned = true,
        }
        drop(runnable);
        if poisoned {
            self.fail_runtime();
        } else if overflow {
            self.fail_overflow();
        }
    }

    fn dispatch_after(&self, duration: Duration, runnable: gpui::RunnableVariant) {
        let mut runnable = Some(runnable);
        let mut overflow = false;
        let mut poisoned = false;
        match self.state.timers.lock() {
            Ok(mut timers)
                if !self
                    .state
                    .stopped
                    .load(std::sync::atomic::Ordering::Acquire) =>
            {
                if timers.len() < 1024 {
                    timers.push_back((
                        Instant::now() + duration,
                        runnable.take().expect("runnable present"),
                    ));
                } else {
                    overflow = true;
                }
            }
            Ok(_) => {}
            Err(_) => poisoned = true,
        }
        drop(runnable);
        if poisoned {
            self.fail_runtime();
        } else if overflow {
            self.fail_overflow();
        }
    }

    fn spawn_realtime(&self, f: Box<dyn FnOnce() + Send>) {
        self.enqueue_background(WorkItem::Realtime(f));
    }
}

#[derive(Debug)]
struct EmbeddedDisplay;

impl PlatformDisplay for EmbeddedDisplay {
    fn id(&self) -> gpui::DisplayId {
        gpui::DisplayId::new(1)
    }

    fn uuid(&self) -> Result<uuid::Uuid> {
        Err(anyhow::anyhow!(
            "embedded GPUI has no host display identity"
        ))
    }

    fn bounds(&self) -> Bounds<Pixels> {
        Bounds::new(point(px(0.0), px(0.0)), size(px(1.0), px(1.0)))
    }
}

/// Platform state owned by one hosted editor instance.
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(crate) struct EmbeddedPlatform {
    dispatcher: Arc<EmbeddedDispatcher>,
    background_executor: BackgroundExecutor,
    foreground_executor: ForegroundExecutor,
    text_system: Arc<dyn PlatformTextSystem>,
    display: Rc<dyn PlatformDisplay>,
    active_window: Cell<Option<AnyWindowHandle>>,
    active_window_shared: Rc<Cell<Option<AnyWindowHandle>>>,
    windows: RefCell<HashMap<gpui::WindowId, AnyWindowHandle>>,
    stopped: Cell<bool>,
    parent: raw_window_handle::RawWindowHandle,
    class_name: &'static str,
    callback_keyboard_only: bool,
    visibility_callback: VisibilityCallback,
    pointer_cancel_callback: PointerCancelCallback,
    gpu_context: gpui_wgpu::GpuContext,
    window_owner: RefCell<Option<Weak<WindowState>>>,
    parent_scale_factor: f32,
}

impl EmbeddedPlatform {
    pub(crate) fn new(
        parent: raw_window_handle::RawWindowHandle,
        class_name: &'static str,
        callback_keyboard_only: bool,
        visibility_callback: VisibilityCallback,
        pointer_cancel_callback: PointerCancelCallback,
    ) -> anyhow::Result<Rc<Self>> {
        let dispatcher = EmbeddedDispatcher::new();
        let dispatcher_trait: Arc<dyn gpui::PlatformDispatcher> = dispatcher.clone();
        let text_system = CosmicTextSystem::new_without_system_fonts("Ioskeley Mono");
        text_system.add_fonts(vec![
            Cow::Borrowed(include_bytes!(
                "../../assets/IoskeleyMono/IoskeleyMono-Regular.ttf"
            )),
            Cow::Borrowed(include_bytes!(
                "../../assets/Sometype_Mono/static/SometypeMono-Regular.ttf"
            )),
        ])?;
        Ok(Rc::new(Self {
            background_executor: BackgroundExecutor::new(dispatcher_trait.clone()),
            foreground_executor: ForegroundExecutor::new(dispatcher_trait),
            dispatcher,
            text_system: Arc::new(text_system),
            display: Rc::new(EmbeddedDisplay),
            active_window: Cell::new(None),
            active_window_shared: Rc::new(Cell::new(None)),
            windows: RefCell::new(HashMap::new()),
            stopped: Cell::new(false),
            parent,
            class_name,
            callback_keyboard_only,
            visibility_callback,
            pointer_cancel_callback,
            gpu_context: Rc::new(RefCell::new(None)),
            window_owner: RefCell::new(None),
            parent_scale_factor: parent_scale_factor(parent),
        }))
    }

    pub(crate) fn pump(&self) {
        self.dispatcher.pump();
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(owner) = self.window_owner.borrow().as_ref().and_then(Weak::upgrade) {
            owner.pump_gateway();
        }
    }

    pub(crate) fn stop(&self) {
        if !self.stopped.replace(true) {
            self.window_owner.borrow_mut().take();
            self.dispatcher.stop();
        }
    }

    pub(crate) fn failed(&self) -> bool {
        self.dispatcher.failed()
    }

    pub(crate) fn sync_visibility(&self) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let owner = { self.window_owner.borrow().as_ref().and_then(Weak::upgrade) };
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(owner) = owner {
            owner.sync_visibility();
        }
    }

    pub(crate) fn resize(&self, size: Size<Pixels>) {
        if let Some(owner) = self.window_owner.borrow().as_ref().and_then(Weak::upgrade) {
            owner.resize(size);
        }
    }

    pub(crate) fn show(&self, visible: bool) {
        if let Some(_owner) = self.window_owner.borrow().as_ref().and_then(Weak::upgrade) {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            if let Some(native) = _owner.native.borrow_mut().as_mut() {
                native.set_visible(visible);
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let _ = visible;
    }

    pub(crate) fn focus(&self, _focused: bool) -> bool {
        if let Some(_owner) = self.window_owner.borrow().as_ref().and_then(Weak::upgrade) {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            if _owner.native.borrow().is_some() {
                let result = _owner
                    .native
                    .borrow_mut()
                    .as_mut()
                    .is_some_and(|native| native.set_focus(_focused));
                if result {
                    if _focused {
                        _owner.focus_changed();
                    } else {
                        _owner.native_focus_lost();
                    }
                }
                return result;
            }
        }
        false
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        self.window_owner
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .map_or(self.parent_scale_factor, |owner| owner.scale_factor())
    }

    pub(crate) fn host_size_from_logical(&self, width: u32, height: u32) -> (u32, u32) {
        logical_to_host_size(width, height, self.host_unit_scale_factor())
    }

    pub(crate) fn logical_size_from_host(&self, width: u32, height: u32) -> (u32, u32) {
        host_to_logical_size(width, height, self.host_unit_scale_factor())
    }

    fn host_unit_scale_factor(&self) -> f32 {
        #[cfg(target_os = "windows")]
        {
            self.scale_factor()
        }
        #[cfg(not(target_os = "windows"))]
        {
            1.0
        }
    }

    pub(crate) fn dispatch_vst3_key(
        &self,
        key: u16,
        key_code: i16,
        modifiers: i16,
        down: bool,
    ) -> bool {
        self.window_owner
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
            .is_some_and(|owner| owner.dispatch_vst3_key(key, key_code, modifiers, down))
    }

    pub(crate) fn capture_rgba(&self) -> anyhow::Result<(u32, u32, Vec<u8>)> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let owner = self
                .window_owner
                .borrow()
                .as_ref()
                .and_then(Weak::upgrade)
                .ok_or_else(|| anyhow::anyhow!("GPUI native window is not open"))?;
            if owner.closed.get() {
                anyhow::bail!("GPUI native window is closed");
            }
            if let Some(native) = owner.native.borrow_mut().as_mut() {
                native.request_capture();
            }
            // GPUI owns the scene; request its next frame rather than trying
            // to clone or reconstruct a scene in the host facade.
            owner.request_frame();
            for _ in 0..8 {
                self.pump();
                if let Some(pixels) = owner
                    .native
                    .borrow_mut()
                    .as_mut()
                    .and_then(NativeChild::take_capture)
                {
                    return Ok(pixels);
                }
            }
            if owner
                .native
                .borrow()
                .as_ref()
                .is_some_and(NativeChild::is_failed)
            {
                anyhow::bail!("GPUI native renderer failed during capture");
            }
            anyhow::bail!("GPUI frame did not produce a capture")
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            anyhow::bail!("GPUI capture requires a native embedded renderer")
        }
    }
}

impl Platform for EmbeddedPlatform {
    fn background_executor(&self) -> BackgroundExecutor {
        self.background_executor.clone()
    }

    fn foreground_executor(&self) -> ForegroundExecutor {
        self.foreground_executor.clone()
    }

    fn text_system(&self) -> Arc<dyn PlatformTextSystem> {
        self.text_system.clone()
    }

    fn run(&self, on_finish_launching: Box<dyn 'static + FnOnce()>) {
        on_finish_launching();
    }

    fn quit(&self) {
        self.stop();
    }

    fn restart(&self, _binary_path: Option<PathBuf>) {}
    fn activate(&self, _ignoring_other_apps: bool) {}
    fn hide(&self) {}
    fn hide_other_apps(&self) {}
    fn unhide_other_apps(&self) {}

    fn displays(&self) -> Vec<Rc<dyn PlatformDisplay>> {
        vec![self.display.clone()]
    }

    fn primary_display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(self.display.clone())
    }

    fn active_window(&self) -> Option<AnyWindowHandle> {
        self.active_window_shared.get()
    }

    fn open_window(
        &self,
        handle: AnyWindowHandle,
        options: WindowParams,
    ) -> anyhow::Result<Box<dyn PlatformWindow>> {
        self.windows.borrow_mut().insert(handle.window_id(), handle);
        self.active_window.set(Some(handle));
        self.active_window_shared.set(Some(handle));
        let window = Box::new(EmbeddedWindow::new(
            handle,
            options,
            self.display.clone(),
            self.active_window_shared.clone(),
            self.dispatcher.clone(),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            self.visibility_callback.clone(),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            self.pointer_cancel_callback.clone(),
        ));
        let _state = window.state.clone();
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            let native = NativeChild::new(
                self.parent,
                self.class_name,
                Rc::downgrade(&_state),
                self.gpu_context.clone(),
                window.bounds().size,
                self.callback_keyboard_only,
            )?;
            _state.atlas.borrow_mut().clone_from(&native.sprite_atlas());
            _state.native.replace(Some(native));
            _state.visibility_state.set(Some(true));
            *self.window_owner.borrow_mut() = Some(Rc::downgrade(&_state));
        }
        Ok(window)
    }

    fn window_appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }

    fn open_url(&self, _url: &str) {}
    fn on_open_urls(&self, _callback: Box<dyn FnMut(Vec<String>)>) {}
    fn register_url_scheme(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }

    fn prompt_for_paths(
        &self,
        _options: PathPromptOptions,
    ) -> oneshot::Receiver<Result<Option<Vec<PathBuf>>>> {
        let (_sender, receiver) = oneshot::channel();
        receiver
    }

    fn prompt_for_new_path(
        &self,
        _directory: &Path,
        _suggested_name: Option<&str>,
    ) -> oneshot::Receiver<Result<Option<PathBuf>>> {
        let (_sender, receiver) = oneshot::channel();
        receiver
    }

    fn can_select_mixed_files_and_dirs(&self) -> bool {
        false
    }

    fn reveal_path(&self, _path: &Path) {}
    fn open_with_system(&self, _path: &Path) {}
    fn on_quit(&self, _callback: Box<dyn FnMut()>) {}
    fn on_reopen(&self, _callback: Box<dyn FnMut()>) {}
    fn on_system_wake(&self, _callback: Box<dyn FnMut()>) {}
    fn set_menus(&self, _menus: Vec<gpui::Menu>, _keymap: &Keymap) {}
    fn set_dock_menu(&self, _menu: Vec<gpui::MenuItem>, _keymap: &Keymap) {}
    fn on_app_menu_action(&self, _callback: Box<dyn FnMut(&dyn gpui::Action)>) {}
    fn on_will_open_app_menu(&self, _callback: Box<dyn FnMut()>) {}
    fn on_validate_app_menu_command(&self, _callback: Box<dyn FnMut(&dyn gpui::Action) -> bool>) {}
    fn thermal_state(&self) -> ThermalState {
        ThermalState::Nominal
    }
    fn on_thermal_state_change(&self, _callback: Box<dyn FnMut()>) {}
    fn app_path(&self) -> Result<PathBuf> {
        std::env::current_exe().map_err(Into::into)
    }
    fn path_for_auxiliary_executable(&self, name: &str) -> Result<PathBuf> {
        Ok(self.app_path()?.with_file_name(name))
    }
    fn set_cursor_style(&self, _style: CursorStyle) {}
    fn hide_cursor_until_mouse_moves(&self) {}
    fn is_cursor_visible(&self) -> bool {
        true
    }
    fn should_auto_hide_scrollbars(&self) -> bool {
        false
    }
    fn read_from_clipboard(&self) -> Option<ClipboardItem> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        {
            native::read_clipboard()
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            None
        }
    }
    fn write_to_clipboard(&self, item: ClipboardItem) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        native::write_clipboard(item);
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let _ = item;
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn read_from_primary(&self) -> Option<ClipboardItem> {
        None
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn write_to_primary(&self, _item: ClipboardItem) {}
    #[cfg(target_os = "macos")]
    fn read_from_find_pasteboard(&self) -> Option<ClipboardItem> {
        None
    }
    #[cfg(target_os = "macos")]
    fn write_to_find_pasteboard(&self, _item: ClipboardItem) {}
    fn write_credentials(&self, _url: &str, _username: &str, _password: &[u8]) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }
    fn read_credentials(&self, _url: &str) -> Task<Result<Option<(String, Vec<u8>)>>> {
        Task::ready(Ok(None))
    }
    fn delete_credentials(&self, _url: &str) -> Task<Result<()>> {
        Task::ready(Ok(()))
    }
    fn keyboard_layout(&self) -> Box<dyn PlatformKeyboardLayout> {
        Box::new(EmbeddedKeyboardLayout)
    }
    fn keyboard_mapper(&self) -> Rc<dyn PlatformKeyboardMapper> {
        Rc::new(DummyKeyboardMapper)
    }
    fn on_keyboard_layout_change(&self, _callback: Box<dyn FnMut()>) {}
}

struct EmbeddedKeyboardLayout;

impl PlatformKeyboardLayout for EmbeddedKeyboardLayout {
    fn id(&self) -> &str {
        "embedded"
    }

    fn name(&self) -> &str {
        "Embedded"
    }
}

struct InputHandlerSlot {
    value: Option<PlatformInputHandler>,
    generation: u64,
}

struct CallbackSlot<T> {
    value: Option<T>,
    generation: u64,
}

type InputCallback = Box<dyn FnMut(PlatformInput) -> DispatchEventResult>;
type FrameCallback = Box<dyn FnMut(RequestFrameOptions)>;
type BoolCallback = Box<dyn FnMut(bool)>;
type ResizeCallback = Box<dyn FnMut(Size<Pixels>, f32)>;
type HitTestCallback = Box<dyn FnMut() -> Option<WindowControlArea>>;

impl<T> Default for CallbackSlot<T> {
    fn default() -> Self {
        Self {
            value: None,
            generation: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
enum KeySource {
    Native,
    Vst3Callback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
pub(crate) enum NativeEventIdentity {
    #[cfg_attr(target_os = "windows", allow(dead_code))]
    Mac {
        timestamp_bits: u64,
        key_code: u16,
        event_type: u16,
        window: u64,
    },
    #[cfg(target_os = "windows")]
    Windows {
        message_time: u32,
        window: u64,
        virtual_key: u32,
        scan_code: u16,
        kind: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NativeEventToken {
    generation: u64,
    identity: NativeEventIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SemanticKey {
    key: String,
    modifiers: Modifiers,
    down: bool,
}

struct LedgerKey {
    stroke: SemanticKey,
    source: KeySource,
    result: DispatchEventResult,
}

struct LedgerText {
    text: String,
    source: KeySource,
    handled: bool,
}

struct EventLedgerEntry {
    token: NativeEventToken,
    key: Option<LedgerKey>,
    text: Option<LedgerText>,
}

#[derive(Default)]
struct EventLedger {
    entries: VecDeque<EventLedgerEntry>,
}

impl EventLedger {
    const MAX_ENTRIES: usize = 128;

    #[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
    fn clear(&mut self) {
        self.entries.clear();
    }

    fn key_duplicate(
        &self,
        token: NativeEventToken,
        stroke: &SemanticKey,
        source: KeySource,
    ) -> Option<DispatchEventResult> {
        self.entries.iter().find_map(|entry| {
            if entry.token != token {
                return None;
            }
            let key = entry.key.as_ref()?;
            (key.source != source && key.stroke == *stroke).then(|| key.result.clone())
        })
    }

    fn record_key(
        &mut self,
        token: NativeEventToken,
        stroke: SemanticKey,
        source: KeySource,
        result: DispatchEventResult,
    ) {
        if self.entries.iter().any(|entry| {
            entry.token == token && entry.key.as_ref().is_some_and(|key| key.stroke == stroke)
        }) {
            return;
        }
        self.push(EventLedgerEntry {
            token,
            key: Some(LedgerKey {
                stroke,
                source,
                result,
            }),
            text: None,
        });
    }

    fn text_duplicate(
        &self,
        token: NativeEventToken,
        text: &str,
        source: KeySource,
    ) -> Option<bool> {
        self.entries.iter().find_map(|entry| {
            if entry.token != token {
                return None;
            }
            let text_entry = entry.text.as_ref()?;
            (text_entry.source != source && text_entry.text == text).then_some(text_entry.handled)
        })
    }

    fn record_text(
        &mut self,
        token: NativeEventToken,
        text: String,
        source: KeySource,
        handled: bool,
    ) {
        if self.entries.iter().any(|entry| {
            entry.token == token
                && entry
                    .text
                    .as_ref()
                    .is_some_and(|text_entry| text_entry.text == text)
        }) {
            return;
        }
        self.push(EventLedgerEntry {
            token,
            key: None,
            text: Some(LedgerText {
                text,
                source,
                handled,
            }),
        });
    }

    fn push(&mut self, entry: EventLedgerEntry) {
        if self.entries.len() >= Self::MAX_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }
}

#[cfg(target_os = "windows")]
#[derive(Default)]
struct WindowsTextState {
    pending_high_surrogate: Option<(u16, Option<NativeEventToken>)>,
    suppressed_commit_units: VecDeque<u16>,
    composition_active: bool,
}

#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
struct WindowState {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    dispatcher: Arc<EmbeddedDispatcher>,
    bounds: Cell<Bounds<Pixels>>,
    active_window: Rc<Cell<Option<AnyWindowHandle>>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    native: RefCell<Option<NativeChild>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    visibility_callback: VisibilityCallback,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pointer_cancel_callback: PointerCancelCallback,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    visibility_state: Cell<Option<bool>>,
    input_handler: RefCell<InputHandlerSlot>,
    input_callback: RefCell<CallbackSlot<InputCallback>>,
    request_frame_callback: RefCell<CallbackSlot<FrameCallback>>,
    active_callback: RefCell<Option<BoolCallback>>,
    hover_callback: RefCell<Option<BoolCallback>>,
    resize_callback: RefCell<CallbackSlot<ResizeCallback>>,
    moved_callback: RefCell<Option<Box<dyn FnMut()>>>,
    close_callback: RefCell<Option<Box<dyn FnOnce()>>>,
    should_close_callback: RefCell<Option<Box<dyn FnMut() -> bool>>>,
    hit_test_callback: RefCell<Option<HitTestCallback>>,
    appearance_callback: RefCell<Option<Box<dyn FnMut()>>>,
    atlas: RefCell<Arc<dyn PlatformAtlas>>,
    title: RefCell<String>,
    background: Cell<WindowBackgroundAppearance>,
    fullscreen: Cell<bool>,
    closed: Cell<bool>,
    in_update: Cell<bool>,
    pending_inputs: RefCell<VecDeque<PlatformInput>>,
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pending_pointer_cancel: Cell<bool>,
    #[cfg(target_os = "windows")]
    pending_active_status: Cell<Option<bool>>,
    pending_frame: Cell<bool>,
    pending_resize: Cell<Option<Size<Pixels>>>,
    suppress_resize_callback: Cell<bool>,
    focus_generation: Cell<u64>,
    event_ledger: RefCell<EventLedger>,
    active_native_token: Cell<Option<NativeEventToken>>,
    last_native_key_token: Cell<Option<NativeEventToken>>,
    #[cfg(target_os = "windows")]
    windows_text: RefCell<WindowsTextState>,
    #[cfg(target_os = "windows")]
    native_pointer_button: Cell<Option<MouseButton>>,
}

impl Drop for WindowState {
    fn drop(&mut self) {
        self.closed.set(true);
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(mut native) = self.native.get_mut().take() {
            native.clear_owner();
            native.close();
        }
    }
}

struct EmbeddedWindow {
    state: Rc<WindowState>,
    handle: AnyWindowHandle,
    display: Rc<dyn PlatformDisplay>,
}

impl Drop for EmbeddedWindow {
    fn drop(&mut self) {
        self.state.closed.set(true);
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(mut native) = self.state.native.borrow_mut().take() {
            native.clear_owner();
            native.close();
        }
    }
}

impl EmbeddedWindow {
    fn new(
        handle: AnyWindowHandle,
        options: WindowParams,
        display: Rc<dyn PlatformDisplay>,
        active_window: Rc<Cell<Option<AnyWindowHandle>>>,
        _dispatcher: Arc<EmbeddedDispatcher>,
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        visibility_callback: VisibilityCallback,
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        pointer_cancel_callback: PointerCancelCallback,
    ) -> Self {
        Self {
            state: Rc::new(WindowState {
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                dispatcher: _dispatcher,
                bounds: Cell::new(options.bounds),
                active_window,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                native: RefCell::new(None),
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                visibility_callback,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                pointer_cancel_callback,
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                visibility_state: Cell::new(None),
                input_handler: RefCell::new(InputHandlerSlot {
                    value: None,
                    generation: 0,
                }),
                input_callback: RefCell::new(CallbackSlot::default()),
                request_frame_callback: RefCell::new(CallbackSlot::default()),
                active_callback: RefCell::new(None),
                hover_callback: RefCell::new(None),
                resize_callback: RefCell::new(CallbackSlot::default()),
                moved_callback: RefCell::new(None),
                close_callback: RefCell::new(None),
                should_close_callback: RefCell::new(None),
                hit_test_callback: RefCell::new(None),
                appearance_callback: RefCell::new(None),
                atlas: RefCell::new(Arc::new(EmptyAtlas::default())),
                title: RefCell::new(String::new()),
                background: Cell::new(WindowBackgroundAppearance::Opaque),
                fullscreen: Cell::new(false),
                closed: Cell::new(false),
                in_update: Cell::new(false),
                pending_inputs: RefCell::new(VecDeque::new()),
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                pending_pointer_cancel: Cell::new(false),
                #[cfg(target_os = "windows")]
                pending_active_status: Cell::new(None),
                pending_frame: Cell::new(false),
                pending_resize: Cell::new(None),
                suppress_resize_callback: Cell::new(false),
                focus_generation: Cell::new(0),
                event_ledger: RefCell::new(EventLedger::default()),
                active_native_token: Cell::new(None),
                last_native_key_token: Cell::new(None),
                #[cfg(target_os = "windows")]
                windows_text: RefCell::new(WindowsTextState::default()),
                #[cfg(target_os = "windows")]
                native_pointer_button: Cell::new(None),
            }),
            handle,
            display,
        }
    }
}

#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
impl WindowState {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(crate) fn native_tick(&self) {
        if self.closed.get() {
            return;
        }
        self.dispatcher.pump();
        self.sync_visibility();
        self.pump_gateway();
        if self.visibility_state.get() == Some(true) {
            self.request_frame();
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn sync_visibility(&self) {
        let visible = self.native.borrow().as_ref().map(NativeChild::is_visible);
        if self.visibility_state.get() == visible {
            return;
        }
        self.visibility_state.set(visible);
        let Some(visible) = visible else {
            return;
        };
        if let Ok(mut callback) = self.visibility_callback.try_borrow_mut()
            && let Some(callback) = callback.as_mut()
        {
            let _ = crate::gui_panic::contain("GPUI visibility observer", || callback(visible));
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn dispatch_pointer_cancel(&self) {
        let Ok(mut callback_slot) = self.pointer_cancel_callback.try_borrow_mut() else {
            self.pending_pointer_cancel.set(true);
            return;
        };
        let Some(mut callback) = callback_slot.take() else {
            return;
        };
        drop(callback_slot);
        let _ = crate::gui_panic::contain("GPUI pointer cancellation callback", &mut callback);
        if !self.closed.get()
            && self
                .pointer_cancel_callback
                .try_borrow()
                .is_ok_and(|callback| callback.is_none())
        {
            self.pointer_cancel_callback.borrow_mut().replace(callback);
        }
    }

    pub(crate) fn resize(&self, size: Size<Pixels>) {
        if self.closed.get() {
            return;
        }
        self.bounds.set(Bounds::new(self.bounds.get().origin, size));
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(native) = self.native.borrow_mut().as_mut() {
            self.suppress_resize_callback.set(true);
            native.resize(size);
            self.suppress_resize_callback.set(false);
        }
        // Native resize callbacks are suppressed while the child is borrowed.
        // Queue the layout notification as this can also be called from inside
        // GPUI's App update, where synchronously invoking its callback reenters
        // the App borrow. The next gateway drain delivers the latest size.
        self.pending_resize.set(Some(size));
    }

    fn with_input_handler<R>(
        &self,
        callback: impl FnOnce(&mut PlatformInputHandler) -> R,
    ) -> Option<R> {
        if self.closed.get() {
            return None;
        }
        let (generation, mut input_handler) = {
            let mut slot = self.input_handler.borrow_mut();
            (slot.generation, slot.value.take()?)
        };
        let result = callback(&mut input_handler);
        let mut slot = self.input_handler.borrow_mut();
        if !self.closed.get() && slot.generation == generation && slot.value.is_none() {
            slot.value = Some(input_handler);
        }
        Some(result)
    }

    pub(crate) fn dispatch_input(&self, event: PlatformInput) -> DispatchEventResult {
        if self.closed.get() {
            return DispatchEventResult {
                propagate: false,
                default_prevented: true,
            };
        }
        if self.in_update.replace(true) {
            let mut pending = self.pending_inputs.borrow_mut();
            if pending.len() >= 1024 {
                self.closed.set(true);
            } else {
                pending.push_back(event);
            }
            return DispatchEventResult {
                propagate: false,
                default_prevented: true,
            };
        }
        let result = self.dispatch_input_one(event);
        self.drain_gateway();
        result
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(crate) fn dispatch_native_key(
        &self,
        event: PlatformInput,
        token: Option<NativeEventToken>,
    ) -> DispatchEventResult {
        let previous = self.active_native_token.replace(token);
        match &event {
            PlatformInput::KeyDown(_) => self.last_native_key_token.set(token),
            PlatformInput::KeyUp(_) => {
                self.last_native_key_token.set(None);
            }
            _ => {}
        }
        let result = self.dispatch_key_from_source(event, KeySource::Native, token);
        self.active_native_token.set(previous);
        result
    }

    pub(crate) fn native_event_token(&self, identity: NativeEventIdentity) -> NativeEventToken {
        NativeEventToken {
            generation: self.focus_generation.get(),
            identity,
        }
    }

    fn current_native_event_token(&self) -> Option<NativeEventToken> {
        if let Some(token) = self.active_native_token.get() {
            return Some(token);
        }
        #[cfg(target_os = "macos")]
        {
            native::current_event_identity().map(|identity| self.native_event_token(identity))
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }

    pub(crate) fn dispatch_vst3_key(
        &self,
        key: u16,
        key_code: i16,
        modifiers: i16,
        down: bool,
    ) -> bool {
        if self.closed.get() || self.input_callback.borrow().value.is_none() {
            return false;
        }
        let Some((key_name, key_char)) = vst3_keystroke(key, key_code) else {
            return false;
        };
        let modifiers = vst3_modifiers(modifiers);
        let keystroke = Keystroke {
            modifiers,
            key: key_name,
            key_char,
        };
        let event = if down {
            PlatformInput::KeyDown(KeyDownEvent {
                keystroke,
                is_held: false,
                prefer_character_input: false,
            })
        } else {
            PlatformInput::KeyUp(KeyUpEvent { keystroke })
        };
        let token = self.current_native_event_token();
        let result = self.dispatch_key_from_source(event, KeySource::Vst3Callback, token);
        if !down {
            return !result.propagate || result.default_prevented;
        }
        if !result.propagate || result.default_prevented {
            return true;
        }
        if !modifiers.control
            && !modifiers.alt
            && !modifiers.platform
            && let Some(text) = vst3_text(key, key_code)
        {
            return self.dispatch_callback_text(&text, token);
        }
        false
    }

    fn dispatch_key_from_source(
        &self,
        event: PlatformInput,
        source: KeySource,
        token: Option<NativeEventToken>,
    ) -> DispatchEventResult {
        let (key, modifiers, down) = match &event {
            PlatformInput::KeyDown(event) => {
                (event.keystroke.key.clone(), event.keystroke.modifiers, true)
            }
            PlatformInput::KeyUp(event) => (
                event.keystroke.key.clone(),
                event.keystroke.modifiers,
                false,
            ),
            _ => return self.dispatch_input(event),
        };
        let stroke = SemanticKey {
            key,
            modifiers,
            down,
        };
        if let Some(token) = token
            && token.generation == self.focus_generation.get()
            && let Some(result) = self
                .event_ledger
                .borrow()
                .key_duplicate(token, &stroke, source)
        {
            return result;
        }
        let result = self.dispatch_input(event);
        if let Some(token) = token
            && token.generation == self.focus_generation.get()
        {
            self.event_ledger
                .borrow_mut()
                .record_key(token, stroke, source, result.clone());
        }
        result
    }

    fn dispatch_input_one(&self, event: PlatformInput) -> DispatchEventResult {
        let (generation, Some(mut callback)) = ({
            let mut slot = self.input_callback.borrow_mut();
            (slot.generation, slot.value.take())
        }) else {
            return DispatchEventResult::default();
        };
        let result = crate::gui_panic::contain("GPUI input callback", || callback(event))
            .unwrap_or(DispatchEventResult {
                propagate: true,
                default_prevented: true,
            });
        let mut slot = self.input_callback.borrow_mut();
        if !self.closed.get() && slot.generation == generation && slot.value.is_none() {
            slot.value = Some(callback);
        }
        result
    }

    pub(crate) fn dispatch_native_text(&self, text: &str, token: Option<NativeEventToken>) -> bool {
        self.dispatch_text_from_source(text, KeySource::Native, token)
    }

    fn dispatch_callback_text(&self, text: &str, token: Option<NativeEventToken>) -> bool {
        self.dispatch_text_from_source(text, KeySource::Vst3Callback, token)
    }

    fn dispatch_text_from_source(
        &self,
        text: &str,
        source: KeySource,
        token: Option<NativeEventToken>,
    ) -> bool {
        if let Some(token) = token
            && token.generation == self.focus_generation.get()
            && let Some(handled) = self
                .event_ledger
                .borrow()
                .text_duplicate(token, text, source)
        {
            return handled;
        }
        let handled = self
            .with_input_handler(|input_handler| {
                let _ = crate::gui_panic::contain("GPUI text input", || {
                    input_handler.replace_text_in_range(None, text);
                });
            })
            .is_some();
        if let Some(token) = token
            && token.generation == self.focus_generation.get()
        {
            self.event_ledger
                .borrow_mut()
                .record_text(token, text.to_string(), source, handled);
        }
        handled
    }

    pub(crate) fn request_frame(&self) {
        if self.closed.get() {
            return;
        }
        if self.in_update.replace(true) {
            self.pending_frame.set(true);
            return;
        }
        // Deliver any queued layout change before drawing the resized surface.
        self.pending_frame.set(true);
        self.drain_gateway();
    }

    fn request_frame_one(&self) {
        let (generation, callback) = {
            let mut slot = self.request_frame_callback.borrow_mut();
            (slot.generation, slot.value.take())
        };
        if let Some(mut callback) = callback {
            let _ = crate::gui_panic::contain("GPUI frame callback", || {
                callback(RequestFrameOptions {
                    require_presentation: true,
                    force_render: false,
                });
            });
            let mut slot = self.request_frame_callback.borrow_mut();
            if !self.closed.get() && slot.generation == generation && slot.value.is_none() {
                slot.value = Some(callback);
            }
        }
    }

    fn drain_gateway(&self) {
        let mut budget = 64;
        while !self.closed.get() && budget > 0 {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            if self.pending_pointer_cancel.replace(false) {
                self.dispatch_pointer_cancel();
                budget -= 1;
                continue;
            }
            #[cfg(target_os = "windows")]
            if let Some(active) = self.pending_active_status.take() {
                self.dispatch_active_status(active);
                budget -= 1;
                continue;
            }
            let event = { self.pending_inputs.borrow_mut().pop_front() };
            if let Some(event) = event {
                let _ = self.dispatch_input_one(event);
                budget -= 1;
                continue;
            }
            if let Some(size) = self.pending_resize.take() {
                self.resize_callback(size);
                budget -= 1;
                continue;
            }
            if self.pending_frame.replace(false) {
                self.request_frame_one();
                budget -= 1;
                continue;
            }
            break;
        }
        let has_pending = {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            let pointer_cancel = self.pending_pointer_cancel.get();
            #[cfg(not(any(target_os = "macos", target_os = "windows")))]
            let pointer_cancel = false;
            #[cfg(target_os = "windows")]
            let active_status = self.pending_active_status.get().is_some();
            #[cfg(not(target_os = "windows"))]
            let active_status = false;
            !self.pending_inputs.borrow().is_empty()
                || pointer_cancel
                || active_status
                || self.pending_resize.get().is_some()
                || self.pending_frame.get()
        };
        self.in_update.set(false);
        if has_pending && !self.closed.get() {
            // Ask GPUI for another frame after the bounded gateway drain. The
            // callback is invoked after `in_update` is released, so a nested
            // AsyncApp.update cannot borrow this gateway recursively.
            self.request_frame_one();
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    fn pump_gateway(&self) {
        if self.closed.get() || self.in_update.replace(true) {
            return;
        }
        self.drain_gateway();
    }

    fn resize_callback(&self, size: Size<Pixels>) {
        let (generation, callback) = {
            let mut slot = self.resize_callback.borrow_mut();
            (slot.generation, slot.value.take())
        };
        if let Some(mut callback) = callback {
            let scale = self.scale_factor();
            let _ = crate::gui_panic::contain("GPUI resize callback", || callback(size, scale));
            let mut slot = self.resize_callback.borrow_mut();
            if !self.closed.get() && slot.generation == generation && slot.value.is_none() {
                slot.value = Some(callback);
            }
        }
    }

    pub(crate) fn native_resize(&self, width: f32, height: f32) {
        if self.closed.get() {
            return;
        }
        let size = size(px(width.max(1.0)), px(height.max(1.0)));
        self.bounds.set(Bounds::new(self.bounds.get().origin, size));
        if self.suppress_resize_callback.get() {
            return;
        }
        if self.in_update.replace(true) {
            self.pending_resize.set(Some(size));
            return;
        }
        self.resize_callback(size);
        self.drain_gateway();
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn native_backing_scale_changed(&self, width: f32, height: f32) {
        if self.closed.get() {
            return;
        }
        let size = size(px(width.max(1.0)), px(height.max(1.0)));
        if let Some(native) = self.native.borrow_mut().as_mut() {
            native.backing_scale_changed(size);
        }
        self.native_resize(width, height);
    }

    /// Make the embedded child the native first responder after a real mouse
    /// click. AppKit and Win32 do not promote an arbitrary custom child view
    /// merely because it accepts keyboard focus.
    pub(crate) fn native_mouse_down_focus(&self) {
        if self.closed.get() {
            return;
        }
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let focused = self
            .native
            .borrow_mut()
            .as_mut()
            .is_some_and(|native| native.set_focus(true));
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let focused = false;
        if focused {
            self.invalidate_event_generation();
            self.dispatch_active_status(true);
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(crate) fn native_focus_lost(&self) {
        if self.closed.get() {
            return;
        }
        self.focus_changed();
        #[cfg(target_os = "windows")]
        self.native_pointer_button.set(None);
        #[cfg(target_os = "windows")]
        self.pending_active_status.set(Some(false));
        self.pending_pointer_cancel.set(true);
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn native_pointer_capture_lost(&self) {
        if self.closed.get() {
            return;
        }
        if self.native_pointer_button.take().is_some() {
            self.pending_pointer_cancel.set(true);
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn native_pointer_pressed(&self, button: MouseButton, captured: bool) -> bool {
        if self.closed.get() || !captured {
            return false;
        }
        if self.pending_pointer_cancel.get() && !self.in_update.get() {
            self.pump_gateway();
        }
        if self.pending_pointer_cancel.get() || self.native_pointer_button.get().is_some() {
            return false;
        }
        self.native_pointer_button.set(Some(button));
        true
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn native_pointer_released(&self, button: MouseButton) -> bool {
        if self.native_pointer_button.get() != Some(button) {
            return false;
        }
        self.native_pointer_button.set(None);
        true
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn native_pointer_button(&self) -> Option<MouseButton> {
        self.native_pointer_button.get()
    }

    fn dispatch_active_status(&self, active: bool) {
        #[cfg(target_os = "windows")]
        self.pending_active_status.set(None);
        let Some(mut callback) = self.active_callback.borrow_mut().take() else {
            return;
        };
        let _ = crate::gui_panic::contain("GPUI active status callback", || callback(active));
        if !self.closed.get() && self.active_callback.borrow().is_none() {
            self.active_callback.borrow_mut().replace(callback);
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn native_resize_device(&self, width: f32, height: f32) {
        if self.closed.get() || self.suppress_resize_callback.get() {
            return;
        }
        let scale = self.scale_factor().max(0.01);
        self.native_resize(width / scale, height / scale);
    }

    pub(crate) fn quarantine_native(&self) {
        self.closed.set(true);
        self.invalidate_event_generation();
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(mut native) = self.native.borrow_mut().take() {
            native.clear_owner();
            native.quarantine();
        }
    }

    pub(crate) fn focus_changed(&self) {
        if !self.closed.get() {
            self.invalidate_event_generation();
        }
    }

    fn invalidate_event_generation(&self) {
        self.focus_generation
            .set(self.focus_generation.get().wrapping_add(1));
        self.event_ledger.borrow_mut().clear();
        self.active_native_token.set(None);
        self.last_native_key_token.set(None);
        #[cfg(target_os = "windows")]
        {
            *self.windows_text.borrow_mut() = WindowsTextState::default();
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn windows_text_token(
        &self,
        window: usize,
        message_time: u32,
    ) -> Option<NativeEventToken> {
        let token = self.last_native_key_token.get()?;
        if token.generation != self.focus_generation.get() {
            return None;
        }
        match token.identity {
            NativeEventIdentity::Windows {
                message_time: token_time,
                window: token_window,
                ..
            } if token_time == message_time && token_window == window as u64 => Some(token),
            _ => None,
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn windows_char(
        &self,
        unit: u16,
        modifiers: Modifiers,
        token: Option<NativeEventToken>,
    ) {
        if windows_char_is_shortcut(unit, modifiers) {
            return;
        }

        let output = {
            let mut state = self.windows_text.borrow_mut();
            if state
                .suppressed_commit_units
                .front()
                .is_some_and(|suppressed| *suppressed == unit)
            {
                state.suppressed_commit_units.pop_front();
                return;
            }
            if !state.suppressed_commit_units.is_empty() {
                state.suppressed_commit_units.clear();
            }

            decode_windows_utf16_unit(&mut state, unit, token)
        };
        for (text, token) in output {
            self.dispatch_native_text(&text, token);
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn windows_ime_composition(
        &self,
        text: &str,
        result: bool,
        token: Option<NativeEventToken>,
    ) {
        if result {
            {
                let mut state = self.windows_text.borrow_mut();
                state.pending_high_surrogate = None;
                state.suppressed_commit_units = text.encode_utf16().collect();
                state.composition_active = false;
            }
            self.dispatch_native_text(text, token);
            self.unmark_text();
        } else {
            self.windows_text.borrow_mut().composition_active = true;
            self.dispatch_marked_text(None, text, None);
        }
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn windows_ime_end(&self) {
        self.windows_text.borrow_mut().composition_active = false;
        self.unmark_text();
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn native_dpi_changed(&self) {
        if self.closed.get() {
            return;
        }
        let size = self.bounds.get().size;
        self.suppress_resize_callback.set(true);
        if let Some(native) = self.native.borrow_mut().as_mut() {
            native.dpi_changed(size);
        }
        self.suppress_resize_callback.set(false);
        self.native_resize(f32::from(size.width), f32::from(size.height));
    }

    pub(crate) fn dispatch_marked_text(
        &self,
        replacement_range: Option<Range<usize>>,
        text: &str,
        selected_range: Option<Range<usize>>,
    ) {
        let _ = self.with_input_handler(|input_handler| {
            let _ = crate::gui_panic::contain("GPUI marked text input", || {
                input_handler.replace_and_mark_text_in_range(
                    replacement_range,
                    text,
                    selected_range,
                );
            });
        });
    }

    pub(crate) fn unmark_text(&self) {
        let _ = self.with_input_handler(|input_handler| {
            let _ = crate::gui_panic::contain("GPUI unmark text", || input_handler.unmark_text());
        });
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn marked_text_range(&self) -> Option<Range<usize>> {
        self.with_input_handler(|input_handler| input_handler.marked_text_range())
            .flatten()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn selected_text_range(&self) -> Option<Range<usize>> {
        self.with_input_handler(|input_handler| {
            input_handler
                .selected_text_range(true)
                .map(|selection| selection.range)
        })
        .flatten()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn bounds_for_range(&self, range: Range<usize>) -> Option<Bounds<Pixels>> {
        self.with_input_handler(|input_handler| input_handler.bounds_for_range(range))
            .flatten()
    }

    #[cfg(target_os = "macos")]
    pub(crate) fn character_index_for_point(&self, x: f32, y: f32) -> Option<usize> {
        self.with_input_handler(|input_handler| {
            input_handler.character_index_for_point(Point::new(px(x), px(y)))
        })
        .flatten()
    }

    fn scale_factor(&self) -> f32 {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(native) = self.native.borrow().as_ref() {
            return native.scale_factor();
        }
        1.0
    }
}

impl HasWindowHandle for EmbeddedWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(native) = self.state.native.borrow().as_ref() {
            return native.window_handle();
        }
        Err(HandleError::NotSupported)
    }
}

impl HasDisplayHandle for EmbeddedWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(native) = self.state.native.borrow().as_ref() {
            return native.display_handle();
        }
        Err(HandleError::NotSupported)
    }
}

#[cfg(target_os = "windows")]
fn windows_char_is_shortcut(unit: u16, modifiers: Modifiers) -> bool {
    // WM_CHAR emits C0 controls for Ctrl-A/C/V and for navigation keys. Those
    // strokes have already gone through WM_KEYDOWN and must not become text.
    // A printable AltGr character is delivered with Ctrl+Alt, so retain that
    // path even though Windows reports the control modifier as pressed.
    if unit < 0x20 || unit == 0x7f {
        return true;
    }
    modifiers.platform
        || (modifiers.control && !modifiers.alt)
        || (modifiers.alt && !modifiers.control)
}

#[cfg(target_os = "windows")]
fn decode_windows_utf16_unit(
    state: &mut WindowsTextState,
    unit: u16,
    token: Option<NativeEventToken>,
) -> Vec<(String, Option<NativeEventToken>)> {
    let mut output = Vec::new();
    if let Some((high, pending_token)) = state.pending_high_surrogate.take() {
        if (0xDC00..=0xDFFF).contains(&unit) {
            let code_point =
                0x1_0000 + (((u32::from(high) - 0xD800) << 10) | (u32::from(unit) - 0xDC00));
            if let Some(character) = char::from_u32(code_point) {
                output.push((character.to_string(), pending_token.or(token)));
            }
        } else {
            output.push((String::from_utf16_lossy(&[high]), pending_token));
            if (0xD800..=0xDBFF).contains(&unit) {
                state.pending_high_surrogate = Some((unit, token));
            } else if let Some(character) = char::from_u32(u32::from(unit)) {
                output.push((character.to_string(), token));
            } else {
                output.push((String::from_utf16_lossy(&[unit]), token));
            }
        }
    } else if (0xD800..=0xDBFF).contains(&unit) {
        state.pending_high_surrogate = Some((unit, token));
    } else if let Some(character) = char::from_u32(u32::from(unit)) {
        output.push((character.to_string(), token));
    } else {
        output.push((String::from_utf16_lossy(&[unit]), token));
    }
    output
}

impl PlatformWindow for EmbeddedWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        self.state.bounds.get()
    }

    fn is_maximized(&self) -> bool {
        false
    }

    fn window_bounds(&self) -> WindowBounds {
        WindowBounds::Windowed(self.bounds())
    }

    fn content_size(&self) -> Size<Pixels> {
        self.bounds().size
    }

    fn resize(&mut self, size: Size<Pixels>) {
        self.state.resize(size);
    }

    fn scale_factor(&self) -> f32 {
        self.state.scale_factor()
    }

    fn appearance(&self) -> WindowAppearance {
        WindowAppearance::Light
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        Some(self.display.clone())
    }

    fn mouse_position(&self) -> Point<Pixels> {
        Point::default()
    }

    fn modifiers(&self) -> Modifiers {
        Modifiers::default()
    }

    fn capslock(&self) -> Capslock {
        Capslock::default()
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        let mut slot = self.state.input_handler.borrow_mut();
        slot.generation = slot.generation.wrapping_add(1);
        slot.value = Some(input_handler);
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        let mut slot = self.state.input_handler.borrow_mut();
        slot.generation = slot.generation.wrapping_add(1);
        slot.value.take()
    }

    fn prompt(
        &self,
        _level: gpui::PromptLevel,
        _msg: &str,
        _detail: Option<&str>,
        _answers: &[PromptButton],
    ) -> Option<oneshot::Receiver<usize>> {
        None
    }

    fn activate(&self) {
        self.state.active_window.set(Some(self.handle));
    }

    fn is_active(&self) -> bool {
        self.state
            .active_window
            .get()
            .is_some_and(|handle| handle.window_id() == self.handle.window_id())
    }

    fn is_hovered(&self) -> bool {
        false
    }

    fn background_appearance(&self) -> WindowBackgroundAppearance {
        self.state.background.get()
    }

    fn set_title(&mut self, title: &str) {
        self.state.title.replace(title.to_string());
    }

    fn set_background_appearance(&self, background_appearance: WindowBackgroundAppearance) {
        self.state.background.set(background_appearance);
    }

    fn minimize(&self) {}
    fn zoom(&self) {}
    fn toggle_fullscreen(&self) {
        self.state.fullscreen.set(!self.state.fullscreen.get());
    }
    fn is_fullscreen(&self) -> bool {
        self.state.fullscreen.get()
    }

    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        let mut slot = self.state.request_frame_callback.borrow_mut();
        slot.generation = slot.generation.wrapping_add(1);
        slot.value = Some(callback);
    }

    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> DispatchEventResult>) {
        let mut slot = self.state.input_callback.borrow_mut();
        slot.generation = slot.generation.wrapping_add(1);
        slot.value = Some(callback);
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        *self.state.active_callback.borrow_mut() = Some(callback);
    }

    fn on_hover_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        *self.state.hover_callback.borrow_mut() = Some(callback);
    }

    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        let mut slot = self.state.resize_callback.borrow_mut();
        slot.generation = slot.generation.wrapping_add(1);
        slot.value = Some(callback);
    }

    fn on_moved(&self, callback: Box<dyn FnMut()>) {
        *self.state.moved_callback.borrow_mut() = Some(callback);
    }

    fn on_should_close(&self, callback: Box<dyn FnMut() -> bool>) {
        *self.state.should_close_callback.borrow_mut() = Some(callback);
    }

    fn on_hit_test_window_control(&self, callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
        *self.state.hit_test_callback.borrow_mut() = Some(callback);
    }

    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        *self.state.close_callback.borrow_mut() = Some(callback);
    }

    fn on_appearance_changed(&self, callback: Box<dyn FnMut()>) {
        *self.state.appearance_callback.borrow_mut() = Some(callback);
    }

    fn draw(&self, _scene: &Scene) {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let failed = {
            let mut native = self.state.native.borrow_mut();
            native.as_mut().is_some_and(|native| {
                crate::gui_panic::contain("GPUI native draw", || native.draw(_scene)).is_none()
            })
        };
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if failed {
            self.state.quarantine_native();
        }
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.state.atlas.borrow().clone()
    }

    fn is_subpixel_rendering_supported(&self) -> bool {
        false
    }

    fn gpu_specs(&self) -> Option<GpuSpecs> {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        if let Some(native) = self.state.native.borrow().as_ref() {
            return native.gpu_specs();
        }
        None
    }

    #[cfg(target_os = "windows")]
    fn get_raw_handle(&self) -> windows_061::Win32::Foundation::HWND {
        self.state
            .native
            .borrow()
            .as_ref()
            .map(|native| windows_061::Win32::Foundation::HWND(native.raw_handle()))
            .unwrap_or_default()
    }

    fn update_ime_position(&self, _bounds: Bounds<Pixels>) {}
}

/// Translate Steinberg's platform-independent virtual-key values into the
/// key names used by GPUI. These values are part of the VST3 ABI and are
/// intentionally kept here so the GPUI feature does not require linking the
/// VST3 SDK for CLAP-only consumers.
fn vst3_keystroke(key: u16, key_code: i16) -> Option<(String, Option<String>)> {
    let key_code = i64::from(key_code);
    let semantic = match key_code {
        1 => Some(("backspace", None)),
        2 => Some(("tab", None)),
        4 | 19 => Some(("enter", None)),
        6 => Some(("escape", None)),
        7 => Some(("space", Some(" ".to_string()))),
        9 => Some(("end", None)),
        10 => Some(("home", None)),
        11 => Some(("left", None)),
        12 => Some(("up", None)),
        13 => Some(("right", None)),
        14 => Some(("down", None)),
        15 => Some(("pageup", None)),
        16 => Some(("pagedown", None)),
        21 => Some(("insert", None)),
        22 => Some(("delete", None)),
        40..=51 => Some((f_key_name(key_code - 40), None)),
        65..=76 => Some((f_key_name(key_code - 65 + 12), None)),
        _ => None,
    };
    if let Some((name, text)) = semantic {
        return Some((name.to_string(), text));
    }
    if matches!(key, 8 | 127) {
        return Some(("backspace".to_string(), None));
    }
    let text = vst3_text(key, key_code as i16)?;
    Some((text.to_lowercase(), Some(text)))
}

fn f_key_name(value: i64) -> &'static str {
    match value {
        0 => "f1",
        1 => "f2",
        2 => "f3",
        3 => "f4",
        4 => "f5",
        5 => "f6",
        6 => "f7",
        7 => "f8",
        8 => "f9",
        9 => "f10",
        10 => "f11",
        11 => "f12",
        12 => "f13",
        13 => "f14",
        14 => "f15",
        15 => "f16",
        16 => "f17",
        17 => "f18",
        18 => "f19",
        19 => "f20",
        20 => "f21",
        21 => "f22",
        22 => "f23",
        _ => "f24",
    }
}

fn vst3_text(key: u16, key_code: i16) -> Option<String> {
    let key_code = i64::from(key_code);
    let code = match key_code {
        7 => return Some(" ".to_string()),
        1 | 2 | 4 | 6 | 9..=16 | 19 | 21 | 22 | 40..=51 | 65..=76 => return None,
        24..=33 if key == 0 => {
            return char::from_u32(b'0' as u32 + (key_code - 24) as u32)
                .map(|character| character.to_string());
        }
        34 if key == 0 => return Some("*".to_string()),
        35 if key == 0 => return Some("+".to_string()),
        36 if key == 0 => return Some(",".to_string()),
        37 if key == 0 => return Some("-".to_string()),
        38 if key == 0 => return Some(".".to_string()),
        39 if key == 0 => return Some("/".to_string()),
        _ if key != 0 => u32::from(key),
        _ if (0x21..=0x7e).contains(&key_code) => key_code as u32,
        _ => return None,
    };
    char::from_u32(code)
        .filter(|character| !character.is_control())
        .map(|character| character.to_string())
}

fn vst3_modifiers(modifiers: i16) -> Modifiers {
    let modifiers = i64::from(modifiers);
    #[cfg(target_os = "macos")]
    let (platform, control) = (4, 8);
    #[cfg(not(target_os = "macos"))]
    let (platform, control) = (8, 4);
    Modifiers {
        shift: modifiers & 1 != 0,
        alt: modifiers & 2 != 0,
        platform: modifiers & platform != 0,
        control: modifiers & control != 0,
        function: false,
    }
}

#[derive(Default)]
struct EmptyAtlas {
    tiles: Mutex<HashMap<AtlasKey, AtlasTile>>,
    next: std::sync::atomic::AtomicU32,
}

impl PlatformAtlas for EmptyAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> anyhow::Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> anyhow::Result<Option<AtlasTile>> {
        if let Some(tile) = self
            .tiles
            .lock()
            .ok()
            .and_then(|tiles| tiles.get(key).copied())
        {
            return Ok(Some(tile));
        }
        let Some((tile_size, _)) = build()? else {
            return Ok(None);
        };
        let index = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
        let tile = AtlasTile {
            texture_id: AtlasTextureId {
                index,
                kind: AtlasTextureKind::Monochrome,
            },
            tile_id: TileId(index),
            padding: 0,
            bounds: Bounds::new(Point::default(), tile_size),
        };
        if let Ok(mut tiles) = self.tiles.lock() {
            tiles.insert(key.clone(), tile);
        }
        Ok(Some(tile))
    }

    fn remove(&self, key: &AtlasKey) {
        if let Ok(mut tiles) = self.tiles.lock() {
            tiles.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn vst3_navigation_and_numeric_key_translation_is_stable() {
        assert_eq!(vst3_keystroke(0, 11), Some(("left".to_string(), None)));
        assert_eq!(
            vst3_keystroke(0, 24),
            Some(("0".to_string(), Some("0".to_string())))
        );
        assert_eq!(vst3_keystroke(0, 40), Some(("f1".to_string(), None)));
        assert_eq!(vst3_keystroke(0, 65), Some(("f13".to_string(), None)));
        assert_eq!(
            vst3_keystroke('A' as u16, 0),
            Some(("a".to_string(), Some("A".to_string()),))
        );
    }

    #[test]
    fn vst3_backspace_character_fallback_preserves_semantic_delete() {
        for character in [8, 127] {
            assert_eq!(
                vst3_keystroke(character, 0),
                Some(("backspace".to_string(), None))
            );
        }
        assert_eq!(vst3_keystroke(0, 1), Some(("backspace".to_string(), None)));
        assert_eq!(vst3_keystroke(0, 22), Some(("delete".to_string(), None)));
    }

    #[test]
    fn vst3_modifier_bits_follow_host_platform_convention() {
        let modifiers = vst3_modifiers(1 | 2 | 4 | 8);
        assert!(modifiers.shift);
        assert!(modifiers.alt);
        assert!(modifiers.platform);
        assert!(modifiers.control);
    }

    #[test]
    fn event_ledger_deduplicates_only_verified_cross_source_events() {
        let token_one = NativeEventToken {
            generation: 7,
            identity: NativeEventIdentity::Mac {
                timestamp_bits: 1,
                key_code: 12,
                event_type: 10,
                window: 3,
            },
        };
        let token_two = NativeEventToken {
            generation: 7,
            identity: NativeEventIdentity::Mac {
                timestamp_bits: 2,
                key_code: 12,
                event_type: 10,
                window: 3,
            },
        };
        let stroke = SemanticKey {
            key: "x".to_string(),
            modifiers: Modifiers::default(),
            down: true,
        };
        let result = DispatchEventResult {
            propagate: false,
            default_prevented: true,
        };
        let mut ledger = EventLedger::default();
        ledger.record_key(token_one, stroke.clone(), KeySource::Native, result.clone());
        assert_eq!(
            ledger.key_duplicate(token_one, &stroke, KeySource::Vst3Callback),
            Some(result.clone())
        );
        assert_eq!(
            ledger.key_duplicate(token_one, &stroke, KeySource::Native),
            None
        );
        assert_eq!(
            ledger.key_duplicate(token_two, &stroke, KeySource::Vst3Callback),
            None
        );

        ledger.record_text(token_one, "🙂".to_string(), KeySource::Native, true);
        assert_eq!(
            ledger.text_duplicate(token_one, "🙂", KeySource::Vst3Callback),
            Some(true)
        );
        assert_eq!(
            ledger.text_duplicate(token_two, "🙂", KeySource::Vst3Callback),
            None
        );

        // Entries are retained independently of elapsed wall-clock time and
        // survive interleaved native events, while a focus generation reset
        // removes every old token.
        ledger.record_key(token_two, stroke.clone(), KeySource::Native, result.clone());
        assert!(
            ledger
                .key_duplicate(token_two, &stroke, KeySource::Vst3Callback)
                .is_some()
        );
        ledger.clear();
        assert_eq!(
            ledger.key_duplicate(token_one, &stroke, KeySource::Vst3Callback),
            None
        );
    }

    #[test]
    fn uncorrelatable_callback_events_are_delivered_independently() {
        let state = test_window_state();
        let count = Rc::new(Cell::new(0_u32));
        let count_for_callback = count.clone();
        state.input_callback.borrow_mut().value = Some(Box::new(move |_event| {
            count_for_callback.set(count_for_callback.get() + 1);
            DispatchEventResult::default()
        }));
        let event = test_key_event();
        let _ = state.dispatch_key_from_source(event.clone(), KeySource::Native, None);
        let _ = state.dispatch_key_from_source(event, KeySource::Vst3Callback, None);
        assert_eq!(count.get(), 2);
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn native_keyup_clears_text_correlation_for_distinct_up_token() {
        let state = test_window_state();
        let down_token = NativeEventToken {
            generation: 0,
            identity: NativeEventIdentity::Mac {
                timestamp_bits: 11,
                key_code: 12,
                event_type: 10,
                window: 3,
            },
        };
        let up_token = NativeEventToken {
            generation: 0,
            identity: NativeEventIdentity::Mac {
                timestamp_bits: 12,
                key_code: 12,
                event_type: 11,
                window: 3,
            },
        };
        state.dispatch_native_key(test_key_event(), Some(down_token));
        assert_eq!(state.last_native_key_token.get(), Some(down_token));
        state.dispatch_native_key(
            PlatformInput::KeyUp(KeyUpEvent {
                keystroke: gpui::Keystroke {
                    modifiers: Modifiers::default(),
                    key: "x".to_string(),
                    key_char: None,
                },
            }),
            Some(up_token),
        );
        assert_eq!(state.last_native_key_token.get(), None);
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn native_pointer_cancel_is_deferred_through_the_gateway() {
        let state = test_window_state();
        let cancel_count = Rc::new(Cell::new(0_u32));
        let cancel_count_for_callback = cancel_count.clone();
        state
            .pointer_cancel_callback
            .borrow_mut()
            .replace(Box::new(move || {
                cancel_count_for_callback.set(cancel_count_for_callback.get() + 1);
            }));

        state.pending_pointer_cancel.set(true);
        assert_eq!(cancel_count.get(), 0);
        state.pump_gateway();
        assert_eq!(cancel_count.get(), 1);
        state.pump_gateway();
        assert_eq!(cancel_count.get(), 1);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_pointer_capture_lifecycle_matches_buttons_and_cancels_safely() {
        let state = test_window_state();
        let cancel_count = Rc::new(Cell::new(0_u32));
        let cancel_count_for_callback = cancel_count.clone();
        state
            .pointer_cancel_callback
            .borrow_mut()
            .replace(Box::new(move || {
                cancel_count_for_callback.set(cancel_count_for_callback.get() + 1);
            }));
        let active_statuses = Rc::new(RefCell::new(Vec::new()));
        let active_statuses_for_callback = active_statuses.clone();
        state
            .active_callback
            .borrow_mut()
            .replace(Box::new(move |active| {
                active_statuses_for_callback.borrow_mut().push(active);
            }));

        assert!(state.native_pointer_pressed(MouseButton::Left, true));
        assert_eq!(state.native_pointer_button(), Some(MouseButton::Left));
        assert!(!state.native_pointer_pressed(MouseButton::Right, true));
        assert!(!state.native_pointer_released(MouseButton::Right));
        assert_eq!(state.native_pointer_button(), Some(MouseButton::Left));
        assert!(state.native_pointer_released(MouseButton::Left));
        assert_eq!(state.native_pointer_button(), None);
        assert_eq!(cancel_count.get(), 0);

        assert!(state.native_pointer_pressed(MouseButton::Right, true));
        state.native_pointer_capture_lost();
        assert_eq!(state.native_pointer_button(), None);
        assert_eq!(cancel_count.get(), 0);
        assert!(state.native_pointer_pressed(MouseButton::Left, true));
        assert_eq!(cancel_count.get(), 1);
        assert_eq!(state.native_pointer_button(), Some(MouseButton::Left));

        state.native_focus_lost();
        assert_eq!(state.native_pointer_button(), None);
        state.pump_gateway();
        assert_eq!(cancel_count.get(), 2);
        assert_eq!(*active_statuses.borrow(), vec![false]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_utf16_surrogates_are_committed_as_one_scalar() {
        let mut state = WindowsTextState::default();
        let token = NativeEventToken {
            generation: 1,
            identity: NativeEventIdentity::Windows {
                message_time: 12,
                window: 4,
                virtual_key: 0,
                scan_code: 0,
                kind: 1,
            },
        };
        assert!(decode_windows_utf16_unit(&mut state, 0xD83D, Some(token)).is_empty());
        assert_eq!(
            decode_windows_utf16_unit(&mut state, 0xDE42, Some(token)),
            vec![("🙂".to_string(), Some(token))]
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_controls_are_filtered_but_altgr_printable_text_survives() {
        assert!(windows_char_is_shortcut(
            0x03,
            Modifiers {
                control: true,
                ..Modifiers::default()
            }
        ));
        assert!(windows_char_is_shortcut(
            'c' as u16,
            Modifiers {
                control: true,
                ..Modifiers::default()
            }
        ));
        assert!(windows_char_is_shortcut(
            'x' as u16,
            Modifiers {
                alt: true,
                ..Modifiers::default()
            }
        ));
        assert!(!windows_char_is_shortcut(
            '€' as u16,
            Modifiers {
                control: true,
                alt: true,
                ..Modifiers::default()
            }
        ));
    }

    #[test]
    fn realtime_worker_is_joined_when_dispatcher_stops() {
        let dispatcher = EmbeddedDispatcher::new();
        let (sender, receiver) = mpsc::channel();
        gpui::PlatformDispatcher::spawn_realtime(
            dispatcher.as_ref(),
            Box::new(move || sender.send(()).expect("test receiver is alive")),
        );
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("dispatcher worker ran realtime work");
        dispatcher.stop();
        assert!(!dispatcher.failed());
    }

    #[test]
    fn host_logical_size_conversion_tracks_dpi_scale() {
        assert_eq!(logical_to_host_size(208, 212, 1.0), (208, 212));
        assert_eq!(logical_to_host_size(208, 212, 1.25), (260, 265));
        assert_eq!(host_to_logical_size(260, 265, 1.25), (208, 212));
    }

    #[test]
    fn nested_input_is_queued_until_outer_callback_returns() {
        let state = test_window_state();
        let weak_state = Rc::downgrade(&state);
        let callback_count = Rc::new(Cell::new(0_u32));
        let callback_count_for_input = callback_count.clone();
        state.input_callback.borrow_mut().value = Some(Box::new(move |_event| {
            let count = callback_count_for_input.get();
            callback_count_for_input.set(count + 1);
            if count == 0
                && let Some(state) = weak_state.upgrade()
            {
                let _ = state.dispatch_input(test_key_event());
            }
            DispatchEventResult::default()
        }));

        let _ = state.dispatch_input(test_key_event());

        assert_eq!(callback_count.get(), 2);
        assert!(!state.closed.get());
    }

    #[test]
    fn callback_replacement_survives_reentrant_dispatch() {
        let state = test_window_state();
        let weak_state = Rc::downgrade(&state);
        let old_count = Rc::new(Cell::new(0_u32));
        let replacement_count = Rc::new(Cell::new(0_u32));
        let old_count_for_input = old_count.clone();
        let replacement_count_for_input = replacement_count.clone();
        state.input_callback.borrow_mut().value = Some(Box::new(move |_event| {
            old_count_for_input.set(old_count_for_input.get() + 1);
            let replacement_count = replacement_count_for_input.clone();
            if let Some(state) = weak_state.upgrade() {
                let mut slot = state.input_callback.borrow_mut();
                if slot.generation == 0 {
                    slot.generation = slot.generation.wrapping_add(1);
                    slot.value = Some(Box::new(move |_event| {
                        replacement_count.set(replacement_count.get() + 1);
                        DispatchEventResult::default()
                    }));
                }
            }
            DispatchEventResult::default()
        }));

        let _ = state.dispatch_input(test_key_event());
        let _ = state.dispatch_input(test_key_event());

        assert_eq!(old_count.get(), 1);
        assert_eq!(replacement_count.get(), 1);
    }

    #[test]
    fn programmatic_resize_defers_and_coalesces_layout_notification() {
        let state = test_window_state();
        let observed = Rc::new(RefCell::new(Vec::new()));
        let callback_observed = Rc::clone(&observed);
        state.resize_callback.borrow_mut().value = Some(Box::new(move |size, _| {
            callback_observed.borrow_mut().push(size);
        }));
        let frame_observed = Rc::clone(&observed);
        state.request_frame_callback.borrow_mut().value = Some(Box::new(move |_| {
            assert_eq!(
                *frame_observed.borrow(),
                vec![size(px(1280.0), px(800.0))],
                "layout must update before the first resized frame"
            );
        }));
        state.resize(size(px(800.0), px(500.0)));
        state.resize(size(px(1280.0), px(800.0)));
        assert!(
            observed.borrow().is_empty(),
            "must not reenter a GPUI update"
        );
        state.request_frame();
        assert_eq!(*observed.borrow(), vec![size(px(1280.0), px(800.0))]);
        state.request_frame();
        assert_eq!(observed.borrow().len(), 1);
        state.quarantine_native();
        state.resize(size(px(640.0), px(400.0)));
        assert!(state.pending_resize.get().is_none());
    }

    #[test]
    fn shutdown_during_input_or_frame_drops_callbacks_without_late_dispatch() {
        struct DropProbe(Rc<Cell<bool>>);

        impl Drop for DropProbe {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let state = test_window_state();
        let weak_state = Rc::downgrade(&state);
        let input_count = Rc::new(Cell::new(0_u32));
        let input_count_for_callback = input_count.clone();
        let input_dropped = Rc::new(Cell::new(false));
        let input_probe = DropProbe(input_dropped.clone());
        let frame_count = Rc::new(Cell::new(0_u32));
        let frame_count_for_callback = frame_count.clone();
        state.request_frame_callback.borrow_mut().value = Some(Box::new(move |_| {
            frame_count_for_callback.set(frame_count_for_callback.get() + 1);
        }));
        let resize_count = Rc::new(Cell::new(0_u32));
        let resize_count_for_callback = resize_count.clone();
        state.resize_callback.borrow_mut().value = Some(Box::new(move |_, _| {
            resize_count_for_callback.set(resize_count_for_callback.get() + 1);
        }));
        state.input_callback.borrow_mut().value = Some(Box::new(move |_event| {
            let _probe = &input_probe;
            input_count_for_callback.set(input_count_for_callback.get() + 1);
            let owner = weak_state.upgrade().expect("callback owner remains alive");
            owner.quarantine_native();
            owner.request_frame();
            owner.native_resize(20.0, 20.0);
            let nested = owner.dispatch_input(test_key_event());
            assert!(nested.default_prevented);
            DispatchEventResult::default()
        }));

        let _ = state.dispatch_input(test_key_event());

        assert_eq!(input_count.get(), 1);
        assert!(input_dropped.get(), "closed callback was not dropped");
        assert!(state.input_callback.borrow().value.is_none());
        assert_eq!(frame_count.get(), 0);
        assert_eq!(resize_count.get(), 0);
        assert!(state.pending_inputs.borrow().is_empty());
        assert!(state.pending_resize.get().is_none());
        assert!(!state.pending_frame.get());
        let _ = state.dispatch_input(test_key_event());
        state.request_frame();
        state.native_resize(30.0, 30.0);
        assert_eq!(input_count.get(), 1);
        assert_eq!(frame_count.get(), 0);
        assert_eq!(resize_count.get(), 0);

        let state = test_window_state();
        let weak_state = Rc::downgrade(&state);
        let frame_count = Rc::new(Cell::new(0_u32));
        let frame_count_for_callback = frame_count.clone();
        let frame_dropped = Rc::new(Cell::new(false));
        let frame_probe = DropProbe(frame_dropped.clone());
        state.request_frame_callback.borrow_mut().value = Some(Box::new(move |_| {
            let _probe = &frame_probe;
            frame_count_for_callback.set(frame_count_for_callback.get() + 1);
            weak_state
                .upgrade()
                .expect("frame owner remains alive")
                .quarantine_native();
        }));

        state.request_frame();
        state.request_frame();

        assert_eq!(frame_count.get(), 1);
        assert!(frame_dropped.get(), "closed frame callback was not dropped");
        assert!(state.request_frame_callback.borrow().value.is_none());
        assert!(state.closed.get());
    }

    fn test_key_event() -> PlatformInput {
        PlatformInput::KeyDown(KeyDownEvent {
            keystroke: Keystroke {
                modifiers: Modifiers::default(),
                key: "x".to_string(),
                key_char: Some("x".to_string()),
            },
            is_held: false,
            prefer_character_input: false,
        })
    }

    fn test_window_state() -> Rc<WindowState> {
        Rc::new(WindowState {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            dispatcher: EmbeddedDispatcher::new(),
            bounds: Cell::new(Bounds::new(
                point(px(0.0), px(0.0)),
                size(px(10.0), px(10.0)),
            )),
            active_window: Rc::new(Cell::new(None)),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            native: RefCell::new(None),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            visibility_callback: Rc::new(RefCell::new(None)),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            pointer_cancel_callback: Rc::new(RefCell::new(None)),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            visibility_state: Cell::new(None),
            input_handler: RefCell::new(InputHandlerSlot {
                value: None,
                generation: 0,
            }),
            input_callback: RefCell::new(CallbackSlot::default()),
            request_frame_callback: RefCell::new(CallbackSlot::default()),
            active_callback: RefCell::new(None),
            hover_callback: RefCell::new(None),
            resize_callback: RefCell::new(CallbackSlot::default()),
            moved_callback: RefCell::new(None),
            close_callback: RefCell::new(None),
            should_close_callback: RefCell::new(None),
            hit_test_callback: RefCell::new(None),
            appearance_callback: RefCell::new(None),
            atlas: RefCell::new(Arc::new(EmptyAtlas::default())),
            title: RefCell::new(String::new()),
            background: Cell::new(WindowBackgroundAppearance::Opaque),
            fullscreen: Cell::new(false),
            closed: Cell::new(false),
            in_update: Cell::new(false),
            pending_inputs: RefCell::new(VecDeque::new()),
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            pending_pointer_cancel: Cell::new(false),
            #[cfg(target_os = "windows")]
            pending_active_status: Cell::new(None),
            pending_frame: Cell::new(false),
            pending_resize: Cell::new(None),
            suppress_resize_callback: Cell::new(false),
            focus_generation: Cell::new(0),
            event_ledger: RefCell::new(EventLedger::default()),
            active_native_token: Cell::new(None),
            last_native_key_token: Cell::new(None),
            #[cfg(target_os = "windows")]
            windows_text: RefCell::new(WindowsTextState::default()),
            #[cfg(target_os = "windows")]
            native_pointer_button: Cell::new(None),
        })
    }
}
