struct Context {
    color: vec4<f32>,
}

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@group(0) @binding(1)
var<uniform> context: Context;

@compute @workgroup_size(8, 8, 1)
fn recolor(@builtin(global_invocation_id) id: vec3<u32>) {
    let location = vec2<i32>(id.xy);
    let value = textureLoad(texture, location);
    let recolored_value = vec4<f32>(context.color[0], context.color[1], context.color[2], value[3]);

    storageBarrier();
    textureStore(texture, location, recolored_value);
}
