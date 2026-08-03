#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // เปลี่ยน UV จากช่วง 0..1 เป็น -1..1
    let position = input.uv * 2.0 - vec2<f32>(1.0, 1.0);

    let radius = length(position);
    let angle = atan2(position.y, position.x);

    // จำนวนแขนของลายหมุน
    let arm_count = 6.0;

    // ความโค้งของลายจากกลางออกไปด้านนอก
    let twist = 18.0;

    // ความเร็วในการหมุน
    let rotation_speed = 3.0;

    // ลายก้นหอย
    let spiral =
        angle * arm_count
        + radius * twist
        - globals.time * rotation_speed;

    // ทำให้แบ่งเป็นแถบสีค่อนข้างชัด
    let stripe = smoothstep(
        -0.15,
        0.15,
        sin(spiral),
    );

    let blue = vec3<f32>(
        0.02,
        0.18,
        1.0,
    );

    let yellow = vec3<f32>(
        1.0,
        0.75,
        0.02,
    );

    // สลับสีน้ำเงินกับสีเหลือง
    var color = mix(blue, yellow, stripe);

    // ทำให้แสงเต้นเบา ๆ
    let pulse =
        0.90
        + sin(globals.time * 4.0 - radius * 10.0) * 0.10;

    // ทำตรงกลางให้สว่างกว่าขอบ
    let center_glow =
        1.0
        + (1.0 - radius) * 0.35;

    color *= pulse * center_glow;

    // ทำให้ขอบวงกลมค่อย ๆ โปร่งใส
    let edge_alpha =
        1.0 - smoothstep(0.88, 1.0, radius);

    return vec4<f32>(
        color,
        edge_alpha * 0.95,
    );
}