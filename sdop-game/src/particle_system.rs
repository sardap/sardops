use core::{ops::Range, time::Duration};

use glam::Vec2;
use heapless::Vec;
use log::info;

use crate::{
    assets::StaticImage,
    display::{ComplexRender, ComplexRenderOption, GameDisplay},
    geo::Rect,
    sprite::Sprite,
};

pub struct ParticleTickArgs<'a> {
    pub delta: Duration,
    pub rng: &'a mut fastrand::Rng,
}

impl<'a> ParticleTickArgs<'a> {
    pub const fn new(delta: Duration, rng: &'a mut fastrand::Rng) -> Self {
        Self { delta, rng }
    }
}

pub struct ParticleTemplate {
    remaing: Range<Duration>,
    area: Rect,
    dir: Range<Vec2>,
    images: &'static [&'static StaticImage],
}

impl ParticleTemplate {
    pub fn new(
        remaing: Range<Duration>,
        area: Rect,
        dir: Range<Vec2>,
        images: &'static [&'static StaticImage],
    ) -> Self {
        Self {
            remaing,
            area,
            dir,
            images,
        }
    }

    pub fn instantiate(&self, rng: &mut fastrand::Rng) -> Particle {
        Particle {
            remaning: Duration::from_micros(rng.u64(
                (self.remaing.start.as_micros() as u64)..(self.remaing.end.as_micros() as u64),
            )),
            pos: Vec2::new(
                rng.i32((self.area.x() as i32)..(self.area.x2() as i32)) as f32 + rng.f32(),
                rng.i32((self.area.y() as i32)..(self.area.y2() as i32)) as f32 + rng.f32(),
            ),
            dir: Vec2::new(
                rng.i32((self.dir.start.x as i32)..(self.dir.end.x as i32)) as f32 + rng.f32(),
                rng.i32((self.dir.start.y as i32)..(self.dir.end.y as i32)) as f32 + rng.f32(),
            ),
            image: self.images[rng.usize(0..self.images.len())],
        }
    }
}

pub type ParticleSpawnFn = fn(args: &mut ParticleTickArgs) -> Option<(ParticleTemplate, u8)>;

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

pub struct ParticleSystem<const MAX_PARTICLES: usize, const MAX_SPAWN_FUNCS: usize> {
    spawners: Vec<ParticleSpawnFn, MAX_SPAWN_FUNCS>,
    particles: [Option<Particle>; MAX_PARTICLES],
    last_particle_index: usize,
}

impl<const MAX_PARTICLES: usize, const MAX_SPAWN_FUNCS: usize> Default
    for ParticleSystem<MAX_PARTICLES, MAX_SPAWN_FUNCS>
{
    fn default() -> Self {
        Self {
            spawners: Vec::new(),
            particles: [(); MAX_PARTICLES].map(|_| None),
            last_particle_index: 0,
        }
    }
}
impl<const MAX_PARTICLES: usize, const MAX_SPAWN_FUNCS: usize>
    ParticleSystem<MAX_PARTICLES, MAX_SPAWN_FUNCS>
{
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

    pub fn run_once_spwaner(&mut self, spawner: ParticleSpawnFn, args: &mut ParticleTickArgs) {
        let particles = &mut self.particles;
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

impl<const MAX_PARTICLES: usize, const MAX_SPAWN_FUNCS: usize> ComplexRender
    for ParticleSystem<MAX_PARTICLES, MAX_SPAWN_FUNCS>
{
    fn render(&self, display: &mut GameDisplay) {
        for particle in &self.particles {
            if let Some(particle) = particle {
                info!("Dota 2 {} {}", particle.pos.x, particle.pos.y);
                display.render_image_complex(
                    particle.pos.x as i32,
                    particle.pos.y as i32,
                    particle.image,
                    ComplexRenderOption::new().with_white().with_center(),
                );
            }
        }
    }
}
