use autodefault::autodefault;
use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_debug_text_overlay::OverlayPlugin;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_pancam::{PanCam, PanCamPlugin};
use iyes_loopless::prelude::*;

use bird::*;
use neat::*;

mod flap;
mod network;

#[autodefault]
fn main() {
    let genome = neat::main();
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("NEAT"),
        })
        .insert_resource(network::NetworkState {
            genes: genome.genes,
            sensor_nodes: genome.trainer.inputs,
            output_nodes: genome.trainer.outputs,
        })
        // base plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(OverlayPlugin { font_size: 32.0 })
        .add_plugin(PanCamPlugin::default())
        // .add_plugin(WorldInspectorPlugin::new())
        // startup system(s)
        .add_startup_system(setup_sys)
        .add_startup_system(flap::setup)
        // systems
        .add_system(flap::render)
        .add_system(network::network_ui)
        .add_system(network::info_widget.after(network::network_ui))
        // system sets
        .run();
}

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

#[autodefault]
fn setup_sys(mut commands: Commands, asset_server: Res<AssetServer>) {
    // spawn camera
    commands
        .spawn_bundle(Camera2dBundle {
            // ! fyi: magic number
            projection: OrthographicProjection { scale: 0.25 },
        })
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Middle],
        })
        .insert(MainCamera);
}
