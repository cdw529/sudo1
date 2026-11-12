///
///  //////////////////////////////////////////////////////////////////////////////////////////////
///  ////////////////////   Startup_Splash_Screen with Bevy_logo & text   /////////////////////////
///  //////////////////////////////////////////////////////////////////////////////////////////////
///  //////////////////////////////   Exits to Main Menu    ///////////////////////////////////////
///  //////////////////////////////////////////////////////////////////////////////////////////////

// use bevy::asset::*;
use bevy::prelude::*;

use crate::sudoku_constants::{ SPLASH_TEXT_1, SPLASH_TEXT_2, TEXT_BRIGHT_BLUE_2 };
use crate::states::GameState;
use crate::board_chooser::despawn_screen;


pub struct SplashPlugin;

impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SplashTimers {
            timer0: Timer::from_seconds(1.5, TimerMode::Once),
            timer1: Timer::from_seconds(1.5, TimerMode::Once),
            timer2: Timer::from_seconds(2.0, TimerMode::Once),
        })
        //
        .add_systems(OnEnter(GameState::Splash0), splash_setup) // Spawn Splash Screens,
        .add_systems(OnEnter(GameState::Splash1), splash_text_1)
        .add_systems(OnEnter(GameState::Splash2), splash_text_2)
        // Timer_2 Ends: Switch to Main Menu State (Pause Menu Screen)
        .add_systems(
            Update,
            countdown.run_if(
                in_state(GameState::Splash0)
                    .or(in_state(GameState::Splash1))
                    .or(in_state(GameState::Splash2)),
            ),
        )
        .add_systems(OnExit(GameState::Splash2), despawn_screen::<OnSplashScreen>);
        // Timer_2 Ends: Despawn all Splash Screen entities
    }
}


// Tag component used to group entities on spawn for subsequent despawn of the splash screen components
#[derive(Component)]
pub struct OnSplashScreen;

/* Tag component used to tag entities added on the main menu screen
#[derive(Component)]
struct OnPauseMenuScreen;
*/


// A single resource to hold all timers for the splash screen sequence.
#[derive(Resource)]
struct SplashTimers {
    timer0: Timer,
    timer1: Timer,
    timer2: Timer,
}

// Countdown system for all splash_screen states
fn countdown(
    time: Res<Time>,
    mut timers: ResMut<SplashTimers>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) -> () {
    match current_state.get() {
        GameState::Splash0 => {
            if timers.timer0.tick(time.delta()).just_finished() {
                eprintln!("Timer0 finished, setting to Splash1");
                next_state.set(GameState::Splash1);
            }
        }
        GameState::Splash1 => {
            if timers.timer1.tick(time.delta()).just_finished() {
                eprintln!("Timer1 finished, setting to Splash2");
                next_state.set(GameState::Splash2);
            }
        }
        GameState::Splash2 => {
            if timers.timer2.tick(time.delta()).just_finished() {
                eprintln!("Timer2 finished, setting to Menu");
                next_state.set(GameState::Menu); // call the pause_menu screen
            }
        }
        _ => {}
    }
}


fn splash_setup(mut commands: Commands, 
    assets: Res<AssetServer>,
) -> () {
    eprintln!("Running Splash Setup");
    let icon = assets.load("icons/blue_bevy_2048.png");
    let font_handle = assets.load("fonts/FiraCode-Bold.ttf");
    commands
        .spawn((
            Node {
                display: Display::Flex,
                top: Val::Px(65.0),   // 50.0, 65.0,  75.0
                left: Val::Px(-25.0), // -25.0
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            // BackgroundColor(Color::BLACK),
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(icon),
                Node {
                    width: Val::Px(375.0), // 575.0
                    ..default()
                },
                // BackgroundColor(BLACK.into()),  // BLACK
                TextLayout::new_with_justify(Justify::Center),
            ));
            parent.spawn((
                Text::new("Bevy"),
                TextFont {
                    font: font_handle,
                    font_size: 48.0, // 8.0
                    ..default()
                },
                TextColor(TEXT_BRIGHT_BLUE_2), // SPLASH_TEXT_1, WHITE
                Node {
                    margin: UiRect::top(Val::Px(200.0)),
                    ..default()
                },
            ));
        });
}
//

