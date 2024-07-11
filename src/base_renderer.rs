use std::rc::Rc;

use wgpu::{
    Color, CommandEncoderDescriptor, Device, 
    LoadOp, Operations, Queue, 
    RenderPassColorAttachment, RenderPassDescriptor, 
    RenderPipeline, StoreOp, Surface, 
    SurfaceConfiguration, SurfaceError, TextureViewDescriptor
};
use winit::{
    dpi::PhysicalSize,
    event::{
        Event, WindowEvent
    },
    event_loop::EventLoop,
    window::Window,
};

use crate::{
    entity::EntityList, utils
};

pub struct BaseRenderer<'a, T> {
    surface: Surface<'a>,
    window: &'a Window,
    queue: Rc<Queue>,
    config: SurfaceConfiguration,
    device: Rc<Device>,
    size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    pub(crate) entities: EntityList, 
    multisample_texture: wgpu::Texture,
    // main_loop: Option<&'a mut dyn FnMut(&'a mut EntityList) -> ()>,
    main_loop: Option<T>,
}

impl<'a, T: for<'b> FnMut(&'b mut EntityList)> BaseRenderer<'a, T> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // handle to the GPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            // Uses Vulkan by default on Windows?
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .enumerate_adapters(wgpu::Backends::all())
            .into_iter()
            .filter(|adapter| {
                adapter.is_surface_supported(&surface)
            })
            .next()
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    ..Default::default()
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            // Check for srgb texture format support
            .filter(|format| format.is_srgb())
            .next()
            // If none is available, fallback to a format that is
            .unwrap_or(surface_capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            // 2 = more smooth, 1 = lower latency
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("base_shader.wgsl"));
    
        let render_pipeline = utils::generate_render_pipeline(&device, config.format, shader);

        let multisample_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Multisample texture"),
            size: wgpu::Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 4,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &vec![],
        });

        let device = Rc::from(device);
        let queue = Rc::from(queue);

        let entities = EntityList::new(device.clone(), queue.clone());

        Self {
            window,
            surface,
            config,
            device,
            queue,
            size,
            render_pipeline,
            entities,
            multisample_texture,
            main_loop: None,
        }
    }

    pub fn set_main_loop(&mut self, main_loop: T) {
        self.main_loop = Some(main_loop);
    }

    pub fn run(&mut self, event_loop: EventLoop<()>) {
        use coarsetime::Instant;

        let mut frames = 0u64;
        let mut fps = 0.6f32;
        let mut time = Instant::now();
        
        event_loop
            .run(move |event, window_target| {
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        println!("Closing...");
                        window_target.exit();
                    }

                    Event::AboutToWait => {
                        self.window.request_redraw();
                    }

                    // Rendering, updation
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => match self.render() {
                        Err(SurfaceError::Lost) => {
                            self.resize(self.size);
                        }
                        Err(SurfaceError::OutOfMemory) => {
                            println!("Out of memory! Exiting.");
                            window_target.exit();
                        }
                        Err(e) => {
                            println!("An error has occured when trying to render: {:?}", e);
                        }
                        Ok(()) => {
                            frames += 1;
                            if frames > 200 {
                                let elapsed = time.elapsed().as_micros();
                                fps = 1000000.0 * (frames as f32 / elapsed as f32);
                                self.window.set_title(format!("FPS: {}", fps).as_str());
                                frames = 0;
                                time = Instant::now();
                            }
                        }
                    },

                    // Input / User interaction
                    Event::WindowEvent {
                        event: WindowEvent::Resized(new_size),
                        ..
                    } => self.resize(new_size),

                    // Accessing the new_inner_size value?
                    // Event::WindowEvent { event: WindowEvent::ScaleFactorChanged { inner_size_writer, .. }, .. } => {
                    //     let new_size = inner_size_writer.request_inner_size();
                    // }
                    _ => {}
                }
            })
            .unwrap();
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.height > 0 && new_size.width > 0 {
            self.config.height = new_size.height;
            self.config.width = new_size.width;
            self.size.height = new_size.height;
            self.size.width = new_size.width;
            self.multisample_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Multisample texture"),
                size: wgpu::Extent3d { width: self.size.width, height: self.size.height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: self.config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &vec![],
            });
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        
        let multisample_view = 
            self.multisample_texture.create_view(&TextureViewDescriptor::default());
        
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Encoder of the renderer"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &multisample_view,
                resolve_target: Some(&view),
                ops: Operations {
                    load: LoadOp::Clear(Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0, }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    
        match &mut self.main_loop {
            Some(func) => func(&mut self.entities),
            None => (),
        };

        for entity in &self.entities.entities[..] {

            // Select shader here
            if let Some(render_pipeline) = &entity.render_pipeline {
                render_pass.set_pipeline(render_pipeline);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
            }

            // Set shader transform
            render_pass.set_bind_group(0, &entity.transform_bind_group, &[]);
            // Set shader parameters
            render_pass.set_bind_group(1, &entity.shader_bind_group, &[]);
            // Pass buffers
            render_pass.set_vertex_buffer(0, entity.vertex_buffer.slice(..));
            render_pass.set_index_buffer(entity.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            // Draw
            render_pass.draw_indexed(0..entity.index_size, 0, 0..1);   
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}

