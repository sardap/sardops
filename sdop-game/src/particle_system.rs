use core::{ops::Range, time::Duration};

use glam::Vec2;
use heapless::Vec;

use crate::{assets::StaticImage, sprite::Sprite};

pub struct ParticleTickArgs<'a> {
    delta: Duration,
    rng: &'a mut fastrand::Rng,
}

impl<'a> ParticleTickArgs<'a> {
    pub const fn new(delta: Duration, rng: &'a mut fastrand::Rng) -> Self {
        Self { delta, rng }
    }
}

pub struct ParticleTemplate {
    remaing: Range<Duration>,
    pos: Range<Vec2>,
    dir: Range<Vec2>,
    images: &'static [&'static StaticImage],
}

impl ParticleTemplate {
    pub fn instantiate(&self, rng: &mut fastrand::Rng) -> Particle {
        Particle {
            remaning: Duration::from_micros(rng.u64(
                (self.remaing.start.as_micros() as u64)..(self.remaing.end.as_micros() as u64),
            )),
            pos: Vec2::new(
                rng.i32((self.pos.start.x as i32)..(self.pos.end.x as i32)) as f32 + rng.f32(),
                rng.i32((self.pos.start.y as i32)..(self.pos.end.y as i32)) as f32 + rng.f32(),
            ),
            dir: Vec2::new(
                rng.i32((self.dir.start.x as i32)..(self.dir.end.x as i32)) as f32 + rng.f32(),
                rng.i32((self.dir.start.y as i32)..(self.dir.end.y as i32)) as f32 + rng.f32(),
            ),
            image: self.images[rng.usize(0..self.images.len())],
        }
    }
}

pub type ParticleSpawnFn = fn(args: &ParticleTickArgs) -> Option<(ParticleTemplate, u8)>;

#[derive(Clone, Copy)]
pub struct Particle {
    remaning: Duration,
    pos: Vec2,
    dir: Vec2,
    image: &'static StaticImage,
}

impl Sprite for Particle {
    fn pos(&self) -> &glam::Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        self.image
    }
}

pub const MAX_PARTICLES: usize = 100;
pub const MAX_SPAWN_FUNCS: usize = 5;

pub struct ParticleSystem {
    spawners: Vec<ParticleSpawnFn, MAX_SPAWN_FUNCS>,
    particles: [Option<Particle>; MAX_PARTICLES],
    last_particle_index: usize,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            spawners: Vec::new(),
            particles: [None; MAX_PARTICLES],
            last_particle_index: 0,
        }
    }

    pub fn with_spawner(mut self, spawner: ParticleSpawnFn) -> Self {
        let _ = self.spawners.push(spawner);
        self
    }

    pub fn tick(&mut self, args: &mut ParticleTickArgs) {
        let spawners = &mut self.spawners;
        let particles = &mut self.particles;
        // tick existing
        for i in 0..particles.len() {
            let mut should_drop = false;
            if let Some(particle) = particles.get_mut(i).unwrap() {
                particle.remaning = particle
                    .remaning
                    .checked_sub(args.delta)
                    .unwrap_or(Duration::ZERO);
                if particle.remaning <= Duration::ZERO {
                    should_drop = true;
                } else {
                    particle.pos.x += particle.dir.x * args.delta.as_secs_f32();
                    particle.pos.y += particle.dir.y * args.delta.as_secs_f32();
                }
            }

            if should_drop {
                particles[i] = None;
            }
        }

        // Spawn new
        for spawner in spawners {
            if let Some((template, count)) = spawner(args) {
                let mut remaning = count;
                for range in &[
                    self.last_particle_index..particles.len(),
                    0..self.last_particle_index,
                ] {
                    for i in range.start..range.end {
                        if particles[i].is_none() {
                            particles[i] = Some(template.instantiate(args.rng));
                            self.last_particle_index = i;
                            remaning -= 1;

                            if remaning <= 0 {
                                break;
                            }
                        }
                    }

                    if remaning <= 0 {
                        break;
                    }
                }
            }
        }
    }
}
