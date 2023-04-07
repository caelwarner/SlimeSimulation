struct Context {
    pause: u32, // align(4)
    width: u32,
    height: u32,
    speed: f32,
    deltaTime: f32,
    time: f32,
    senseAngleOffset: f32,
    senseDistance: f32,
    turnSpeed: f32,
    turnRandomness: f32,
}

struct Agent {
    position: vec2<f32>,
    @align(8) angle: f32,
}

@group(0) @binding(0)
var textureIn: texture_storage_2d<rgba8unorm, read>;

@group(0) @binding(1)
var textureOut: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> context: Context;

@group(0) @binding(3)
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

fn sense(id: u32, angleOffset: f32) -> f32 {
    let angle = agents[id].angle + angleOffset;
    let direction = vec2<f32>(cos(angle), sin(angle));
    let sensePosition = vec2<i32>(agents[id].position + direction * context.senseDistance);

    var sum = 0.0;

    for (var x = -2; x <= 2; x++) {
        for (var y = -2; y <= 2; y++) {
            sum += textureLoad(textureIn, sensePosition)[3];
        }
    }

    return sum;
}

@compute @workgroup_size(16, 1, 1)
fn update(@builtin(global_invocation_id) id: vec3<u32>) {
    if (context.pause == u32(1)) {
        return;
    }

    var random = hash(u32(agents[id.x].position.x) * context.width + u32(agents[id.x].position.y) + hash(id.x + u32(context.time * 1000000.0)));

    let senseLeft = sense(id.x, -context.senseAngleOffset);
    let senseForward = sense(id.x, 0.0);
    let senseRight = sense(id.x, context.senseAngleOffset);

    let turnSpeed = context.turnSpeed * 3.1415 * 2.0 * 0.01;
    let turnRandomness = scaleTo01(random);

    if (senseLeft > senseForward && senseLeft > senseRight) {
        agents[id.x].angle -= turnSpeed;
    } else if (senseRight > senseForward && senseRight > senseLeft) {
        agents[id.x].angle += turnSpeed;
    }

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
    textureStore(textureOut, location, color);
}
