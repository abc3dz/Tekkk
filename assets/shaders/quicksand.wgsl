#import bevy_pbr::forward_io::VertexOutput

struct QuicksandMaterial {
    color_light: vec4<f32>,
    color_dark: vec4<f32>,
    parameters: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: QuicksandMaterial;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;

    let time = material.parameters.x;
    let speed = material.parameters.y;
    let scale = material.parameters.z;
    let edge_darkness = material.parameters.w;

    // ทำให้ลายทรายเคลื่อนไปเรื่อย ๆ
    let moving_uv = vec2<f32>(
        uv.x * scale + time * speed,
        uv.y * scale - time * speed * 0.55
    );

    // คลื่นทรายสองทิศทางซ้อนกัน
    let wave_1 = sin(moving_uv.x * 6.0 + sin(moving_uv.y * 2.5));
    let wave_2 = sin(
        moving_uv.y * 7.0
        - time * speed * 2.0
        + sin(moving_uv.x * 3.0)
    );

    // ลายหมุนวนบริเวณกลางพื้น
    let centered_uv = uv - vec2<f32>(0.5, 0.5);
    let distance_from_center = length(centered_uv);
    let angle = atan2(centered_uv.y, centered_uv.x);

    let swirl = sin(
        angle * 4.0
        + distance_from_center * 35.0
        - time * speed * 5.0
    );

    // รวมคลื่นเข้าด้วยกัน
    let pattern = (
        wave_1 * 0.35
        + wave_2 * 0.25
        + swirl * 0.40
    );

    // แปลงค่าจาก -1..1 ให้เป็น 0..1
    let mix_amount = clamp(pattern * 0.5 + 0.5, 0.0, 1.0);

    var final_color = mix(
        material.color_dark,
        material.color_light,
        mix_amount
    );

    // ทำขอบให้เข้มกว่าตรงกลางเล็กน้อย
    let edge = smoothstep(0.25, 0.70, distance_from_center);
    final_color = vec4<f32>(final_color.rgb * (1.0 - edge * edge_darkness),final_color.a);

    return final_color;
}