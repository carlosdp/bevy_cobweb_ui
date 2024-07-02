//! Demonstrates building a simple counter as a small reactive scene.

use bevy::prelude::*;
use bevy::window::WindowTheme;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_cobweb_ui::sickle::ui_builder::*;
use bevy_cobweb_ui::sickle::SickleUiPlugin;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactComponent, Deref, Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Counter(usize);

impl Counter
{
    fn increment(&mut self)
    {
        self.0 += 1;
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: ResMut<SceneLoader>)
{
    let scene = LoadableRef::new("examples/counter.load.json", "root");

    c.ui_builder(UiRoot).load_scene(&mut s, scene, |l| {
        l.edit("button", |l| {
            let button_id = l.id();
            l.on_pressed(move |mut c: Commands, mut counters: ReactiveMut<Counter>| {
                counters.get_mut(&mut c, button_id).map(Counter::increment);
            });

            l.edit("text", |l| {
                l.update_on(entity_mutation::<Counter>(button_id), |text_id| {
                    move |mut e: TextEditor, counters: Reactive<Counter>| {
                        let Some(counter) = counters.get(button_id) else { return };
                        e.write(text_id, |t| write!(t, "Counter: {}", **counter));
                    }
                });
            });
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn setup(mut commands: Commands)
{
    commands.spawn(Camera2dBundle {
        transform: Transform { translation: Vec3 { x: 0., y: 0., z: 1000. }, ..default() },
        ..default()
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn main()
{
    App::new()
        .add_plugins(bevy::DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window { window_theme: Some(WindowTheme::Dark), ..Default::default() }),
            ..Default::default()
        }))
        .add_plugins(SickleUiPlugin)
        .add_plugins(CobwebUiPlugin)
        .load("examples/counter.load.json")
        .register_type::<Counter>()
        .register_reactive_loadable::<Counter>()
        .add_systems(PreStartup, setup)
        .add_systems(OnEnter(LoadState::Done), build_ui)
        .run();
}

//-------------------------------------------------------------------------------------------------------------------
