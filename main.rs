/////////////////////////////////   IMPORT EXTERNAL MODULES / CRATES   ///////////////////////////////////////////////
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy::window::*;
// use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
// use bevy::ecs::observer::*;
// use bevy_feathers::*;

// ////////////////////////////////////   RE-IMPORT LOCAL PATHS (crate)   ////////////////////////////////////////////
// use crate::constants::*;
use crate::board_chooser::*;
use crate::intro_splash::SplashPlugin;
// use crate::nav_board_orig::*;
use crate::nav_board::*;
use crate::pause::*;
use crate::setup_board::*;
// use crate::setup_board::SetupBoardPlugin;
// use crate::setup_game_board::SetupBoardPlugin;
// use crate::print_resources::PrintPlugin;
use crate::solver::SolverPlugin;
use crate::states::StatePlugin;
use crate::text_msgs::*;

//////////////////////////////////////   CREATE LOCAL MODULES (crate)   //////////////////////////////////////////////
// git remote add origin https://github.com/cdw529/sudo1.git
// git push -u origin main
pub mod constants;
pub mod board_chooser;
pub mod sudoku_constants;
pub mod intro_splash;
// pub mod nav_board_orig;
pub mod nav_board;
pub mod pause;
pub mod setup_board;
// pub mod setup_game_board;
pub mod solver;
pub mod states;
pub mod text_msgs;

#[derive(Component, Default, Debug, Clone)]
#[require(Camera2d)]
pub struct MainCamera;

fn main() {
    App::new()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(ClearColor(Color::srgba(0.1, 0.1, 0.2, 1.0)))
        .insert_resource(SelectedCellRes(0))
        .insert_resource(GlobalAmbientLight {
            color: Color::WHITE,
            brightness: 10000.0,
            affects_lightmapped_meshes: false,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                mode: WindowMode::Windowed,
                position: WindowPosition::Centered(MonitorSelection::Primary),
                // position: WindowPosition::At(IVec2::new(250, 0)),  // 750, 0
                // resolution: [WINDOW_WTH as u32, WINDOW_HT as u32].into(),  // Win 752 X 1014
                resolution: [766 as u32, 1016].into(),
                title: "Sudoku".to_string(),
                enabled_buttons: EnabledButtons {
                    minimize: true,
                    maximize: true,
                    close: true,
                },
                ..Default::default()
            }),
            ..Default::default()
        }))
        // .add_plugins(EguiPlugin::default())
        // .add_plugins(WorldInspectorPlugin::default())
        .add_plugins(BoardChoicePlugin)
        .add_plugins(SplashPlugin)
        .add_plugins(HoverSystemsPlugin)
        .add_plugins(PausePlugin)
        .add_plugins(SetupBoardPlugin)
        .add_plugins(SolverPlugin)
        .add_plugins(StatePlugin)
        .add_plugins(MessagesPlugin)
        // .add_plugins(FeathersPlugin)
        //
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    // commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.spawn((Camera2d::default(), IsDefaultUiCamera, MainCamera));
}

/*
fn position_window(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let window_width = window.resolution.width as f32;
        let x = ((monitor_size.x as f32 - window_width) / 2.0) as i32;
        let y = 0;
        window.position = WindowPosition::At(IVec2::new(x, y));
        //window.position = WindowPosition::At(IVec2::new(x, y));
    }
}
*/

/* 0.18  replace winit wakeup system with bevy event loop proxy, to trigger "WakeUp" event on app startup,
    for cursor position tracking setup in "MainCamera" component

fn wakeup_system(event_loop_proxy: Res<EventLoopProxyWrapper>) -> Result {
    event_loop_proxy.send_event(WinitUserEvent::WakeUp)?;
    Ok(())
}


pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
*/

/*
  Finalized target architecture (for a basic working Sudoku first)
  Use a 4-layer ECS flow:

 1  Input layer
      Mouse/keyboard/gamepad systems only detect input and emit intents

 2  Intent layer
      Domain events: SelectCell, SetDigit, ToggleTemp, ClearCell.

 3  Game logic layer
      Applies Sudoku rules, updates board resources, computes candidates.

 4  UI projection layer
      Renders board/messages/highlights from state/resources (no game-rule decisions).

  That’s the most idiomatic path for Bevy 0.18.x and great for learning Rust/Bevy cleanly.
*/
