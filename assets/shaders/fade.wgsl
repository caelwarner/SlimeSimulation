struct Context {
    pause: u32,
    fadeRate: f32,
    deltaTime: f32,
    hasTrails: u32,
}

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var<uniform> context: Context;

@compute @workgroup_size(8, 8, 1)
fn fade(@builtin(global_invocation_id) id: vec3<u32>) {
    if (context.pause == u32(1)) {
        return;
    }

    var fadeRate = 0.3;
    if (context.hasTrails == u32(1)) {
        fadeRate = context.fadeRate * context.deltaTime * 5.0;
    }

    let location = vec2<i32>(id.xy);
    let value = textureLoad(texture, location);
    let faded_value = max(vec4<f32>(0.0), value - vec4<f32>(0.0, 0.0, 0.0, fadeRate));

    storageBarrier();
    textureStore(texture, location, faded_value);
}
