use glutin::context::PossiblyCurrentContext;
use winit::{
    event::{ElementState, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
};

pub struct ApplicationState {
    pub gl_context: Option<PossiblyCurrentContext>,
    paused: bool,
}

impl ApplicationState {
    pub fn new() -> ApplicationState {
        ApplicationState {
            gl_context: None,
            paused: false,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                // close and exit when escape is pressed
                Some(VirtualKeyCode::Escape) => *control_flow = ControlFlow::Exit,
                // pause waveform render when space is pressed
                Some(VirtualKeyCode::Space) if input.state == ElementState::Pressed => {
                    self.toggle_pause()
                }

                _ => {}
            },
            _ => (),
        }
    }
}
