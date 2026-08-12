use bevy::prelude::*;

use crate::combat::*;
use crate::components::*;

#[derive(Component, Debug)]
pub struct ElementDrop {
    pub element: Element,
    pub amount: u32,
}

#[derive(Component, Debug)]
pub struct ElementDropIcon {
    pub base_y: f32,
    pub phase: f32,
}

#[derive(Resource)]
pub struct ElementDropAssets {
    pub mesh: Handle<Mesh>,

    pub earth: Handle<StandardMaterial>,
    pub fire: Handle<StandardMaterial>,
    pub water: Handle<StandardMaterial>,
    pub wind: Handle<StandardMaterial>,
    pub inw: Handle<StandardMaterial>,
}

pub struct ElementDropPlugin;

impl Plugin for ElementDropPlugin {
    fn build(&self, app: &mut App) {app
        .add_systems(Startup,setup_element_drop_assets,)
        .add_systems(Update,
            (
                        animate_element_drop_icons,
                        collect_element_drops,
                    )
        );
    }
}

pub fn setup_element_drop_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(0.6, 0.6));
    let earth = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load(
                "textures/ElementalEarth.png"
            ),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let fire = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load(
                "textures/ElementalFire.png"
            ),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let water = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load(
                "textures/ElementalWater.png"
            ),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let wind = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load(
                "textures/ElementalWind.png"
            ),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    let inw = materials.add(StandardMaterial {
        base_color_texture: Some(
            asset_server.load(
                "textures/ElementalInw.png"
            ),
        ),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        cull_mode: None,
        ..default()
    });
    
    commands.insert_resource(
        ElementDropAssets {
            mesh,
            earth,
            fire,
            water,
            wind,
            inw,
        },
    );
}

pub fn spawn_element_drop(
    commands: &mut Commands,
    assets: &ElementDropAssets,
    position: Vec3,
    element: Element,
    amount: u32,
) {
    if amount == 0 {
        return;
    }

    let (
        material,
        light_color,
        phase,
    ) = match element {
        Element::Earth => (
            assets.earth.clone(),
            Color::srgb(0.45, 0.22, 0.08),
            0.0,
        ),

        Element::Fire => (
            assets.fire.clone(),
            Color::srgb(1.0, 0.20, 0.05),
            1.0,
        ),

        Element::Water => (
            assets.water.clone(),
            Color::srgb(0.15, 0.65, 1.0),
            2.0,
        ),

        Element::Wind => (
            assets.wind.clone(),
            Color::srgb(0.65, 0.25, 1.0),
            3.0,
        ),

        Element::Inw => (
            assets.inw.clone(),
            Color::srgb(1.0, 0.75, 0.10),
            4.0,
        ),

        Element::Neutral => {
            return;
        }
    };

    commands
        .spawn((
            Name::new("Element Drop"),

            ElementDrop {
                element,
                amount,
            },

            Transform::from_translation(position),
        ))
        .with_children(|parent| {

            // ==========================
            // Icon
            // ==========================
            parent.spawn((
                Name::new("Element Drop Icon"),

                ElementDropIcon {
                    base_y: 0.70,
                    phase,
                },

                Mesh3d(
                    assets.mesh.clone()
                ),

                MeshMaterial3d(
                    material
                ),

                Transform::from_xyz(
                    0.0,
                    0.70,
                    0.0,
                ),
            ));

            // ==========================
            // แสงสีตามธาตุ
            // ==========================
            parent.spawn((
                Name::new("Element Drop Light"),

                PointLight {
                    color: light_color,

                    intensity: 120.0,

                    range: 1.4,

                    shadows_enabled: false,

                    ..default()
                },

                Transform::from_xyz(
                    0.0,
                    0.35,
                    0.0,
                ),
            ));
        });
}

pub fn spawn_element_reward_drops(
    commands: &mut Commands,
    assets: &ElementDropAssets,
    reward: &ElementExpReward,
    position: Vec3,
    rng: &mut impl rand::Rng,
) {
    let drops = [
        (
            Element::Water,
            reward.water.roll(rng),
            Vec3::new(-0.6, 0.0, 0.0),
        ),
        (
            Element::Fire,
            reward.fire.roll(rng),
            Vec3::new(0.6, 0.0, 0.0),
        ),
        (
            Element::Wind,
            reward.wind.roll(rng),
            Vec3::new(0.0, 0.0, -0.6),
        ),
        (
            Element::Earth,
            reward.earth.roll(rng),
            Vec3::new(0.0, 0.0, 0.6),
        ),
        (
            Element::Inw,
            reward.inw.roll(rng),
            Vec3::new(0.0, 0.0, 0.0),
        ),
    ];

    for (
        element,
        amount,
        offset,
    ) in drops
    {
        if amount == 0 {
            continue;
        }

        spawn_element_drop(
            commands,
            assets,
            position + offset,
            element,
            amount,
        );
    }
}

pub fn collect_element_drops(
    mut commands: Commands,

    mut player_query: Query<
        (
            &GlobalTransform,
            &mut ElementMastery,
        ),
        With<Player>,
    >,

    drop_query: Query<
        (
            Entity,
            &GlobalTransform,
            &ElementDrop,
        ),
        Without<Player>,
    >,
) {
    let Ok((
        player_transform,
        mut mastery,
    )) = player_query.single_mut()
    else {
        return;
    };

    let player_position =
        player_transform.translation();

    for (
        drop_entity,
        drop_transform,
        drop,
    ) in &drop_query
    {
        let distance =
            player_position.distance(
                drop_transform.translation()
            );

        if distance > 0.8 {
            continue;
        }

        if let Some(progress) =
            mastery.get_mut(drop.element)
        {
            progress.exp =
                progress.exp
                    .saturating_add(
                        drop.amount
                    );

            info!(
                "Picked up {:?} +{}",
                drop.element,
                drop.amount,
            );
        }

        commands
            .entity(drop_entity)
            .despawn();
    }
}

pub fn animate_element_drop_icons(
    time: Res<Time>,

    camera_query:
        Query<
            &GlobalTransform,
            With<Camera3d>,
        >,

    mut icon_query:
        Query<
            (
                &mut Transform,
                &GlobalTransform,
                &ElementDropIcon,
            ),
        >,
) {
    let Ok(camera_transform) =
        camera_query.single()
    else {
        return;
    };

    let camera_position =
        camera_transform.translation();

    let elapsed =
        time.elapsed_secs();

    for (
        mut transform,
        global_transform,
        icon,
    ) in &mut icon_query
    {
        // ==========================
        // ลอยขึ้นลง
        // ==========================

        let floating =
            (
                elapsed * 2.5
                    + icon.phase
            )
            .sin()
                * 0.10;

        transform.translation.y =
            icon.base_y
                + floating;

        // ==========================
        // Billboard
        // หันเฉพาะแกน Y
        // ==========================

        let icon_position =
            global_transform.translation();

        let to_camera =
            camera_position
                - icon_position;

        let flat_direction =
            Vec3::new(
                to_camera.x,
                0.0,
                to_camera.z,
            );

        if flat_direction.length_squared()
            > 0.0001
        {
            let direction =
                flat_direction.normalize();

            transform.rotation =
                Quat::from_rotation_y(
                    direction
                        .x
                        .atan2(
                            direction.z
                        ),
                );
        }
    }
}