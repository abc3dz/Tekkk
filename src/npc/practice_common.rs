use bevy::prelude::*;

use crate::combat::CombatTarget;
use crate::components::{
    EnemyHealthBar,
    EnemyHealthBarFill,
    Health,
    Player,
};

const ENEMY_HEALTH_BAR_WIDTH: f32 = 76.0;
const ENEMY_HEALTH_BAR_SHOW_DISTANCE: f32 = 10.0;

pub fn spawn_enemy_health_bar(
    commands: &mut Commands,
    target: Entity,
) {
    commands.spawn((
            EnemyHealthBar { target },
            Node {
                position_type: PositionType::Absolute,
                width: Val::Px(ENEMY_HEALTH_BAR_WIDTH),
                height: Val::Px(10.0),
                padding: UiRect::all(Val::Px(2.0)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(
                Color::srgb(0.02, 0.02, 0.02),
            ),
        ))
        .with_child((
            EnemyHealthBarFill,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(
                Color::srgb(0.9, 0.02, 0.04),
            ),
        ));
}

pub fn update_enemy_health_bars(
    mut commands: Commands,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    target_query: Query<(&GlobalTransform, &Health), With<CombatTarget>>,
    mut bar_query: Query<(Entity, &EnemyHealthBar, &Children, &mut Node), (With<EnemyHealthBar>, Without<EnemyHealthBarFill>)>,
    mut fill_query: Query<&mut Node, (With<EnemyHealthBarFill>, Without<EnemyHealthBar>)>,
) {
    let Ok((camera, camera_transform)) = camera_query.single()
    else { return };

    let Ok(player_transform) = player_query.single()
    else { return };

    let player_position = player_transform.translation();

    for (bar_entity, health_bar, children, mut bar_node) in &mut bar_query{
        let Ok((target_transform, health)) = target_query.get(health_bar.target)
        else {
            commands.entity(bar_entity).despawn();
            continue;
        };
        let target_position = target_transform.translation();
        let offset = target_position - player_position;
        let distance = Vec2::new(offset.x,offset.z,).length();

        if distance > ENEMY_HEALTH_BAR_SHOW_DISTANCE {
            bar_node.display = Display::None;
            continue;
        }

        let world_position = target_position + Vec3::Y * 2.0;
        let Ok(screen_position) = camera.world_to_viewport(camera_transform, world_position)
        else {
            bar_node.display = Display::None;
            continue;
        };

        bar_node.display = Display::Flex;
        bar_node.left = Val::Px(screen_position.x - ENEMY_HEALTH_BAR_WIDTH * 0.5,);
        bar_node.top = Val::Px(screen_position.y - 5.0,);
        let hp_ratio = health.current as f32 / health.max.max(1) as f32;
        let hp_percent = hp_ratio.clamp(0.0, 1.0) * 100.0;
        for child in children.iter() {
            if let Ok(mut fill_node) = fill_query.get_mut(child) {
                fill_node.width =
                    Val::Percent(hp_percent);
            }
        }
    }
}

pub struct PracticeCommonPlugin;

impl Plugin for PracticeCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_enemy_health_bars,
        );
    }
}