use crate::{
    items::{ItemExtra, ItemKind, UsableItem, UseItemOutput},
    scene::{
        SceneEnum, alarm_set_scene::AlarmSetScene, credits_scene::CreditsScene, fishing_scene,
        home_scene, star_gazing_scene,
    },
};

// const USE_SHOP_UPGRADE: UsableItem = UsableItem::new(ItemKind::ShopUpgrade, |game_ctx| {
//     game_ctx
//         .shop
//         .set_item_count(game_ctx.shop.get_item_count() + 1);

//     UseItemOutput::new().with_consumed()
// });

const USE_FISHING_ROD: UsableItem = UsableItem::new(ItemKind::FishingRod, |game_ctx| {
    let mut result =
        UseItemOutput::new().with_scene(SceneEnum::Fishing(fishing_scene::FishingScene::new()));
    let entry = game_ctx.inventory.get_entry_mut(ItemKind::FishingRod);
    entry.item_extra.uses -= 1;
    if entry.item_extra.uses <= 0 {
        entry.item_extra = ItemExtra::new_from_kind(ItemKind::FishingRod);
        result = result.with_consumed();
    }

    result
})
.with_is_usable_fn(|game_ctx| {
    !matches!(
        game_ctx.home.state,
        home_scene::State::Exploring | home_scene::State::GoneOut { outing_end_time: _ }
    )
});

const USE_FISH: UsableItem = UsableItem::new(ItemKind::Fish, |game_ctx| {
    game_ctx.home_fish_tank.add(&mut game_ctx.rng);
    UseItemOutput::new().with_consumed()
})
.with_is_usable_fn(|game_ctx| game_ctx.inventory.has_item(ItemKind::FishTank));

const USE_TELESCOPE: UsableItem = UsableItem::new(ItemKind::Telescope, |_| {
    UseItemOutput::new().with_scene(SceneEnum::StarGazing(
        star_gazing_scene::StarGazingScene::new(),
    ))
})
.with_is_usable_fn(|game_ctx| game_ctx.inventory.has_item(ItemKind::Telescope));

const USE_ALARM: UsableItem = UsableItem::new(ItemKind::Alarm, |_| {
    UseItemOutput::new().with_scene(SceneEnum::AlarmSet(AlarmSetScene::new()))
})
.with_is_usable_fn(|game_ctx| {
    game_ctx.inventory.has_item(ItemKind::AnalogueClock)
        || game_ctx.inventory.has_item(ItemKind::DigitalClock)
});

const USE_CREDITS: UsableItem = UsableItem::new(ItemKind::CreditsScroll, |_| {
    UseItemOutput::new().with_scene(SceneEnum::Credits(CreditsScene::new()))
});

pub const ALL_USEABLE_ITEMS: &[UsableItem] = &[
    // USE_SHOP_UPGRADE,
    USE_FISHING_ROD,
    USE_FISH,
    USE_TELESCOPE,
    USE_ALARM,
    USE_CREDITS,
];
