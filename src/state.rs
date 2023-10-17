//! A particle simulation system, largely inspired by Sebastian Lague's efforts:
//! https://www.youtube.com/watch?v=rSKMYc1CQHE

use std::f32::consts::PI;

use glam::{IVec2, Vec2};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::engine::Interaction;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }
}

pub struct State {
    rng: ThreadRng,

    pub bounding_box: Rect,

    // particles
    pub positions: Vec<Vec2>,
    pub predicted_positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub densities: Vec<f32>,

    // (particle_idx, cell_key)
    spatial_lookup: Vec<(usize, usize)>,
    start_indices: Vec<usize>,

    last_update_offset: f32,
}

const PARTICLE_COUNT: usize = 1200;
impl State {
    pub const PIXELS_PER_UNIT: f32 = 50.0;

    const TICK_RATE: f32 = 30.0;
    const TICK_DELTA: f32 = 1.0 / Self::TICK_RATE;

    const MASS: f32 = 1.0;
    const TARGET_DENSITY: f32 = 5.0;
    const SMOOTHING_RADIUS: f32 = 0.7;
    const COLLISION_DAMPING: f32 = 0.75;
    const PRESSURE_MULTIPLIER: f32 = 50.0;

    const INTERACTION_RADIUS: f32 = 1.5;
    const INTERACTION_STRENGTH: f32 = 5.0;

    pub fn smoothing_radius(&self) -> f32 {
        Self::SMOOTHING_RADIUS
    }

