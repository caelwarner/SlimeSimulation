@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

struct Agent {
    position: vec2<f32>,
    @align(8) angle: f32,
}

@group(0) @binding(1)
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

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(16, 1, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let direction = vec2<f32>(cos(agents[id.x].angle), sin(agents[id.x].angle));
    var newPosition = agents[id.x].position + direction * 1.0;

    if (newPosition.x < 0.0 || newPosition.y >= 3440.0 || newPosition.y < 0.0 || newPosition.y >= 1440.0) {
        newPosition = vec2<f32>(min(3440.0 - 0.01, max(0.0, newPosition.x)), min(1440.0 - 0.01, max(0.0, newPosition.y)));
        agents[id.x].angle = randomFloat(u32(agents[id.x].position.x) * u32(3440) + u32(agents[id.x].position.y) + hash(id.x)) * 3.14159 * 2.0;
    }

    agents[id.x].position = newPosition;

    let location = vec2<i32>(agents[id.x].position);
    let color = vec4<f32>(0.0, 1.0, 1.0, 1.0);

    storageBarrier();
    textureStore(texture, location, color);
}
