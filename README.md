## Slime Simulation
This is a slime simulation heavily inspired by [Sebastian Lague's video](https://www.youtube.com/watch?v=X-iSQQgOd1A), written in Rust.
This simulation is made up of hundreds of thousands of individual agents (1,000,000 by default) which all individually move around the canvas. 
Each agent has three sensors, one directly in front, one to the left, and the other to the right. The use these sensors to look for the other agents which they then move towards.
Despite this simple rule, complex behaviours emerge creating intricate patterns. 

To be able to update 1,000,000 (or more) agents over 100 times per second compute shaders are used, which get run on the GPU.
The GPU is great for handling simple but heavily parallelized tasks. When using 1,000,000 agents at 144 updates per second on a 2560 x 1440 canvas,
my Nvidia RTX 3060Ti is at ~40% usage.

## Usage
Either download the Windows executable in releases section, or clone the repository and compile the source code with Cargo,
then simply run the executable. On the first run a TOML config file will be generated in the same directory.
This config file allows the configuration of the window size and the underlying texture (canvas) size.

## Images
Here are some examples of the simulation.

![](https://user-images.githubusercontent.com/52902343/230708083-1c38cf0f-9429-44a1-b769-82df2dc84353.png)
![](https://user-images.githubusercontent.com/52902343/230708309-b6eae48f-2fd6-43d4-a829-dbaba35dd73a.png)
![](https://user-images.githubusercontent.com/52902343/230708490-77142978-c882-408e-a941-933a7dcfe1f2.png)
![](https://user-images.githubusercontent.com/52902343/230708602-0fad5741-67db-4839-b61c-35c42d646842.png)
![](https://user-images.githubusercontent.com/52902343/230708217-7f88a51e-ddaa-42cf-be97-f04264557a02.png)