//
fn splash_text_1(mut commands: Commands, 
    assets: Res<AssetServer>
) -> () {
    let font_handle_fcode_b: Handle<Font> = assets.load("fonts/FiraCode-Bold.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(225.0), // 175.0
                left: Val::Px(2.),   // 2.0
                width: Val::Percent(100.),
                // height: Val::Percent(100.),  // Remove to stop overflow
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("| Sudoku |".to_string()),
                TextFont {
                    font: font_handle_fcode_b.clone(),
                    font_size: 80.0,
                    ..default()
                },
                TextColor(SPLASH_TEXT_1.into()),
            ));
        });
}
//

//
fn splash_text_2(
    mut commands: Commands, 
    assets: Res<AssetServer>
) -> () {
    // let font_handle_fcode_r: Handle<Font> = assets.load("fonts/FiraCode-Regular.ttf");
    let font_handle_fcode_b: Handle<Font> = assets.load("fonts/FiraCode-Bold.ttf");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(310.0), // 260.0
                left: Val::Px(4.0),  // 4.0
                width: Val::Percent(100.0),
                // height: Val::Percent(100.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            OnSplashScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Bevy Game Engine | RUST".to_string()),
                TextFont {
                    font: font_handle_fcode_b.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(SPLASH_TEXT_2.into()),
            ));
        });
}



// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/*
pub fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}
*/


/*
pub fn pause_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Common style for all buttons on the screen
    let font_handle_fcode_b: Handle<Font> = asset_server.load("fonts/FiraCode-Bold.ttf");
    let button_node = Node {
        width: px(300),  // 400
        height: px(85),  // 65
        margin: UiRect::all(px(10)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    // let button_icon_node = Node {  // Button Icon is not used, so delete this 'assets: Res<AssetServer>'
    let button_text_font = TextFont {
        font: font_handle_fcode_b,
        font_size: 33.0,
        ..default()
    };
    commands.spawn((
        DespawnOnExit(GameState::Menu),
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnPauseMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(BLACK.into()),  // CRIMSON
            children![
                // Display the game name
                (
                    Text::new("| PAUSE MENU |"),  // "SUDOKU PAUSE MENU" , "Bevy Game Menu UI"
                    TextFont {
                        font_size: 50.0,
                        ..default()
                    },
                    TextColor(LIGHT_BLUE.into()),  // LIGHT_BLUE, YELLOW
                    Node {
                        // margin: UiRect::all(px(80.)),  // 20, 50
                        margin: UiRect::bottom(px(320.)),
                        ..default()
                    },
                ),
                // Display three buttons for each action available from the main menu:
                // - new game
                // - settings
                // - quit
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                    children![
                        // (ImageNode::new(right_icon), button_icon_node.clone()),
                        (   Text::new("Resume"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Settings,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (   Text::new("New Game"),  // Go to Board Chooser
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::ButtonThree,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (   Text::new("Save Game"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),


                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::ButtonFour,
                    children![
                        // (ImageNode::new(wrench_icon), button_icon_node.clone()),
                        (   Text::new("Show Solution"),
                            button_text_font.clone(),
                            TextColor(TEXT_COLOR),
                        ),
                    ]
                ),

                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![
                        // (ImageNode::new(exit_icon), button_icon_node.clone()),
                        (   Text::new("Quit"),
                            button_text_font,
                            TextColor(TEXT_COLOR.into()),
                        ),
                    ]
                ),
            ]
        )],
    ));
}
*/

// next_state.set(GameState::SpawnBoard);

/*
#[derive(Resource, Component, Copy, Clone)]
pub struct SelectedOption;  // Tag component to indicate a selected button Resource for substate option selection


fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}
*/

/*
fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_writer: MessageWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<PauseState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_writer.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(PauseState::Game);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::SettingsDisplay => {
                    menu_state.set(MenuState::SettingsDisplay);
                }
                MenuButtonAction::SettingsSound => {
                    menu_state.set(MenuState::SettingsSound);
                }
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }
            }
        }
    }
}
*/
