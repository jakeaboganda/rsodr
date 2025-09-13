// Disable the standard console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use std::f32::consts::PI;
use bevy::math::primitives::Rectangle;

// This is the main function where the Bevy application starts.
fn main() {
    // A Bevy app is created and configured with the `DefaultPlugins`.
    App::new()
        // Add Bevy's default plugins, which provide functionality for rendering,
        // input, UI, and more.
        .add_plugins(DefaultPlugins)
        // Add a system that will be run once at the start of the application.
        .add_systems(Startup, setup)
        // Add a system to handle camera movement and interaction.
        .add_systems(Update, (camera_input, camera_orbit).chain())
        // Run the app.
        .run();
}

// A struct to hold the data for a single segment of the road.
// This mirrors the information you described from your library API.
#[derive(Debug, Clone)]
struct RoadSegment {
    start_pos: Vec3,
    end_pos: Vec3,
    start_s: f32,
    end_s: f32,
    width: f32,
    left_side: Vec<Vec3>,
    right_side: Vec<Vec3>,
    road_id: u32,
    lane_id: u32,
    lane_section_id: u32,
}

// Generates some dummy road data for visualization.
// In a real application, this would be replaced with calls to your library's API.
fn generate_road_data() -> Vec<RoadSegment> {
    // We'll create a simple straight road for demonstration purposes.
    let segment = RoadSegment {
        start_pos: Vec3::new(0.0, 0.0, 0.0),
        end_pos: Vec3::new(100.0, 0.0, 0.0),
        start_s: 0.0,
        end_s: 100.0,
        width: 4.0,
        // For a straight road, the left and right sides are simple offsets.
        left_side: vec![Vec3::new(0.0, 0.0, 2.0), Vec3::new(100.0, 0.0, 2.0)],
        right_side: vec![Vec3::new(0.0, 0.0, -2.0), Vec3::new(100.0, 0.0, -2.0)],
        road_id: 1,
        lane_id: 1,
        lane_section_id: 1,
    };

    // Create a second segment at an angle.
    let segment_2 = RoadSegment {
        start_pos: Vec3::new(100.0, 0.0, 0.0),
        end_pos: Vec3::new(150.0, 0.0, 50.0),
        start_s: 100.0,
        end_s: 150.0 + 50.0, // using pythagoras theorem to calculate longitudinal length
        width: 4.0,
        left_side: vec![Vec3::new(100.0, 0.0, 2.0), Vec3::new(150.0, 0.0, 52.0)],
        right_side: vec![Vec3::new(100.0, 0.0, -2.0), Vec3::new(150.0, 0.0, 48.0)],
        road_id: 1,
        lane_id: 1,
        lane_section_id: 2,
    };

    vec![segment, segment_2]
}

// Spawns the 3D entities for the road network.
fn spawn_roads(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let road_data = generate_road_data();

    for segment in road_data {
        // Create a custom mesh for the road segment.
        let mesh = Mesh::from(Rectangle::new(
            segment.end_pos.distance(segment.start_pos),
            segment.width,
        ));

        // Calculate the direction and rotation of the road segment.
        let direction = (segment.end_pos - segment.start_pos).normalize();
        let rotation = Quat::from_rotation_y(direction.z.atan2(direction.x));

        // Calculate the center position of the road segment.
        let position = (segment.start_pos + segment.end_pos) / 2.0;

        // Spawn a PbrBundle to represent the road segment in 3D.
        commands.spawn(PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial::from(Color::rgb(0.2, 0.2, 0.2))),
            transform: Transform::from_translation(position).with_rotation(rotation),
            ..default()
        });
    }
}

// A component to mark the main camera.
#[derive(Component)]
struct MainCamera;

// A component to hold the camera's state for orbiting.
#[derive(Component)]
struct CameraOrbit {
    center: Vec3,
    distance: f32,
    azimuth: f32, // Horizontal angle in radians.
    elevation: f32, // Vertical angle in radians.
    pan: Vec2, // For panning the camera.
}

// A system to set up the scene: camera, light, and roads.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a directional light source to illuminate the scene.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(10.0, 10.0, 10.0),
            rotation: Quat::from_rotation_x(PI / 4.0)
                .mul_quat(Quat::from_rotation_y(PI / 4.0)),
            ..default()
        },
        ..default()
    });

    // Spawn the roads.
    spawn_roads(commands.reborrow(), meshes, materials);

    // Spawn the camera with its custom components.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-100.0, 100.0, 150.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
        CameraOrbit {
            center: Vec3::ZERO,
            distance: 200.0,
            azimuth: -PI / 4.0,
            elevation: PI / 4.0,
            pan: Vec2::ZERO,
        },
    ));
}

// A system to handle mouse input for the camera.
fn camera_input(
    mut query: Query<&mut CameraOrbit, With<MainCamera>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut cursor_moved: EventReader<CursorMoved>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut last_cursor_position: Local<Option<Vec2>>,
) {
    let mut orbit = query.single_mut();

    // Zoom with the mouse wheel.
    for event in mouse_wheel.read() {
        let zoom_factor = 1.0 + event.y * -0.1;
        orbit.distance = (orbit.distance * zoom_factor).clamp(5.0, 500.0);
    }

    // Handle rotation and panning with mouse buttons.
    let mut current_cursor_position = None;
    for event in cursor_moved.read() {
        current_cursor_position = Some(event.position);
    }

    if let (Some(current_pos), Some(last_pos)) = (*last_cursor_position, current_cursor_position) {
        let delta = current_pos - last_pos;

        // Pan with the middle mouse button.
        if mouse_buttons.pressed(MouseButton::Middle) {
            orbit.pan += delta * 0.1;
        }

        // Orbit with the left mouse button.
        if mouse_buttons.pressed(MouseButton::Left) {
            orbit.azimuth -= delta.x * 0.005;
            orbit.elevation = (orbit.elevation + delta.y * 0.005).clamp(-PI / 2.0, PI / 2.0);
        }
    }
    *last_cursor_position = current_cursor_position;
}

// A system to update the camera's position based on its orbit state.
fn camera_orbit(mut query: Query<(&mut Transform, &CameraOrbit), With<MainCamera>>) {
    let (mut transform, orbit) = query.single_mut();

    let rotation = Quat::from_axis_angle(Vec3::Y, orbit.azimuth)
        * Quat::from_axis_angle(Vec3::X, orbit.elevation);

    let new_pos = rotation * Vec3::new(0.0, 0.0, orbit.distance) + orbit.center;
    
    // Apply panning to the center point.
    let pan_transform = Transform::from_translation(Vec3::new(orbit.pan.x, orbit.pan.y, 0.0));
    let final_pos = new_pos + pan_transform.translation;

    *transform = Transform::from_translation(final_pos).looking_at(orbit.center, Vec3::Y);
}
