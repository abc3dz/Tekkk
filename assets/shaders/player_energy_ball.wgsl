#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // สีส้มมืด
    let dark_orange = vec3<f32>(
        0.16,
        0.025,
        0.002,
    );

    // สีส้มสว่าง
    let bright_orange = vec3<f32>(
        1.0,
        0.32,
        0.015,
    );

    // จำนวนแถบสีรอบ Sphere
    let stripe_count = 6.0;

    // ความเร็วที่สีไหลจากหน้าไปหลัง
    let scroll_speed = 1.8;

    let phase =
        in.uv.x * stripe_count
        - globals.time * scroll_speed;

    let wave = sin(
        phase * 6.2831853,
    );

    let color_blend = smoothstep(
        -0.25,
        0.25,
        wave,
    );

    let pulse =
        0.90
        + sin(globals.time * 9.0)
            * 0.10;

    let final_color = mix(
        dark_orange,
        bright_orange,
        color_blend,
    ) * pulse;

    return vec4<f32>(
        final_color,
        1.0,
    );
}