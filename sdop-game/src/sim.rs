use crate::{poop::add_poop, scene::SceneTickArgs};

pub fn tick_sim(time_scale: f32, args: &mut SceneTickArgs) {
    let delta = args.delta.mul_f32(time_scale);

    if args.game_ctx.pet.should_die().is_none() {
        let poop_count = args.game_ctx.poop_count() as u8;
        let pet = &mut args.game_ctx.pet;

        pet.tick_age(delta);

        let sleeping = pet.definition().should_be_sleeping(&args.timestamp);

        pet.tick_hunger(delta, sleeping);
        pet.tick_poop(delta);
        pet.tick_since_game(delta, sleeping);
        pet.tick_death(delta, &mut args.game_ctx.rng, sleeping, poop_count);
        if pet.should_poop(sleeping) {
            add_poop(&mut args.game_ctx.poops, args.timestamp);
        }
    }
}
