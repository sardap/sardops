use core::{ops::Range, time::Duration};

use glam::Vec2;
use heapless::Vec;

use crate::{
    assets::StaticImage,
    display::{ComplexRender, ComplexRenderOption, GameDisplay},
    geo::RectVec2,
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
    remaining: Range<Duration>,
    area: RectVec2,
    dir: Range<Vec2>,
    images: &'static [&'static StaticImage],
}

impl ParticleTemplate {
    pub const fn new(
        remaining: Range<Duration>,
        area: RectVec2,
        dir: Range<Vec2>,
        images: &'static [&'static StaticImage],
    ) -> Self {
        Self {
            remaining,
            area,
            dir,
            images,
        }
    }

    pub fn instantiate(&self, rng: &mut fastrand::Rng) -> Particle {
        Particle {
            remaining: Duration::from_micros(rng.u64(
                (self.remaining.start.as_micros() as u64)..(self.remaining.end.as_micros() as u64),
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

pub type ParticleSpawnFn = fn(args: &mut ParticleTickArgs) -> (&'static ParticleTemplate, u8);

#[derive(Clone, Copy)]
pub struct Particle {
    remaining: Duration,
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

#[derive(Debug, Clone)]
pub enum SpawnTrigger {
    TimerConst {
        current: Duration,
        max: Duration,
    },
    TimerRange {
        current: Duration,
        current_target: Duration,
        target: Range<Duration>,
    },
}

impl SpawnTrigger {
    pub fn timer_const(max: Duration) -> Self {
        Self::TimerConst {
            current: Duration::ZERO,
            max,
        }
    }

    pub const fn timer_range(max_range: Range<Duration>) -> Self {
        Self::TimerRange {
            current: Duration::ZERO,
            current_target: Duration::ZERO,
            target: max_range,
        }
    }

    pub fn tick(&mut self, delta: Duration, rng: &mut fastrand::Rng) -> bool {
        match self {
            SpawnTrigger::TimerConst { current, max } => {
                *current += delta;
                if *current >= *max {
                    *current = Duration::ZERO;
                    return true;
                }
            }
            SpawnTrigger::TimerRange {
                current,
                current_target,
                target,
            } => {
                *current += delta;
                if *current > *current_target {
                    *current = Duration::ZERO;
                    *current_target = Duration::from_millis(
                        rng.u64((target.start.as_millis() as u64)..(target.end.as_millis() as u64)),
                    );
                    return true;
                }
            }
        };

        false
    }
}

#[derive(Clone)]
pub struct Spawner {
    pub name: &'static str,
    trigger: SpawnTrigger,
    spawn_fn: ParticleSpawnFn,
}

impl Spawner {
    pub const fn new(name: &'static str, trigger: SpawnTrigger, spawn_fn: ParticleSpawnFn) -> Self {
        Self {
            name,
            trigger,
            spawn_fn,
        }
    }

    pub fn tick(&mut self, args: &mut ParticleTickArgs) -> Option<(&'static ParticleTemplate, u8)> {
        if self.trigger.tick(args.delta, args.rng) {
            Some((self.spawn_fn)(args))
        } else {
            None
        }
    }
}

pub struct ParticleSystem<const MAX_PARTICLES: usize, const MAX_SPAWN_FUNCS: usize> {
    spawners: Vec<Spawner, MAX_SPAWN_FUNCS>,
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

    pub fn with_spawner(mut self, spawner: Spawner) -> Self {
        self.add_spawner(spawner);
        self
    }

    pub fn add_spawner(&mut self, spawner: Spawner) {
        let _ = self.spawners.push(spawner);
    }

    pub fn remove_spawner(&mut self, name: &'static str) {
        if let Some((index, _)) = self
            .spawners
            .iter()
            .enumerate()
            .find(|(_, i)| i.name == name)
        {
            self.spawners.remove(index);
        }
    }

    pub fn run_once_spawner(&mut self, spawner: ParticleSpawnFn, args: &mut ParticleTickArgs) {
        let particles = &mut self.particles;
        let (template, count) = spawner(args);
        let mut remaining = count;
        for range in &[
            self.last_particle_index..particles.len(),
            0..self.last_particle_index,
        ] {
            for i in range.start..range.end {
                if particles[i].is_none() {
                    particles[i] = Some(template.instantiate(args.rng));
                    self.last_particle_index = i;
                    remaining -= 1;

                    if remaining <= 0 {
                        break;
                    }
                }
            }

            if remaining <= 0 {
                break;
            }
        }
    }

    pub fn tick(&mut self, args: &mut ParticleTickArgs) {
        let particles = &mut self.particles;
        // tick existing
        for i in 0..particles.len() {
            let mut should_drop = false;
            if let Some(particle) = particles.get_mut(i).unwrap() {
                particle.remaining = particle
                    .remaining
                    .checked_sub(args.delta)
                    .unwrap_or(Duration::ZERO);
                if particle.remaining <= Duration::ZERO {
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
        for spawner in &mut self.spawners {
            if let Some((template, count)) = spawner.tick(args) {
                let mut remaining = count;
                for range in &[
                    self.last_particle_index..particles.len(),
                    0..self.last_particle_index,
                ] {
                    for i in range.start..range.end {
                        if particles[i].is_none() {
                            particles[i] = Some(template.instantiate(args.rng));
                            self.last_particle_index = i;
                            remaining -= 1;

                            if remaining <= 0 {
                                break;
                            }
                        }
                    }

                    if remaining <= 0 {
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
