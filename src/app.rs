//! The application runtime: a tokio-native event loop, state store, and
//! rendering driver.
//!
//! This is the entry point for most users. [`App`] owns the backend and
//! renderer; you supply a closure that builds a widget tree each frame and
//! optionally reacts to events.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::Mutex;

use crate::backend::{self, Backend};
use crate::error::Result;
use crate::event::Event;
use crate::renderer::Renderer;
use crate::widgets::base::{Widget, WidgetTree};

/// A type-erased slot in the state store.
type AnyState = Box<dyn std::any::Any + Send + Sync>;

/// Per-frame context handed to the user's render callback.
pub struct Context {
    /// The latest input event, if any. `None` means this frame is a
    /// redraw (e.g. after a resize or a wakeup).
    pub event: Option<Event>,
    /// The terminal size in (width, height) at the start of this frame.
    pub size: (u16, u16),
    /// Time elapsed since the app started.
    pub elapsed: Duration,
    /// Whether the app should exit after this frame. Set via [`Context::exit`].
    pub should_exit: bool,
    /// Pending wakeup deadline, if any.
    wakeup: Option<Instant>,
    state: StateStore,
}

impl Context {
    /// Request that the app exit after this frame.
    pub fn exit(&mut self) {
        self.should_exit = true;
    }

    /// Request a wakeup after `dur`. Use this to drive animations or polling
    /// when no input events are expected. The next frame will receive an
    /// [`Event::Wakeup`] (in `ctx.event`).
    pub fn request_wakeup(&mut self, dur: Duration) {
        self.wakeup = Some(Instant::now() + dur);
    }

    /// Store a piece of state by key. Replaces any existing value.
    pub fn set_state<T: std::any::Any + Send + Sync>(&mut self, key: impl Into<String>, value: T) {
        self.state.insert(key.into(), Box::new(value));
    }

    /// Borrow a piece of state by key, if it exists and matches the type.
    pub fn state<T: std::any::Any + Send + Sync + Clone>(&self, key: &str) -> Option<T> {
        self.state.get::<T>(key)
    }

    /// Borrow a piece of state by key, defaulting to `default` if missing.
    pub fn state_or<T: std::any::Any + Send + Sync + Clone>(&self, key: &str, default: T) -> T {
        self.state.get::<T>(key).unwrap_or(default)
    }
}

#[derive(Default)]
struct StateStore {
    map: std::collections::HashMap<String, AnyState>,
}

impl StateStore {
    fn insert(&mut self, key: String, value: AnyState) {
        self.map.insert(key, value);
    }
    fn get<T: std::any::Any + Send + Sync + Clone>(&self, key: &str) -> Option<T> {
        self.map
            .get(key)
            .and_then(|v| v.downcast_ref::<T>().cloned())
    }
}

/// A builder for [`App`].
pub struct AppBuilder {
    backend: Option<Box<dyn Backend>>,
    frame_budget: Duration,
    poll_timeout: Duration,
}

impl AppBuilder {
    /// Construct a new builder with defaults.
    pub fn new() -> Self {
        Self {
            backend: None,
            frame_budget: Duration::from_millis(16),
            poll_timeout: Duration::from_millis(250),
        }
    }

    /// Supply a custom backend. If not called, [`App::default`] uses
    /// [`backend::default_backend`].
    #[must_use]
    pub fn backend(mut self, b: Box<dyn Backend>) -> Self {
        self.backend = Some(b);
        self
    }

    /// Set the target frame interval (default: ~60fps).
    #[must_use]
    pub fn frame_budget(mut self, d: Duration) -> Self {
        self.frame_budget = d;
        self
    }

    /// Set the input poll timeout (default: 250ms).
    #[must_use]
    pub fn poll_timeout(mut self, d: Duration) -> Self {
        self.poll_timeout = d;
        self
    }

