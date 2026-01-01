use core::{ops::Range, time::Duration};

use glam::Vec2;
use heapless::Vec;

use crate::{
    assets::StaticImage,
    display::{ComplexRender, ComplexRenderOption, GameDisplay, Rotation},
    geo::RectVec2,
    sprite::Sprite,
};

pub struct Particles<'a> {
    particles: &'a mut [Option<Particle>],
    last_particle_index: &'a mut usize,
}

impl<'a> Particles<'a> {
    fn new(particles: &'a mut [Option<Particle>], last_particle_index: &'a mut usize) -> Self {
        Self {
            particles,
            last_particle_index,
        }
    }

    pub fn add(&mut self, particle: Particle) {
        for _ in 0..self.particles.len() {
            if *self.last_particle_index >= self.particles.len() {
                *self.last_particle_index = 0;
            }

            if self.particles[*self.last_particle_index].is_none() {
                self.particles[*self.last_particle_index] = Some(particle);
                break;
            }
            *self.last_particle_index += 1;
        }
    }
}

pub struct ParticleSpawnArgs<'a> {
    pub delta: Duration,
    pub rng: &'a mut fastrand::Rng,
    // This 100% is bad and stupid
    pub pet_pos: &'a Vec2,
}

impl<'a> ParticleSpawnArgs<'a> {
    pub const fn new(delta: Duration, rng: &'a mut fastrand::Rng, pet_pos: &'a Vec2) -> Self {
        Self {
            delta,
            rng,
            pet_pos,
        }
    }
}

const DEFAULT_ROTATION: &'static [Rotation] = &[Rotation::R0];

pub enum TemplateCullTatic {
    Remaning(Range<Duration>),
    OutsideRect(RectVec2),
}

pub struct ParticleTemplate {
    cull: TemplateCullTatic,
    area: RectVec2,
    dir: Range<Vec2>,
    images: &'static [&'static StaticImage],
    rotation: &'static [Rotation],
}

impl ParticleTemplate {
    pub const fn new(
        cull: TemplateCullTatic,
        area: RectVec2,
        dir: Range<Vec2>,
        images: &'static [&'static StaticImage],
    ) -> Self {
        Self {
            cull,
            area,
            dir,
            images,
            rotation: DEFAULT_ROTATION,
        }
    }

    pub const fn with_rotation(mut self, rotation: &'static [Rotation]) -> Self {
        debug_assert!(rotation.len() > 0);
        self.rotation = rotation;
        self
    }

    pub fn instantiate(&self, rng: &mut fastrand::Rng) -> Particle {
        Particle {
            cull: match &self.cull {
                TemplateCullTatic::Remaning(range) => CullTattic::Remaining(Duration::from_micros(
                    rng.u64((range.start.as_micros() as u64)..(range.end.as_micros() as u64)),
                )),
                TemplateCullTatic::OutsideRect(rect_vec2) => CullTattic::OutsideRect(*rect_vec2),
            },
            pos: Vec2::new(
                rng.i32((self.area.x() as i32)..(self.area.x2() as i32)) as f32 + rng.f32(),
                rng.i32((self.area.y() as i32)..(self.area.y2() as i32)) as f32 + rng.f32(),
            ),
            dir: Vec2::new(
                if self.dir.start.x == self.dir.end.x {
                    self.dir.start.x as f32
                } else {
                    rng.i32((self.dir.start.x as i32)..(self.dir.end.x as i32)) as f32 + rng.f32()
                },
                if self.dir.start.y == self.dir.end.y {
                    self.dir.start.y as f32
                } else {
                    rng.i32((self.dir.start.y as i32)..(self.dir.end.y as i32)) as f32 + rng.f32()
                },
            ),
            image: self.images[rng.usize(0..self.images.len())],
            rotation: *rng.choice(self.rotation).unwrap_or(&Rotation::R0),
        }
    }
}

pub type ParticleSpawnFn = fn(particles: &mut Particles, args: &mut ParticleSpawnArgs);

#[derive(Clone, Copy)]
pub enum CullTattic {
    Remaining(Duration),
    OutsideRect(RectVec2),
}

#[derive(Clone, Copy)]
pub struct Particle {
    cull: CullTattic,
    pos: Vec2,
    dir: Vec2,
    image: &'static StaticImage,
    rotation: Rotation,
}

impl Sprite for Particle {
    fn pos(&self) -> &glam::Vec2 {
        &self.pos
    }

    fn image(&self) -> &impl crate::assets::Image {
        self.image
    }

    fn size_x(&self) -> i32 {
        self.image.isize.x
    }

    fn size_y(&self) -> i32 {
        self.image.isize.y
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

    fn tick(&mut self, particles: &mut Particles, args: &mut ParticleSpawnArgs) {
        if self.trigger.tick(args.delta, args.rng) {
            (self.spawn_fn)(particles, args)
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

    pub fn run_once_spawner(&mut self, spawner: ParticleSpawnFn, args: &mut ParticleSpawnArgs) {
        let particles = &mut self.particles;
        spawner(
            &mut Particles::new(&mut self.particles, &mut self.last_particle_index),
            args,
        );
    }

    pub fn tick(&mut self, args: &mut ParticleSpawnArgs) {
        let particles = &mut self.particles;
        // tick existing
        for i in 0..particles.len() {
            let mut should_drop = false;
            if let Some(particle) = particles.get_mut(i).unwrap() {
                match &mut particle.cull {
                    CullTattic::Remaining(duration) => {
                        *duration = duration.checked_sub(args.delta).unwrap_or(Duration::ZERO);
                        should_drop = *duration <= Duration::ZERO;
                    }
                    CullTattic::OutsideRect(rect_vec2) => {
                        should_drop = !rect_vec2.point_inside(&particle.pos);
                    }
                }

                particle.pos.x += particle.dir.x * args.delta.as_secs_f32();
                particle.pos.y += particle.dir.y * args.delta.as_secs_f32();
            }

            if should_drop {
                particles[i] = None;
            }
        }

        let mut particles = Particles::new(particles, &mut self.last_particle_index);
        // Spawn new
        for spawner in &mut self.spawners {
            spawner.tick(&mut particles, args);
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
                    ComplexRenderOption::new()
                        .with_white()
                        .with_center()
                        .with_rotation(particle.rotation),
                );
            }
        }
    }
}
