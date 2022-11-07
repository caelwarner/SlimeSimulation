@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

struct Context {
    pause: u32,
    width: u32,
    height: u32,
    speed: f32,
    deltaTime: f32,
    time: f32,
}

struct Agent {
    position: vec2<f32>,
    @align(8) angle: f32,
}

@group(0) @binding(1)
var<uniform> context: Context;

@group(0) @binding(2)
var<storage, read_write> agents: array<Agent>;

fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn scaleTo01(value: u32) -> f32 {
    return f32(value) / 4294967295.0;
}

@compute @workgroup_size(16, 1, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    if (context.pause == u32(1)) {
        return;
    }

    var random = hash(u32(agents[id.x].position.x) * context.width + u32(agents[id.x].position.y) + hash(id.x + u32(context.time * 1000000.0)));

    let direction = vec2<f32>(cos(agents[id.x].angle), sin(agents[id.x].angle));
    var newPosition = agents[id.x].position + direction * context.speed * context.deltaTime * 50.0;

    if (newPosition.x < 0.0 || newPosition.x >= f32(context.width) || newPosition.y < 0.0 || newPosition.y >= f32(context.height)) {
        newPosition = vec2<f32>(
            min(f32(context.width) - 1.0, max(1.0, newPosition.x)),
            min(f32(context.height) - 1.0, max(1.0, newPosition.y)),
        );

        random = hash(random);
        agents[id.x].angle = scaleTo01(random) * 3.1415 * 2.0;
    }

    agents[id.x].position = newPosition;

    let location = vec2<i32>(agents[id.x].position);
    let color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    storageBarrier();
    textureStore(texture, location, color);
}
