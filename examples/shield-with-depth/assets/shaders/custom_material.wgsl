#import bevy_shader_utils::simplex_noise_3d simplex_noise_3d
#import bevy_pbr::mesh_view_bindings globals
#import bevy_pbr::mesh_view_bindings as view_bindings
#import bevy_pbr::mesh_bindings mesh
#import bevy_pbr::mesh_functions as mesh_functions
#import bevy_pbr::mesh_vertex_output MeshVertexOutput
#import bevy_shader_utils::fresnel fresnel

// NOTE: Bindings must come before functions that use them!
#import bevy_pbr::prepass_utils

struct Vertex {
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(5) joint_indices: vec4<u32>,
    @location(6) joint_weights: vec4<f32>,
#endif
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    #ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
    #endif
    #ifdef VERTEX_TANGENTS
    @location(3) world_tangent: vec4<f32>,
    #endif
    #ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
    #endif
    @location(5) position_diff: f32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let thickness = 5.0;
    // higher is shorter
    let how_long_to_stay_in_opposite_state = 30.0;
    let frequency = 2.0;
    // let position_diff = pow(sin(2.0 * material.time), 1.0);
    let position_diff = 1.0 - pow(thickness * sin(frequency * globals.time + vertex.position.y + vertex.position.z), how_long_to_stay_in_opposite_state);
    // let smooth_diff = smoothstep(0.0, 1.0, position_diff);
    let position = (vertex.normal * (smoothstep(0.0, 1.0, position_diff)) * 0.04) + vertex.position;

    var out: VertexOutput;
    out.position_diff = position_diff;

#ifdef SKINNED
    var model = skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    var model = mesh.model;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = bevy_pbr::skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
#endif
#endif

#ifdef VERTEX_POSITIONS
    // out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(position, 1.0));
    out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(model, vertex.tangent);
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif


    return out;
}

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@fragment
fn fragment(
    @builtin(front_facing) is_front: bool,
    mesh: MeshVertexOutput,
    @location(5) position_diff: f32,
) -> @location(0) vec4<f32> {
    // return color;
    var noise = simplex_noise_3d(vec3<f32>(mesh.world_normal.xy * 4.2, globals.time));
    var alpha = (noise + 1.0) / 2.0;

    let highlight = smoothstep(0.0, 1.0, position_diff + 1.0);

    let fresnel = fresnel(view_bindings::view.world_position.xyz, mesh.world_position.xyz, mesh.world_normal, 2.0, 1.0);

    let offset = 0.82;
    let intersection_intensity = 10.0;
    let sample_index = 0u;

    let depth = bevy_pbr::prepass_utils::prepass_depth(mesh.position, sample_index);

    // thanks to https://github.com/IceSentry for this line in particular,
    // which I was having trouble landing on
    var intersection = 1.0 - ((mesh.position.z - depth) * 100.0) - offset;
    intersection = smoothstep(0.0, 1.0, intersection);
    if is_front {
        intersection *= intersection_intensity;
    } else {
        intersection *= intersection_intensity / 2.0;
    }

    let color = mix(vec3(1.00, 0.455, 0.827), vec3(1.00, 0.555, 0.927), highlight) * (alpha + 0.5) * 5.0;
    if is_front {
        return vec4(color * (10.0 * highlight + 1.0), fresnel * 0.4 + intersection + highlight * 0.003);
    } else {
        return vec4(color, intersection);
    }
}
