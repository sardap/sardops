use core::time::Duration;

use crate::{
    game_consts::SIM_LENGTH_STEP, poop::add_poop, scene::SceneTickArgs,
    temperature::TemperatureLevel,
};

pub fn tick_sim(time_scale: f32, args: &mut SceneTickArgs) {
    let delta = args.delta.mul_f32(time_scale);

    let mut runs = delta.as_micros() as u64 / SIM_LENGTH_STEP.as_micros() as u64;
    let left_over = delta.as_micros() as u64 % SIM_LENGTH_STEP.as_micros() as u64;

    args.game_ctx.sim_extra += Duration::from_micros(left_over);
    while args.game_ctx.sim_extra > SIM_LENGTH_STEP {
        args.game_ctx.sim_extra -= SIM_LENGTH_STEP;
        runs += 1;
    }

    let delta = SIM_LENGTH_STEP;

    for _ in 0..runs {
        if let Some(egg) = &mut args.game_ctx.egg {
            egg.sim_tick(delta, &mut args.game_ctx.sim_rng);
        }

        if args.game_ctx.pet.should_die().is_none() {
            args.game_ctx.pet.tick_mood(
                &args.game_ctx.poops,
                TemperatureLevel::from(args.input.temperature()),
                &args.game_ctx.home_layout,
            );
            args.game_ctx
                .pet
                .tick_breed(&mut args.game_ctx.sim_rng, args.game_ctx.egg.is_some());

            let poop_count = args.game_ctx.poop_count() as u8;
            let pet = &mut args.game_ctx.pet;

            pet.tick_age(delta);

            let sleeping = pet.definition().should_be_sleeping(&args.timestamp);

            pet.tick_hunger(delta, sleeping);
            pet.tick_poop(delta);
            pet.tick_since_game(delta, sleeping);
            pet.tick_death(delta, &mut args.game_ctx.sim_rng, sleeping, poop_count);
            pet.tick_evolve(delta, &args.game_ctx.inventory);
            pet.tick_illness(&mut args.game_ctx.sim_rng, delta);
            if pet.should_poop(&mut args.game_ctx.sim_rng, sleeping) {
                add_poop(&mut args.game_ctx.poops, args.timestamp);
            }
            args.game_ctx
                .suiter_system
                .sim_tick(delta, &mut args.game_ctx.sim_rng, pet, sleeping);
        }
    }
}
