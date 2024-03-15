#![allow(unused_assignments)]
mod bitmap;
use bitmap::Bitmap;
use std::sync::Arc;
use wgpu::include_wgsl;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const LOGICAL_WIDTH: f64 = 800.0;
const LOGICAL_HEIGHT: f64 = 600.0;

pub async fn run() {
    let inner_size = LogicalSize::new(LOGICAL_WIDTH, LOGICAL_HEIGHT);
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("winit-macos-mouse-movement")
            .with_inner_size(inner_size)
            .with_min_inner_size(inner_size)
            .with_max_inner_size(inner_size)
            .build(&event_loop)
            .unwrap(),
    );

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = instance.create_surface(window.clone()).unwrap();
    let mut size = window.inner_size();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: Default::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .unwrap();

    let caps = surface.get_capabilities(&adapter);
    let format = caps.formats[0];
    surface.configure(
        &device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        },
    );

    let mut bitmap = Bitmap::new(size.width, size.height);

    let bitmap_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let bitmap_view = bitmap_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let bitmap_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    bitmap.write_texture(&queue, &bitmap_texture);

    let shader_module = device.create_shader_module(include_wgsl!("shader.wgsl"));
    let texture_bg_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let texture_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &texture_bg_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&bitmap_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&bitmap_sampler),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&texture_bg_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: None,
        multiview: None,
        multisample: wgpu::MultisampleState::default(),
    });

    let mut mouse_down = false;
    let mut last_drawn_sample: Option<PhysicalPosition<f64>> = None;
    let mut samples: Vec<PhysicalPosition<f64>> = vec![];

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    }

                    WindowEvent::Resized(new_size) => {
                        if new_size.width > 0 && new_size.height > 0 {
                            size = new_size;
                            surface.configure(
                                &device,
                                &wgpu::SurfaceConfiguration {
                                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                    format,
                                    width: size.width,
                                    height: size.height,
                                    present_mode: wgpu::PresentMode::Fifo,
                                    desired_maximum_frame_latency: 2,
                                    alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                                    view_formats: vec![],
                                },
                            );
                        }
                    }

                    WindowEvent::RedrawRequested => {
                        match samples.len() {
                            0 => {}
                            1 => {
                                let p = samples[0];

                                match last_drawn_sample {
                                    Some(p0) => {
                                        bitmap.draw_line(
                                            (p0.x.round() as i64, p0.y.round() as i64),
                                            (p.x.round() as i64, p.y.round() as i64),
                                            255,
                                            0,
                                            0,
                                        );
                                    }
                                    None => {
                                        bitmap.set_pixel(
                                            p.x.round() as i64,
                                            p.y.round() as i64,
                                            255,
                                            0,
                                            0,
                                            255,
                                        );
                                    }
                                }
                                last_drawn_sample = Some(p);
                            }
                            _ => {
                                let mut current = samples[0];

                                if let Some(p) = last_drawn_sample {
                                    let from = (p.x.round() as i64, p.y.round() as i64);
                                    let to = (current.x.round() as i64, current.y.round() as i64);
                                    bitmap.draw_line(from, to, 255, 0, 0);
                                }

                                for sample in samples.iter().skip(1) {
                                    let from = (current.x.round() as i64, current.y.round() as i64);
                                    let to = (sample.x.round() as i64, sample.y.round() as i64);
                                    bitmap.draw_line(from, to, 255, 0, 0);
                                    current = *sample;
                                }
                                last_drawn_sample = Some(current);
                            }
                        }

                        bitmap.write_texture(&queue, &bitmap_texture);

                        let output = surface.get_current_texture().unwrap();
                        let view = output
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        let mut enc =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        {
                            let mut rpass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 1.0,
                                            g: 1.0,
                                            b: 1.0,
                                            a: 1.0,
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                            rpass.set_pipeline(&pipeline);
                            rpass.set_bind_group(0, &texture_bg, &[]);
                            rpass.draw(0..6, 0..1);
                        }

                        queue.submit(Some(enc.finish()));
                        output.present();

                        samples.clear();

                        if !mouse_down {
                            last_drawn_sample = None;
                        }

                        window.request_redraw();
                    }

                    WindowEvent::CursorMoved { position, .. } => {
                        if mouse_down {
                            samples.push(position);
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if button == MouseButton::Left {
                            mouse_down = state == ElementState::Pressed;
                        }
                    }

                    _ => {}
                }
            }
        })
        .unwrap();
}

pub fn main() {
    pollster::block_on(run());
}
