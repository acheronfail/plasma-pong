use std::time::Instant;

use glutin::context::PossiblyCurrentContext;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;

macro_rules! make_vec {
    ($struct:ident, $($name:ident$(,)*)+) => {
        #[derive(Debug, Clone, PartialEq)]
        pub struct $struct {
            $(
                pub $name: f32,
            )+
        }

        impl $struct {
            #[allow(unused)]
            pub fn new($($name: f32,)+) -> $struct {
                $struct {
                    $(
                        $name,
                    )+
                }
            }

            #[allow(unused)]
            pub fn empty() -> $struct {
                $(
                    let $name = 0.0;
                )+
                $struct::new($($name,)+)
            }
        }
    };
}

make_vec!(Vec2, x, y);
make_vec!(Vec3, x, y, z);
make_vec!(Vec4, x, y, z, w);

pub struct FpsCounter {
    last_check: Instant,
    frames_since_last_check: f32,
    last_fps: f32,
}

impl FpsCounter {
    pub fn new() -> FpsCounter {
        FpsCounter {
            last_check: Instant::now(),
            frames_since_last_check: 0.0,
            last_fps: f32::INFINITY,
        }
    }

    pub fn update(&mut self) {
        let time = self.last_check.elapsed().as_secs_f32();
        if time < 0.5 {
            self.frames_since_last_check += 1.0;
        } else {
            self.last_fps = (1. / time) * self.frames_since_last_check;
            self.frames_since_last_check = 0.0;
            self.last_check = Instant::now();
        }
    }

    pub fn fps(&self) -> f32 {
        self.last_fps
    }
}

pub struct ApplicationState {
    pub gl_context: Option<PossiblyCurrentContext>,
    pub window: Window,
    pub surface_dimensions: PhysicalSize<u32>,

    pub paused: bool,
    pub particles: Vec<Vec2>,

    pub fps_counter: FpsCounter,
}

fn generate_grid(n: u32, rand: bool) -> Vec<Vec2> {
    let mut points = Vec::new();
    let size = (n as f32).sqrt() as u32;

    if rand {
        use rand::prelude::*;
        let mut rng = rand::thread_rng();

        for _ in 0..n {
            points.push(Vec2::new(
                rng.gen::<f32>() * 2.0 - 1.0,
                rng.gen::<f32>() * 2.0 - 1.0,
            ))
        }
    } else {
        let spacing = 2.0 / (size - 1) as f32;

        for i in 0..size {
            for j in 0..size {
                points.push(Vec2 {
                    x: -1.0 + spacing * i as f32,
                    y: -1.0 + spacing * j as f32,
                });
            }
        }
    }
    points
}

impl ApplicationState {
    pub fn new(window: Window) -> ApplicationState {
        let window_dimensions = window.inner_size();
        ApplicationState {
            gl_context: None,
            window,
            surface_dimensions: window_dimensions,

            paused: false,
            particles: generate_grid(500, true),

            fps_counter: FpsCounter::new(),
        }
    }

    pub fn after_update(&mut self) {
        self.fps_counter.update();
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => control_flow.set_exit(),
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
