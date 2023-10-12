use std::f32::consts::PI;

use glam::Vec2;
use glutin::context::PossiblyCurrentContext;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::window::Window;

use crate::fps::FpsCounter;

pub struct ApplicationState {
    pub gl_context: Option<PossiblyCurrentContext>,
    pub window: Window,
    pub surface_dimensions: PhysicalSize<u32>,
    rng: ThreadRng,

    pub paused: bool,

    // particles
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub densities: Vec<f32>,
    pub predicted_positions: Vec<Vec2>,

    pub fps_counter: FpsCounter,
}

const PARTICLE_COUNT: usize = 500;
impl ApplicationState {
    pub const PARTICLE_RADIUS: f32 = 15.0;

    const MASS: f32 = 1.0;
    const TARGET_DENSITY: f32 = 0.5;
    const SMOOTHING_RADIUS: f32 = 0.25;
    const COLLISION_DAMPING: f32 = 0.5;
    const PRESSURE_MULTIPLIER: f32 = 5.0;

    pub fn new(window: Window) -> ApplicationState {
        let window_dimensions = window.inner_size();
        ApplicationState {
            gl_context: None,
            window,
            surface_dimensions: window_dimensions,
            rng: thread_rng(),

            paused: false,

            positions: generate_grid(PARTICLE_COUNT, true),
            velocities: vec![Vec2::ZERO; PARTICLE_COUNT],
            densities: vec![0.0; PARTICLE_COUNT],
            predicted_positions: vec![Vec2::ZERO; PARTICLE_COUNT],

            fps_counter: FpsCounter::new(),
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        for i in 0..PARTICLE_COUNT {
            self.predicted_positions[i] =
                self.positions[i] + self.velocities[i] * Vec2::new(1.0 / 120.0, 1.0 / 120.0);
        }

        for i in 0..PARTICLE_COUNT {
            self.densities[i] = self.calculate_density(&self.predicted_positions[i]);
        }

        for i in 0..PARTICLE_COUNT {
            let pressure_force = self.calculate_pressure_force(i);
            let pressure_accel = pressure_force / self.densities[i];
            self.velocities[i] += pressure_accel * delta_time;
        }

        for i in 0..PARTICLE_COUNT {
            self.positions[i] += self.velocities[i] * delta_time;
        }

        self.resolve_collisions();

        self.fps_counter.update();
    }

    fn calculate_pressure_force(&mut self, idx: usize) -> Vec2 {
        let mut pressure_force = Vec2::ZERO;
        for other_idx in 0..PARTICLE_COUNT {
            if other_idx == idx {
                continue;
            }

            let offset = self.predicted_positions[other_idx] - self.predicted_positions[idx];
            let dist = offset.length();
            let dir = if dist == 0.0 {
                self.rng.gen::<Vec2>().normalize()
            } else {
                offset / dist
            };

            let slope = smoothing_kernel_derivative(dist, Self::SMOOTHING_RADIUS);
            let density = self.densities[other_idx];
            let shared_pressure = self.calculate_shared_pressure(density, self.densities[idx]);
            pressure_force += shared_pressure * dir * slope * Self::MASS / density;
        }

        pressure_force
    }

    fn convert_density_to_pressure(&self, density: f32) -> f32 {
        let density_err = density - Self::TARGET_DENSITY;
        let pressure = density_err * Self::PRESSURE_MULTIPLIER;
        pressure
    }

    fn calculate_shared_pressure(&self, density_a: f32, density_b: f32) -> f32 {
        let pressure_a = self.convert_density_to_pressure(density_a);
        let pressure_b = self.convert_density_to_pressure(density_b);
        (pressure_a + pressure_b) / 2.0
    }

    fn resolve_collisions(&mut self) {
        for i in 0..PARTICLE_COUNT {
            let p = &mut self.positions[i];
            let v = &mut self.velocities[i];
            if p.x.abs() > 1.0 {
                p.x = 1.0 * p.x.signum();
                v.x *= -1.0 * Self::COLLISION_DAMPING;
            }
            if p.y.abs() > 1.0 {
                p.y = 1.0 * p.y.signum();
                v.y *= -1.0 * Self::COLLISION_DAMPING;
            }
        }
    }

    fn calculate_density(&self, point: &Vec2) -> f32 {
        let mut density = 0.0;

        for other in &self.positions {
            let dist = (*other - *point).length();
            let influence = smoothing_kernel(dist, Self::SMOOTHING_RADIUS);
            density += Self::MASS * influence;
        }

        density
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

fn smoothing_kernel(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    let volume = (PI * radius.powi(4)) / 6.0;
    (radius - dist) * (radius - dist) / volume
}

fn smoothing_kernel_derivative(dist: f32, radius: f32) -> f32 {
    if dist >= radius {
        return 0.0;
    }

    let scale = 12.0 / (radius.powi(4) * PI);
    (dist - radius) * scale
}

fn generate_grid(n: usize, rand: bool) -> Vec<Vec2> {
    let mut points = Vec::new();
    let size = (n as f32).sqrt() as usize;

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
