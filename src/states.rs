// Set up sub_states for game states

use bevy::prelude::*;

#[derive(Resource, States, Reflect, Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Splash0,
    Splash1,
    Splash2,
    //
    Menu, // MainMenu
    //
    ChooseBoard,
    //
    Easy,
    Medium,
    Hard,
    Extreme,
    //
    SolveBoard,
    // ShowBoard,
    ShowTemps,  // show temps for selected cell while tab key held down, remove when tab key released
    // #[default]
    SpawnBoard,
    // SpawnCells,
    NavBoard,
    NoUpdate,
    UpdateCell,
    EditTemps,
    ReturnToTemps,
    EditPerms0,
    EditPerms1,
    // PauseWindow,
    PrintBoard,
}

#[derive()]
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
    }
}

/*
// Substates for Pause submenu
#[derive(SubStates, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::Menu)]
#[states(scoped_entities)]
pub enum MenuSubState {
    #[default]
    Main,
    ChooseBoard,
    Players,
    Settings,
    PauseMenu,
}
*/
