#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{
        alpha_discard,
        apply_pbr_lighting,
        main_pass_post_lighting_processing,
    },
}

struct ToonExtension {
    shadow_cutoff: f32,
    mid_cutoff: f32,
    shadow_brightness: f32,
    mid_brightness: f32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> toon_material: ToonExtension;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input =
        pbr_input_from_standard_material(in, is_front);

    pbr_input.material.base_color = alpha_discard(
        pbr_input.material,
        pbr_input.material.base_color,
    );

    let normal = normalize(pbr_input.N);

    // ทิศทางแสงปลอมสำหรับทดสอบ Cel Shader
    let light_direction =
        normalize(vec3<f32>(-0.5, 0.8, 0.3));

    let light = max(
        dot(normal, light_direction),
        0.0,
    );

    // แบ่งแสงแข็ง ๆ เป็น 3 ระดับ
    var shade = 0.25;

    if light > 0.4 {
        shade = 0.6;
    }

    if light > 0.75 {
        shade = 1.0;
    }

    var out: FragmentOutput;

    out.color = vec4<f32>(
        pbr_input.material.base_color.rgb * shade,
        pbr_input.material.base_color.a,
    );

    out.color = main_pass_post_lighting_processing(
        pbr_input,
        out.color,
    );

    return out;
}