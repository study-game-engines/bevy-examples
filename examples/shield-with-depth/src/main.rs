use bevy::{
    core_pipeline::{
        bloom::BloomSettings,
        prepass::{
            DepthPrepass, MotionVectorPrepass,
            NormalPrepass,
        },
    },
    pbr::{
        MaterialPipeline, MaterialPipelineKey,
        NotShadowCaster,
    },
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor,
            ShaderRef, ShaderType,
            SpecializedMeshPipelineError,
        },
    },
};
use bevy_asset_loader::prelude::*;
use std::f32::consts::FRAC_PI_2;

use bevy_basic_camera::{
    CameraController, CameraControllerPlugin,
};
use bevy_shader_utils::ShaderUtilsPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(ClearColor(
            Color::hex("071f3c").unwrap(),
        ))
        .insert_resource(AmbientLight {
            color: Color::ORANGE_RED,
            brightness: 0.02,
        })
        .add_state::<MyStates>()
        .add_plugins((
            DefaultPlugins,
            CameraControllerPlugin,
            ShaderUtilsPlugin,
            MaterialPlugin::<CustomMaterial> {
                prepass_enabled: false,
                ..default()
            },
            MaterialPlugin::<PrepassOutputMaterial> {
                // This material only needs to read the prepass textures,
                // but the meshes using it should not contribute to the prepass render, so we can disable it.
                prepass_enabled: false,
                ..default()
            },
        ))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next),
        )
        .add_collection_to_loading_state::<_, GlbAssets>(
            MyStates::AssetLoading,
        )
        .add_systems(OnEnter(MyStates::Next), setup)
        .add_systems(
            Update,
            toggle_prepass_view
                .run_if(in_state(MyStates::Next)),
        )
        .run();
}

/// set up a simple 3D scene
fn setup(
    // assets: Res<GlbAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut std_materials: ResMut<Assets<StandardMaterial>>,
    mut depth_materials: ResMut<
        Assets<PrepassOutputMaterial>,
    >,
    asset_server: Res<AssetServer>,
) {
    // Debug
    // A quad that shows the outputs of the prepass
    // To make it easy, we just draw a big quad right in front of the camera. For a real application, this isn't ideal.
    commands.spawn((
        MaterialMeshBundle {
            mesh: meshes.add(
                shape::Quad::new(Vec2::new(20.0, 20.0))
                    .into(),
            ),
            material: depth_materials.add(
                PrepassOutputMaterial {
                    settings: ShowPrepassSettings::default(
                    ),
                },
            ),
            transform: Transform::from_xyz(
                -0.75, 1.25, 3.0,
            )
            .looking_at(
                Vec3::new(2.0, -2.5, -5.0),
                Vec3::Y,
            ),
            ..default()
        },
        NotShadowCaster,
    ));
    // end Debug
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 3., 5.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        // To enable the prepass you need to add the components associated with the ones you need
        // This will write the depth buffer to a texture that you can use in the main pass
        DepthPrepass,
        // This will generate a texture containing world normals (with normal maps applied)
        NormalPrepass,
        MotionVectorPrepass,
        CameraController {
            orbit_mode: true,
            orbit_focus: Vec3::new(0.0, 0.5, 0.0).into(),
            ..default()
        },
        BloomSettings::default(),
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 8.0, 4.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes
            .add(shape::Plane::from_size(10.0).into()),
        material: std_materials
            .add(Color::rgb(1.0, 1.0, 1.0).into()),
        transform: Transform::from_xyz(0.0, 0.3, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes
            .add(shape::Plane::from_size(5.0).into()),
        material: std_materials
            .add(Color::rgb(1.0, 1.0, 1.0).into()),
        transform: Transform::from_xyz(0.3, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_z(
                FRAC_PI_2,
            )),

        ..default()
    });

    let transform = Transform::from_xyz(0.0, 0.5, 0.0);
    // cube
    commands.spawn((
        MaterialMeshBundle {
            // mesh: assets.hex_sphere.clone(),
            mesh: asset_server.load("models/hex-sphere-5-subdivisions.glb#Mesh0/Primitive0"),
            transform,
            material: materials.add(CustomMaterial {
                color: Color::BLUE,
                alpha_mode: AlphaMode::Blend,
            }),
            ..default()
        },
        NotShadowCaster,
    ));

    commands.spawn(SceneBundle {
        // scene: assets.ferris.clone(),
        scene: asset_server
            .load("models/ferris3d_v1.0.glb#Scene0"),
        transform: transform.clone().with_rotation(
            Quat::from_rotation_y(-FRAC_PI_2),
        ),
        ..default()
    });
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/custom_material.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // descriptor.primitive.cull_mode = None;
        if let Some(label) = &mut descriptor.label {
            *label = format!("shield_{}", *label).into();
        }
        descriptor.primitive.cull_mode = None;

        Ok(())
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(0)]
    color: Color,
    alpha_mode: AlphaMode,
}

#[derive(Resource, AssetCollection)]
struct GlbAssets {
    // #[asset(
    //     path = "models/hex-sphere-5-subdivisions.glb#Mesh0/Primitive0"
    // )]
    // hex_sphere: Handle<Mesh>,
    // #[asset(path = "models/ferris3d_v1.0.glb#Scene0")]
    // ferris: Handle<Scene>,
}

#[derive(
    Default, Clone, Eq, PartialEq, Debug, Hash, States,
)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

// Debug

#[derive(Debug, Clone, Default, ShaderType)]
struct ShowPrepassSettings {
    show_depth: u32,
    show_normals: u32,
    show_motion_vectors: u32,
    padding_1: u32,
    padding_2: u32,
}

// This shader simply loads the prepass texture and outputs it directly
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct PrepassOutputMaterial {
    #[uniform(0)]
    settings: ShowPrepassSettings,
}

impl Material for PrepassOutputMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/show_prepass.wgsl".into()
    }

    // This needs to be transparent in order to show the scene behind the mesh
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Every time you press space, it will cycle between transparent, depth and normals view
fn toggle_prepass_view(
    keycode: Res<Input<KeyCode>>,
    material_handle: Query<&Handle<PrepassOutputMaterial>>,
    mut materials: ResMut<Assets<PrepassOutputMaterial>>,
) {
    if keycode.just_pressed(KeyCode::Space) {
        let handle = material_handle.single();
        let mat = materials.get_mut(handle).unwrap();
        if mat.settings.show_depth == 1 {
            dbg!("normal");
            mat.settings.show_depth = 0;
            mat.settings.show_normals = 1;
        } else if mat.settings.show_normals == 1 {
            dbg!("transparent");
            mat.settings.show_depth = 0;
            mat.settings.show_normals = 0;
        } else {
            dbg!("depth");
            mat.settings.show_depth = 1;
            mat.settings.show_normals = 0;
        }

        // let mut text = text.single_mut();
        // text.sections[0].value =
        //     format!("Prepass Output: {out_text}\n");
    }
}
