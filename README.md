This fork added support for Bevy 0.16.1. Working on support for camera rendertargets in compute shaders.

This crate is a plugin for the Bevy game engine to simplify the use of compute shaders.

It provides a pretty simple API. First, add the `BevyComputePlugin` to your Bevy app. To initiate the compute shaders, first set up all the needed buffers in the `ShaderBufferSet`. Then, send a `StartComputeEvent` with a `Vec` of `ComputeTask`s that will define the sequence of shaders to run. If relevant, be prepared to recieve `CopyBufferEvent`s, which will have buffer data returned from the computer shaders back to the CPU, and `ComputeTaskDoneEvent`s, which will tell you that a given compute task has completed.

And that's really it. But let's cover these steps in a big more detail.

# Add the Plugin

This is done in the standard way. Just add this call to your Bevy app initialization:

```Rust
app.add_plugins((BevyComputePlugin));
```

# Making Buffers

The `ShaderBufferSet` provides a simple API for managing GPU buffers. This is added as a resource by the `BevyComputePlugin`, so you can request `Res<ShaderBufferSet>` in any system to manage your buffers.

It provides the following functions for creating buffers:

- `add_storage_uninit` - Add an uninitialized storage buffer.
- `add_storage_zeroed` - Add a storage buffer filled with 0 bytes.
- `add_storage_init` - Add a storage buffer with initial data provided.
- `add_uniform_init` - Add a uniform buffer with initial data provided.
- `add_texture_fill` - Add a texture buffer filled with a solid color.

All of these return a `ShaderBufferHandle`, which you can store and treat like an opaque reference to access the buffer in the future. Except for `add_read_write_texture`, which returns a tuple of two such handles.

Every one of these functions takes a `Binding`, which determines how it's bound to the shaders. WGSL shaders require that each buffer have a group and a binding, which are numeric identifiers used to match the buffers specified on the CPU to those that exist in the shaders. The `Binding` is an enum, which can come in three types:

- `SingleBound(u32, u32)` - This is the standard binding. The first value is the group and the second the binding.
- `Double(u32, (u32, u32))` - This is a double buffer. There's actually two buffers. One is considered the front buffer, and one the back buffer, and they can be swapped. The first value the group both buffers will be in, and the tuple is the bindings of the front and back buffers, respectively. This is discussed in more detail in the "Double Buffering" section below.
- `SingleUnbound` - This buffer is not bound, and is thus inaccessible in shaders. While there are unbound buffers used in the background for data transmission purposes, it's rarely if ever useful to specify this at this level.

The `ShaderBufferSet` also provides a few more functions for managing buffers:

- `delete_buffer` - Predictably, this deletes a buffer.
- `image_handle` - Extracts the Bevy `Handle<Image>` associated with a texture buffer, so it can be displayed.
- `set_buffer` - Sets the contents of a buffer.

## Setting Buffer Contents

Buffer contents are internally just arrays of bytes, but they can be converted from more complicated data structures. This API uses the `ShaderType` trait to do that, which comes from the Encase crate that is included with Bevy. You can put `#[derive(ShaderType)]` in front of any data type, as long as all fields in that data type also implement `ShaderType`. All basic numeric types already do, along with any array, tuple or `Vec` of types that implement `ShaderType`. Which makes it very easy to pass whatever structured data you want into your shaders. Just be careful, because the shader has to specify the structure of the data independently, and if there's a mismatch it will only throw an error if they're a different size.

# Starting the Compute Shader

To start running the compute shaders, you need to throw a `StartComputeEvent`. This contains a `Vec` of `ComputeTask`s, which details all the compute tasks to complete, and a optional `ShaderBufferHandle`, for the optional iteration buffer.

## ComputeTask

A compute task represents one stage of your compute shader program. The compute task is optionally provided a number of iterations, and it will run for that many ticks before moving on to the next task. If that's not provided, it'll run forever. A compute task is also given a list of `ComputeStep`s, each of which is a specific shader to run, or other compute-related action to take, in order, each iteration. It can also be given an optional label, which is used to identify the task in the `ComputeTaskDoneEvent` that's thrown when the task completes.

Each `ComputeStep` contains just two fields.

The first is an optional maximum frequency. If provided, this means this step won't necessarily run every iteration, but only if it's been long enough since the last time it ran. The frequency is in Hz, or iterations per second. So if a max frequency of 30 is provided, that means if it's been less than 1000/30=16.67 ms since the last time it ran, then it won't run this iteration. This is often useful if you have a long running computation, and want to display the results in real time. You can potentially speed things up by only updating the display at a set framerate, even if the computation is running at a much faster rate.

The second field of the `ComputeStep` is a `ComputeAction`, which is an enum which describes what to actually do. It has the following options:

- `RunShader` - The meat of the compute shaders. This runs an actual shader. You must provide the Bevy asset path to the shader file, the name of the entry point function in that shader file, and the workgroup count in the x, y and z dimensions.
- `CopyBuffer` - Copy the data from a buffer to the CPU. Will be returned as a `Vec<u8>` via a `CopyBufferEvent`.
- `SwapBuffers` - Swap double buffers. See the "Double Buffering" section below.

# Double Buffering

It can sometimes be useful to have double buffers, where one buffer is the front buffer, and one the back buffer, and you read from the front buffer while writing to the back buffer, and then swap them for the next frame. This allows you to avoid reading from and writing to the same buffer, which can result in weird behavior when some of the data you're reading was written last frame, and some was written earlier this frame.

So this plugin supports this directly. When you declare a buffer with the `Double` binding type, it will actually create two buffers internally. One of them is considered the front buffer, which will be bound to the first binding provided, and the back buffer will be bound to the second binding. When the `SwapBuffers` compute action happens, it will swap which buffer is considered the front buffer.

When you do a `CopyBuffer` compute action on a double buffer, it will always copy out of the front buffer. Also, if you call the `image_handle` function on a double buffer texture, it will return the handle for the front buffer.


There's also a special accommodation for using a double buffered texture on a Bevy sprite. The `DoubleBufferedSprite` component requires a `Sprite` component, and it will automatically update that image handle on that sprite every frame to contain the new front buffer.
