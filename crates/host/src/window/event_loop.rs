//! The window's loop: native events in, normalized events out, a frame when
//! the window asks for one.
//!
//! Two things are the window's own and never reach the screen code: the text
//! size and full screen. Everything else is passed on.

use surface::{
    ButtonState, CellPosition, EventSink, Flow, HostError, InputEvent, Presenter,
};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use crate::font::FontSet;
use crate::input::{WheelCarry, key_event, middle_paste, modifiers, pointer_button, typed_text};

use super::create::create_window;
use super::options::WindowOptions;
use super::shortcuts::{Shortcut, shortcut_for};
use super::state::WindowState;

/// Opens the window and runs until the screen asks to stop or it is closed.
pub fn run_window(
    fonts: &FontSet,
    options: WindowOptions,
    sink: &mut dyn EventSink,
) -> Result<(), HostError> {
    let mut state = WindowState::new(fonts, options);
    state.run(sink)
}

/// The loop itself, for a state that is already built.
pub(crate) fn drive(state: &mut WindowState<'_>, sink: &mut dyn EventSink) -> Result<(), HostError> {
    let event_loop = EventLoop::<()>::with_user_event()
        .build()
        .map_err(|error| HostError(format!("no window system: {error}")))?;
    state.set_proxy(event_loop.create_proxy());
    let mut app = WindowApp {
        sink,
        state,
        modifiers: surface::Modifiers::default(),
        pointer: (0.0, 0.0),
        wheel: WheelCarry::default(),
    };
    event_loop
        .run_app(&mut app)
        .map_err(|error| HostError(format!("the window loop stopped: {error}")))?;
    match app.state.failure.take() {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct WindowApp<'a, 'f> {
    sink: &'a mut dyn EventSink,
    state: &'a mut WindowState<'f>,
    modifiers: surface::Modifiers,
    pointer: (f64, f64),
    wheel: WheelCarry,
}

impl ApplicationHandler<()> for WindowApp<'_, '_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.window().is_some() {
            return;
        }
        let metrics = self.state.logical_metrics();
        let window = match create_window(event_loop, self.state.options(), &metrics) {
            Ok(window) => window,
            Err(error) => return self.fail(event_loop, error),
        };
        if let Err(error) = self.state.attach(window) {
            return self.fail(event_loop, error);
        }
        self.grid_changed(event_loop);
        self.state.request_redraw();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, (): ()) {
        let flow = self.sink.on_wake(&mut *self.state);
        self.apply(event_loop, flow);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.deliver(event_loop, InputEvent::CloseRequested),
            WindowEvent::Resized(size) => self.resized(event_loop, size.width, size.height),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if self.state.set_scale_factor(scale_factor) {
                    self.grid_changed(event_loop);
                }
                self.state.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                self.sink.on_redraw(&mut *self.state);
                self.apply(event_loop, Flow::Continue);
            }
            WindowEvent::ModifiersChanged(state) => self.modifiers = modifiers(state.state()),
            WindowEvent::Focused(gained) => {
                self.deliver(event_loop, InputEvent::Focus { gained });
            }
            WindowEvent::KeyboardInput { event, .. } => self.key(event_loop, &event),
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer = (position.x, position.y);
                let pos = self.cell();
                let modifiers = self.modifiers;
                self.deliver(event_loop, InputEvent::PointerMoved { pos, modifiers });
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.button(event_loop, state, button);
            }
            WindowEvent::MouseWheel { delta, .. } => self.wheel(event_loop, delta),
            _ => {}
        }
    }
}

impl WindowApp<'_, '_> {
    /// The window grew or shrank: a new canvas, and a new grid if the cell
    /// count changed.
    fn resized(&mut self, event_loop: &ActiveEventLoop, width: u32, height: u32) {
        if self.state.resized(width, height) {
            self.grid_changed(event_loop);
        }
        self.state.request_redraw();
    }

    /// Tells the screen the grid it now has.
    fn grid_changed(&mut self, event_loop: &ActiveEventLoop) {
        let (cols, rows) = self.state.grid();
        self.deliver(event_loop, InputEvent::Resized { cols, rows });
    }

