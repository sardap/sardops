use core::time::Duration;

use crate::{poop::add_poop, scene::SceneTickArgs};

pub const SIM_LENGTH_STEP: Duration = Duration::from_millis(15);

pub fn tick_sim(time_scale: f32, args: &mut SceneTickArgs) {
    let delta = args.delta.mul_f32(time_scale);

    let mut runs = delta.as_micros() as u64 / SIM_LENGTH_STEP.as_micros() as u64;
    let left_over = delta.as_micros() as u64 % SIM_LENGTH_STEP.as_micros() as u64;

    args.game_ctx.sim_extra += Duration::from_micros(left_over);
    while args.game_ctx.sim_extra > SIM_LENGTH_STEP {
        args.game_ctx.sim_extra -= SIM_LENGTH_STEP;
        runs += 1;
    }

    for _ in 0..=runs {
        if let Some(egg) = &mut args.game_ctx.egg {
            egg.sim_tick(delta, &mut args.game_ctx.rng);
        }

        if args.game_ctx.pet.should_die().is_none() {
            args.game_ctx.pet.tick_mood(&args.game_ctx.poops);
            args.game_ctx
                .pet
                .tick_breed(&mut args.game_ctx.rng, args.game_ctx.egg.is_some());

            let poop_count = args.game_ctx.poop_count() as u8;
            let pet = &mut args.game_ctx.pet;

            pet.tick_age(delta);

            let sleeping = pet.definition().should_be_sleeping(&args.timestamp);

            pet.tick_hunger(delta, sleeping);
            pet.tick_poop(delta);
            pet.tick_since_game(delta, sleeping);
            pet.tick_death(delta, &mut args.game_ctx.rng, sleeping, poop_count);
            pet.tick_evolve(delta, &args.game_ctx.inventory);
            if pet.should_poop(sleeping) {
                add_poop(&mut args.game_ctx.poops, args.timestamp);
            }
            args.game_ctx
                .suiter_system
                .sim_tick(delta, &mut args.game_ctx.rng, pet, sleeping);
        }
    }
}
