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

    // ให้ Bevy คำนวณแสงจริงจาก light ทุกดวงในฉาก (ทิศทาง, สี, ความเข้ม)
    // รวมถึง sample shadow map ให้ครบตามปกติก่อน แล้วค่อยเอาความสว่าง
    // ที่ได้ (ซึ่งมีเงาจริงติดมาด้วยแล้ว) มาบีบเป็นแถบสีแบบ cel shading
    let lit_color = apply_pbr_lighting(pbr_input);
    let luminance = dot(
        lit_color.rgb,
        vec3<f32>(0.2126, 0.7152, 0.0722),
    );

    // แบ่งแสงแข็ง ๆ เป็น 3 ระดับ ตามค่าที่ตั้งมาจากฝั่ง Rust
    var shade = toon_material.shadow_brightness;

    if luminance > toon_material.shadow_cutoff {
        shade = toon_material.mid_brightness;
    }

    if luminance > toon_material.mid_cutoff {
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