    pub fn new() -> State {
        let bounding_box = Rect::new(0.0, 0.0, 16.0, 9.0);
        let positions = generate_grid(bounding_box, PARTICLE_COUNT);
        State {
            rng: thread_rng(),

            bounding_box,

            positions,
            predicted_positions: vec![Vec2::ZERO; PARTICLE_COUNT],
            velocities: vec![Vec2::ZERO; PARTICLE_COUNT],
            densities: vec![0.0; PARTICLE_COUNT],

            spatial_lookup: vec![(0, 0); PARTICLE_COUNT],
            start_indices: vec![usize::MAX; PARTICLE_COUNT],

            last_update_offset: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32, interaction: Option<Interaction>) {
        let end = self.last_update_offset + delta_time;
        let mut t = Self::TICK_DELTA;

        while t < end {
            self.tick(Self::TICK_DELTA, interaction.as_ref());
            t += Self::TICK_DELTA;
        }

        self.last_update_offset = end % Self::TICK_DELTA;
    }

    fn tick(&mut self, delta_time: f32, interaction: Option<&Interaction>) {
        // apply user input
        match interaction {
            Some(interaction) => {
                let (pos, strength) = match interaction {
                    Interaction::Repel(pos) => (pos, -Self::INTERACTION_STRENGTH),
                    Interaction::Suck(pos) => (pos, Self::INTERACTION_STRENGTH),
                };

                for i in 0..PARTICLE_COUNT {
                    let interaction_force =
                        self.interaction_force(*pos, Self::INTERACTION_RADIUS, strength, i);
                    self.velocities[i] += interaction_force;
                }
            }
            _ => (),
        }

        self.update_spatial_lookup();

        // predict next positions
        for i in 0..PARTICLE_COUNT {
            self.predicted_positions[i] =
                self.positions[i] + self.velocities[i] * (Vec2::ONE * Self::TICK_DELTA);
        }

        // calculate densities
        for i in 0..PARTICLE_COUNT {
            self.densities[i] = self.calculate_density(i);
        }

        // calculate velocities
        for i in 0..PARTICLE_COUNT {
            let pressure_force = self.calculate_pressure_force(i);
            let pressure_accel = pressure_force / self.densities[i];
            self.velocities[i] += pressure_accel * delta_time;
        }

        // move particles
        for i in 0..PARTICLE_COUNT {
            self.positions[i] += self.velocities[i] * delta_time;
        }

        self.resolve_collisions();
    }

    fn get_neighbours_by_idx(&self, idx: usize) -> Vec<usize> {
        self.get_neighbours_by_pos(self.positions[idx])
    }

    fn get_neighbours_by_pos(&self, world_pos: Vec2) -> Vec<usize> {
        let center_pos = world_pos_to_cell_pos(world_pos, Self::SMOOTHING_RADIUS);
        let sqr_radius = Self::SMOOTHING_RADIUS * Self::SMOOTHING_RADIUS;

        const OFFSETS: [IVec2; 9] = [
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(-1, 0),
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(-1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ];

        let mut neighbours = vec![];
        for offset in OFFSETS {
            let cell_key = create_cell_hash(center_pos + offset) % self.spatial_lookup.len();
            let cell_start_idx = self.start_indices[cell_key];

            for i in cell_start_idx..self.spatial_lookup.len() {
                if self.spatial_lookup[i].1 != cell_key {
                    break;
                }

                let (particle_idx, _) = self.spatial_lookup[i];
                let sqr_dist = (self.positions[particle_idx] - world_pos).length_squared();

                if sqr_dist <= sqr_radius {
                    neighbours.push(particle_idx);
                }
            }
        }

        neighbours
    }

    fn update_spatial_lookup(&mut self) {
        for i in 0..PARTICLE_COUNT {
            let cell_pos = world_pos_to_cell_pos(self.positions[i], Self::SMOOTHING_RADIUS);
            let cell_key = create_cell_hash(cell_pos) % self.spatial_lookup.len();
            self.spatial_lookup[i] = (i, cell_key);
            self.start_indices[i] = usize::MAX;
        }

        self.spatial_lookup.sort_by_key(|(_, cell_key)| *cell_key);

        for i in 0..self.spatial_lookup.len() {
            let (_, cell_key) = self.spatial_lookup[i];
            let prev_cell_key = if i == 0 {
                usize::MAX
            } else {
                let (_, key_prev) = self.spatial_lookup[i - 1];
                key_prev
            };

            if cell_key != prev_cell_key {
                self.start_indices[cell_key] = i;
            }
        }
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
                offset.normalize()
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
            let dst = offset.length();
            let dir = if dst == 0.0 {
                self.rng.gen::<Vec2>()
            } else {
                offset
            }
            .normalize();

            let slope = smoothing_kernel_derivative(dst, Self::SMOOTHING_RADIUS);
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

            if p.x < self.bounding_box.left() {
                p.x = self.bounding_box.left();
                v.x *= v.x.signum() * Self::COLLISION_DAMPING;
            }
            if p.x > self.bounding_box.right() {
                p.x = self.bounding_box.right();
                v.x *= -v.x.signum() * Self::COLLISION_DAMPING;
            }
            if p.y < self.bounding_box.top() {
                p.y = self.bounding_box.top();
                v.y *= v.y.signum() * Self::COLLISION_DAMPING;
            }
            if p.y > self.bounding_box.bottom() {
                p.y = self.bounding_box.bottom();
                v.y *= -v.y.signum() * Self::COLLISION_DAMPING;
            }
        }
    }

    fn calculate_density(&self, idx: usize) -> f32 {
        let mut density = 0.0;

        for other_idx in self.get_neighbours_by_idx(idx) {
            let dist = (self.positions[other_idx] - self.positions[idx]).length();
            let influence = smoothing_kernel(dist, Self::SMOOTHING_RADIUS);
            density += influence;
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

fn generate_grid(bounding_box: Rect, n: usize) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        points.push(Vec2::new(
            rng.gen::<f32>() * bounding_box.w + bounding_box.x,
            rng.gen::<f32>() * bounding_box.h + bounding_box.y,
        ))
    }

    points
}

fn world_pos_to_cell_pos(world_pos: Vec2, smoothing_radius: f32) -> IVec2 {
    IVec2::new(
        (world_pos.x / smoothing_radius).floor() as i32,
        (world_pos.y / smoothing_radius).floor() as i32,
    )
}

fn create_cell_hash(cell_pos: IVec2) -> usize {
    let a = (cell_pos.x) as usize * 15823;
    let b = (cell_pos.y) as usize * 9737333;
    a + b
}