    /// Build the app.
    pub fn build(self) -> App {
        let backend = self
            .backend
            .unwrap_or_else(|| Box::new(backend::default_backend()) as Box<dyn Backend>);
        App {
            backend,
            frame_budget: self.frame_budget,
            poll_timeout: self.poll_timeout,
            start: Instant::now(),
        }
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The application runtime.
///
/// Owns the terminal backend. Call [`App::run`] with a closure that builds a
/// widget tree each frame. The renderer is constructed per `run` invocation.
pub struct App {
    backend: Box<dyn Backend>,
    frame_budget: Duration,
    poll_timeout: Duration,
    start: Instant,
}

impl Default for App {
    fn default() -> Self {
        AppBuilder::new().build()
    }
}

impl App {
    /// Construct a builder for customizing the app.
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    /// Run the event loop. `render` is called each frame with a fresh
    /// [`Context`]; it should return the root widget for this frame.
    ///
    /// The loop exits when the user calls [`Context::exit`] or when the
    /// backend reports an error.
    pub async fn run<F, W>(&mut self, mut render: F) -> Result<()>
    where
        F: FnMut(&mut Context) -> W,
        W: Widget + 'static,
    {
        self.backend.enter()?;
        // Ensure we leave raw mode even on error.
        let result = self.run_loop(&mut render).await;
        let _ = self.backend.leave();
        result
    }

    async fn run_loop<F, W>(&mut self, render: &mut F) -> Result<()>
    where
        F: FnMut(&mut Context) -> W,
        W: Widget + 'static,
    {
        let (w, h) = self.backend.size()?;
        let mut renderer = Renderer::new(w, h);
        let mut next_wakeup: Option<Instant> = None;
        let mut state = StateStore::default();

        loop {
            let frame_start = Instant::now();
            let (w, h) = self.backend.size()?;
            if (w, h) != (renderer.curr.width, renderer.curr.height) {
                renderer.resize(w, h);
            }

            // Poll for an event, but no longer than the poll timeout or the
            // next wakeup deadline (whichever is sooner).
            let poll_ms = {
                let poll_deadline = frame_start + self.poll_timeout;
                let earliest = match next_wakeup {
                    Some(wu) if wu < poll_deadline => wu,
                    _ => poll_deadline,
                };
                let now = Instant::now();
                if earliest <= now {
                    0
                } else {
                    earliest.duration_since(now).as_millis() as u64
                }
            };
            let event = self.backend.poll(poll_ms)?;

            // Determine which event to surface this frame.
            let surfaced_event = if let Some(ev) = event {
                Some(ev)
            } else if let Some(wu) = next_wakeup {
                if Instant::now() >= wu {
                    Some(Event::Wakeup)
                } else {
                    None
                }
            } else {
                None
            };

            let elapsed = self.start.elapsed();
            let mut ctx = Context {
                event: surfaced_event,
                size: (w, h),
                elapsed,
                should_exit: false,
                wakeup: None,
                state: std::mem::take(&mut state),
            };

            // Build the widget tree.
            let root_widget = render(&mut ctx);
            let root: Box<dyn Widget> = Box::new(root_widget);
            let tree = WidgetTree::build(root);

            // Compute layout.
            let rects = tree.compute_rects(f32::from(w), f32::from(h));

            // Paint.
            renderer.begin();
            {
                let buf = renderer.buffer();
                tree.paint(buf, &rects, elapsed);
            }

            // Present.
            renderer.present(self.backend.as_mut())?;

            // Track requested wakeup.
            next_wakeup = ctx.wakeup;

            // Persist state.
            state = std::mem::take(&mut ctx.state);

            if ctx.should_exit {
                return Ok(());
            }

            // Yield to the runtime so other tasks (e.g. streaming LLM tokens)
            // can make progress.
            tokio::task::yield_now().await;

            // Sleep remaining frame budget.
            let elapsed_frame = frame_start.elapsed();
            if let Some(remaining) = self.frame_budget.checked_sub(elapsed_frame) {
                tokio::time::sleep(remaining).await;
            }
        }
    }
}

/// A handle for sending events into an [`App`] from other tasks (e.g. an
/// async LLM stream producing tokens).
///
/// Construct via [`App::spawner`] inside the render closure; the handle is
/// cheap to clone. Events sent through it are surfaced as [`Event::User`] on
/// the next frame.
#[derive(Clone)]
pub struct EventSender {
    inner: Arc<Mutex<Option<Event>>>,
}

impl EventSender {
    /// Send a user event. Only the most recently sent event is surfaced each
    /// frame; if you need a queue, wrap your own channel.
    pub fn send(&self, event: Event) {
        *self.inner.lock() = Some(event);
    }
}
