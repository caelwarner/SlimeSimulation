@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(8, 8, 1)
fn fade(@builtin(global_invocation_id) id: vec3<u32>) {
    let location = vec2<i32>(id.xy);
    let value = textureLoad(texture, location);
    let faded_value = max(vec4<f32>(0.0), value - vec4<f32>(0.0, 0.0, 0.0, 0.005));

    storageBarrier();
    textureStore(texture, location, faded_value);
}