    fn key(&mut self, event_loop: &ActiveEventLoop, event: &winit::event::KeyEvent) {
        if self.window_shortcut(event_loop, event) {
            return;
        }
        if let Some(key) = key_event(event, self.modifiers) {
            self.deliver(event_loop, InputEvent::Key(key));
        }
        if let Some(text) = typed_text(event, self.modifiers) {
            self.deliver(event_loop, InputEvent::Text(text));
        }
    }

    /// Text size, full screen and paste, which belong to the window itself.
    fn window_shortcut(
        &mut self,
        event_loop: &ActiveEventLoop,
        event: &winit::event::KeyEvent,
    ) -> bool {
        if event.state != ElementState::Pressed {
            return false;
        }
        let Some(shortcut) = shortcut_for(&event.logical_key, self.modifiers) else {
            return false;
        };
        let points = self.state.points();
        let next = match shortcut {
            Shortcut::FullScreen => return self.toggle_full_screen(),
            Shortcut::Paste => return self.paste(event_loop),
            Shortcut::Larger => WindowOptions::stepped(points, 1),
            Shortcut::Smaller => WindowOptions::stepped(points, -1),
            Shortcut::ResetSize => self.state.options().points,
        };
        self.set_points(event_loop, next);
        true
    }

    fn toggle_full_screen(&mut self) -> bool {
        self.state.toggle_full_screen();
        true
    }

    /// The system clipboard's text goes to the screen as a paste. With
    /// nothing on the clipboard the key does nothing.
    fn paste(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if let Some(text) = self.state.clipboard_text() {
            self.deliver(event_loop, InputEvent::Paste(text));
        }
        true
    }

    /// A new text size: the grid changes with it, so the screen is told.
    fn set_points(&mut self, event_loop: &ActiveEventLoop, points: u8) {
        if self.state.set_points(points) {
            self.grid_changed(event_loop);
        }
        self.state.request_redraw();
    }

    fn button(
        &mut self,
        event_loop: &ActiveEventLoop,
        state: ElementState,
        button: winit::event::MouseButton,
    ) {
        let Some(button) = pointer_button(button) else {
            return;
        };
        let state = match state {
            ElementState::Pressed => ButtonState::Pressed,
            ElementState::Released => ButtonState::Released,
        };
        // On Linux a middle press pastes the primary selection, as in every
        // other program there; the screen has no other use for that button.
        if let Some(paste) = middle_paste(button, state, || self.state.primary_text()) {
            self.deliver(event_loop, paste);
            return;
        }
        let event = InputEvent::Button { button, state, pos: self.cell(), modifiers: self.modifiers };
        self.deliver(event_loop, event);
    }

    fn wheel(&mut self, event_loop: &ActiveEventLoop, delta: winit::event::MouseScrollDelta) {
        let cell = self.state.cell_size();
        let (rows, cols) = self.wheel.add(delta, cell);
        if rows == 0 && cols == 0 {
            return;
        }
        // Held down, the modifier turns the wheel into the text size; rows
        // count down, so a turn away from the user makes the text larger.
        if self.modifiers.ctrl {
            let step = if rows < 0 { 1 } else { -1 };
            let points = WindowOptions::stepped(self.state.points(), step);
            self.set_points(event_loop, points);
            return;
        }
        let event = InputEvent::Wheel {
            pos: self.cell(),
            rows,
            cols,
            modifiers: self.modifiers,
        };
        self.deliver(event_loop, event);
    }

    /// Where the pointer last was, as a cell.
    fn cell(&self) -> CellPosition {
        let (col, row, frac_x, frac_y) = self.state.cell_at(self.pointer.0, self.pointer.1);
        CellPosition {
            col,
            row,
            frac_x,
            frac_y,
        }
    }

    fn deliver(&mut self, event_loop: &ActiveEventLoop, event: InputEvent) {
        let flow = self.sink.on_input(event, &mut *self.state);
        self.apply(event_loop, flow);
    }

    /// Stops the loop when the screen is done or the window itself failed.
    fn apply(&mut self, event_loop: &ActiveEventLoop, flow: Flow) {
        if flow == Flow::Quit || self.state.failure.is_some() {
            event_loop.exit();
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: HostError) {
        self.state.failure = Some(error);
        event_loop.exit();
    }
}
