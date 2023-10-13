//! A particle simulation system, largely inspired by Sebastian Lague's efforts:
//! https://www.youtube.com/watch?v=rSKMYc1CQHE

use std::f32::consts::PI;

use glam::Vec2;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::engine::Interaction;

pub struct State {
    rng: ThreadRng,

    // particles
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub densities: Vec<f32>,
    pub predicted_positions: Vec<Vec2>,
}

const PARTICLE_COUNT: usize = 500;
impl State {
    pub const PARTICLE_RADIUS: f32 = 10.0;

    const MASS: f32 = 1.0;
    const TARGET_DENSITY: f32 = 0.5;
    const SMOOTHING_RADIUS: f32 = 0.25;
    const COLLISION_DAMPING: f32 = 0.5;
    const PRESSURE_MULTIPLIER: f32 = 5.0;

    pub fn new() -> State {
        State {
            rng: thread_rng(),

            positions: generate_grid(PARTICLE_COUNT, true),
            velocities: vec![Vec2::ZERO; PARTICLE_COUNT],
            densities: vec![0.0; PARTICLE_COUNT],
            predicted_positions: vec![Vec2::ZERO; PARTICLE_COUNT],
        }
    }

    pub fn update(&mut self, delta_time: f32, interaction: Option<Interaction>) {
        match interaction {
            Some(interaction) => {
                let (pos, radius, strength) = match interaction {
                    Interaction::Repel { pos, radius } => (pos, radius, -1.0),
                    Interaction::Suck { pos, radius } => (pos, radius, 1.0),
                };

                for i in 0..PARTICLE_COUNT {
                    let interaction_force = self.interaction_force(pos, radius, strength, i);
                    self.velocities[i] += interaction_force;
                }
            }
            _ => (),
        }

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
    }

    fn interaction_force(&self, input: Vec2, radius: f32, strength: f32, idx: usize) -> Vec2 {
        let offset = input - self.positions[idx];
        let sqr_dist = offset.length_squared();

        // if particle is inside input radius, calculate force towards input point
        if sqr_dist < radius * radius {
            let dist = sqr_dist.sqrt();
            let dir_to_input_point = if dist <= f32::EPSILON {
                Vec2::ZERO
            } else {
                offset / dist
            };

            // value is 1 when particle is exactly at input point; 0 when at edge of input circle
            let center_t = 1.0 - dist / radius;
            // calculate the force (velocity is subtracted to slow the particle down)
            (dir_to_input_point * strength - self.velocities[idx]) * center_t
        } else {
            Vec2::ZERO
        }
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
