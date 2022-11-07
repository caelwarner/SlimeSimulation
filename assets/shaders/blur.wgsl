struct Context {
    pause: u32,
    width: u32,
    height: u32,
    blurRadius: u32,
}

@group(0) @binding(0)
var textureIn: texture_storage_2d<rgba8unorm, read>;

@group(0) @binding(1)
var textureOut: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> context: Context;

@compute @workgroup_size(8, 8, 1)
fn blur(@builtin(global_invocation_id) id: vec3<u32>) {
    if (context.pause == u32(1)) {
        return;
    }

    let radius = i32(context.blurRadius);
    let location = vec2<i32>(id.xy);

    var sum = vec4<f32>(0.0);
    var divisor = 0;

    for (var x = -radius; x <= radius; x++) {
        for (var y = -radius; y <= radius; y++) {
            let currentLocation = location + vec2<i32>(x, y);

            if (currentLocation.x >= 0 && currentLocation.x < i32(context.width) && currentLocation.y >= 0 && currentLocation.y < i32(context.height)) {
                sum += textureLoad(textureIn, currentLocation);
                divisor++;
            }
        }
    }

    storageBarrier();
    textureStore(textureOut, location, sum / vec4<f32>(f32(divisor)));
}
