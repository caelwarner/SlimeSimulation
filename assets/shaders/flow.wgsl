@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, read_write>;

struct Time {
    time_since_startup: f32,
};

@group(0) @binding(1)
var<uniform> time: Time;

struct Agent {
    position: vec2<f32>,
    angle: f32,
    _padding: u32,
}

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

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(16, 1, 1)
fn init(@builtin(global_invocation_id) id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
//    agents[id.x].position = vec2<f32>(1280.0 / 2.0, 720.0 / 2.0);
//    agents[id.x].angle = randomFloat(id.x) * 360.0;

//    let location = vec2<i32>(agents[id.x].position);
//    let color = vec4<f32>(1.0);
//
//    textureStore(texture, location, color);
}

@compute @workgroup_size(16, 1, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    let direction = vec2<f32>(cos(agents[id.x].angle), sin(agents[id.x].angle));
    var newPosition = agents[id.x].position + direction * 2.0;

    if (newPosition.x < 0.0 || newPosition.y >= 1280.0 || newPosition.y < 0.0 || newPosition.y >= 720.0) {
        newPosition = vec2<f32>(min(1280.0 - 0.01, max(0.0, newPosition.x)), min(720.0 - 0.01, max(0.0, newPosition.y)));
        agents[id.x].angle = randomFloat(u32(agents[id.x].position.x) * u32(1280) + u32(agents[id.x].position.y) + hash(id.x)) * 3.14159 * 2.0;
    }

    agents[id.x].position = newPosition;

    let location = vec2<i32>(agents[id.x].position);
    let color = vec4<f32>(1.0);

    storageBarrier();
    textureStore(texture, location, color);
}
