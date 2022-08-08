# SDF Viewer ([demo below](#demo-try-it))

*A fast and cross-platform Signed Distance Function (SDF) viewer, easily integrated with your SDF library.*

A Signed Distance Function/Field (SDF) is an alternative approach to design 3D objects.
In its simplest form, it is just a function that maps each point in 3D space to the signed distance to the closest
surface of the object. It can underestimate the distance to the surface, but it can never overestimate the distance.
A negative distance means that the point is inside the object.

SDF libraries will provide you with a wide range of SDF primitives, both 2D (rectangle, circle, text, etc.) and 3D (box,
sphere, cylinder, triangle mesh, etc.), and operators (union, difference, intersection, extrude, revolve, etc.) to use
in your designs. This application will take your code (written in any language) and quickly and interactively render it
to a window. The main objetive of this app is to speed up the design/modeling phase by rendering the changes to your SDF
as fast as possible.

I use this to design objects for 3D printing, but it can be used for any 3D modeling task.
If you are looking for inspiration or want to learn how to build your own SDF library, check
out [shadertoy](https://www.shadertoy.com/results?query=tag%3Ddistancefields).

## Features / future plans

- [x] Cross-platform: desktop (Linux, Windows, MacOS) and web.
- [x] Cross-language: easy to integrate with most languages and frameworks.
    - [x] Works by building your code for wasm and running it at near-native speeds.
    - [x] [Rust demo](src/sdf/demo/ffi.rs), observable and customizable
      through [this link](https://yeicor.github.io/sdf-viewer/?cliurl=demo_sdf.wasm&envdark).
    - [x] Development server to ease integration and allow remote rendering.
    - [x] Testing / feature showcase integration and list of [available integrations](#integrations).
- [x] High-performance:
    - [x] Very-fast initialization on all platforms.
    - [x] Interactive framerate, even while loading (uses the GPU for viewing the SDF).
    - [x] Loads SDFs in several passes, increasing the level of detail in each one.
- [x] Easily customizable:
    - [x] Different rendering materials (color, metallic, roughness...).
    - [x] Parameters to quickly customize your SDF from the UI.
- [x] Compatible:
    - [x] Upload your SDF to a server and display it anywhere by
      adding [?cliurl=\<url>](https://yeicor.github.io/sdf-viewer/?cliurl=demo_sdf.wasm&envdark) to the link.
    - [ ] Export a triangle mesh (importing should be provided by your SDF library).
- [ ] [TODO](https://github.com/Yeicor/sdf-viewer/search?q=TODO)s, [FIXME](https://github.com/Yeicor/sdf-viewer/search?q=FIXME)s
  and [HACK](https://github.com/Yeicor/sdf-viewer/search?q=HACK)s (any help is appreciated 😉).

## Demo ([try it!](https://yeicor.github.io/sdf-viewer/?envdark))

This example loads and renders an SDF with custom materials at the maximum framerate of **60 FPS on an integrated
graphics card** (i7-9750H). The initial load of the SDF is also interactive at 30 FPS (configurable): it quickly loads a
low-resolution version of the object and iteratively increases the level of detail until the SDF is fully loaded.
Parameters are configured from the UI, also rendering the changes in real time.

![demo.gif](.github/docs/demo.gif)

Another slightly more complex example (see [sdf-viewer-go](https://github.com/Yeicor/sdf-viewer-go/) for more
information):

- It renders an object defined using the Go language.
- It recompiles and reloads the SDF when a change in the code is detected

![demo2.gif](https://github.com/Yeicor/sdf-viewer-go/raw/main/.github/docs/demo.gif)

## Integrations

| Repo                                                      | Language | Library                                                                                                               | Features                          | Notes                                                                                                     |
|:----------------------------------------------------------|----------|-----------------------------------------------------------------------------------------------------------------------|-----------------------------------|-----------------------------------------------------------------------------------------------------------|
| [sdf-viewer](https://github.com/Yeicor/sdf-viewer/)       | Rust     | [Pure Rust](src/sdf/mod.rs)                                                                                           | Core<br/>Hierarchy<br/>Parameters | Demo / feature showcase<br/>Used for testing, not extensible                                              |
| [sdf-viewer-go](https://github.com/Yeicor/sdf-viewer-go/) | Go       | [Pure Go](https://github.com/Yeicor/sdf-viewer-go/tree/main/sdf-viewer-go)<br/>[SDFX](https://github.com/deadsy/sdfx) | Core<br/>Hierarchy<br/>Parameters | May be used as a guide for implementing your own<br/>integration due to the simplicity of the Go language |

**Feel free to create integrations for other languages and frameworks and add them to this list!**

It is very simple to integrate with other languages and frameworks. The only requirement to provide support for a
language is the ability to compile to WASM exporting at least the core methods (getting bounding box and sampling the
distance at any point).

## Implementation details

### Rendering

The renderer is a GPU-accelerated raytracer. To take the SDF definition written for the CPU and render it with the GPU, I had to fill a 3D texture that is then raytraced by a shader. This 3D texture represents samples of the SDF in the 3D grid that contains the object. Each sample contains the distance to the surface and some other material properties like the color and roughness.

Afterward, [this GLSL shader](https://github.com/Yeicor/sdf-viewer/blob/master/src/app/scene/sdf/material.frag) does the actual rendering. This shader is applied to a cuboid mesh that represents the bounding box of the object. The mesh is useful for only raytracing the part of the screen that may reach the object, and for extracting the rays for each pixel from the hit points. The shader simply walks along the ray for each pixel, moving by the amount of distance reported by the SDF on each position. If the surface is reached at some point, the normal is computed and the lighting is applied for the material saved in the closest voxel. To get the distance at a point that does not match the grid, interpolation is applied, leading to round corners if the level of detail is not high enough.

The distance function must always be equal to or underestimate the distance to the closest point on the surface of the 3D model. An invalid SDF would cause rendering issues such as "stairs" when looking at a flat surface at an angle. Using this renderer is a nice way of testing for issues while developing objects for an SDF library.

I chose raytracing instead of meshing as building a detailed mesh is slower. While loading the SDF into the 3D texture mentioned above, the shader is capable of rendering a real-time preview of the object, which provides much better interactivity. This is enhanced by the fact that I do several passes to the grid slowly increasing the level of detail by filling more voxels with data, in a way similar to [bitmap interlacing](https://en.wikipedia.org/wiki/Interlacing_(bitmaps)).

A high-quality meshing algorithm that preserves sharp features should be applied to get the final model, but the objective of this app is to interactively render previews while designing 3D models through code.

### Building

All [releases](https://github.com/Yeicor/sdf-viewer/releases) include builds for most platforms.
Follow the [release.yml](.github/workflows/release.yml) workflow to learn how to build the project by yourself.

