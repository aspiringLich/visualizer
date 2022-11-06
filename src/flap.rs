use bevy::prelude::*;
use bird::*;

pub fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::ORANGE_RED,
            anchor: bevy::sprite::Anchor::Center,
            ..default()
        },
        ..default()
    });
}

pub fn render(state: Local<BirdState>, mut commands: Commands) {}
// // hi breon
