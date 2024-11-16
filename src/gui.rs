use std::cmp::min;

use egui::{Align, Button, Checkbox, ClippedPrimitive, Context, Layout, Slider, TexturesDelta};
use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use egui_winit::EventResponse;
use pixels::{wgpu, PixelsContext};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window;

use crate::{World, WIDTH};

/// Manages all state required for rendering egui over `Pixels`.
pub struct Framework {
    // State for egui.
    egui_ctx: Context,
    egui_state: egui_winit::State,
    screen_descriptor: ScreenDescriptor,
    renderer: Renderer,
    paint_jobs: Vec<ClippedPrimitive>,
    textures: TexturesDelta,

    // State for the GUI
    gui: Gui,
}

/// Example application state. A real application will need a lot more state than this.
struct Gui {
    /// Only show the egui window when true.
    open_boid_window: bool,
    open_predator_window: bool,
    open_debug_window: bool,
}

impl Framework {
    /// Create egui.
    pub fn new<T>(
        event_loop: &EventLoopWindowTarget<T>,
        width: u32,
        height: u32,
        scale_factor: f32,
        pixels: &pixels::Pixels,
    ) -> Self {
        let max_texture_size = pixels.device().limits().max_texture_dimension_2d as usize;

        let egui_ctx = Context::default();
        let mut egui_state = egui_winit::State::new(event_loop);
        egui_state.set_max_texture_side(max_texture_size);
        egui_state.set_pixels_per_point(scale_factor);
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point: scale_factor,
        };
        let renderer = Renderer::new(pixels.device(), pixels.render_texture_format(), None, 1);
        let textures = TexturesDelta::default();
        let gui = Gui::new();

        Self {
            egui_ctx,
            egui_state,
            screen_descriptor,
            renderer,
            paint_jobs: Vec::new(),
            textures,
            gui,
        }
    }

    /// Handle input events from the window manager.
    pub fn handle_event(&mut self, event: &winit::event::WindowEvent) -> EventResponse {
        self.egui_state.on_event(&self.egui_ctx, event)
    }

    /// Resize egui.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.screen_descriptor.size_in_pixels = [width, height];
        }
    }

    /// Update scaling factor.
    pub fn scale_factor(&mut self, scale_factor: f64) {
        self.screen_descriptor.pixels_per_point = scale_factor as f32;
    }

    /// Prepare egui.
    pub fn prepare(&mut self, window: &Window, world: &mut World) {
        // Run the egui frame and create all paint jobs to prepare for rendering.
        let raw_input = self.egui_state.take_egui_input(window);
        let output = self.egui_ctx.run(raw_input, |egui_ctx| {
            // Draw the demo application.
            self.gui.ui(egui_ctx, world);
        });

        self.textures.append(output.textures_delta);
        self.egui_state
            .handle_platform_output(window, &self.egui_ctx, output.platform_output);
        self.paint_jobs = self.egui_ctx.tessellate(output.shapes);
    }

    /// Render egui.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        render_target: &wgpu::TextureView,
        context: &PixelsContext,
    ) {
        // Upload all resources to the GPU.
        for (id, image_delta) in &self.textures.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }
        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            encoder,
            &self.paint_jobs,
            &self.screen_descriptor,
        );

        // Render egui with WGPU
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: render_target,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.renderer
                .render(&mut rpass, &self.paint_jobs, &self.screen_descriptor);
        }

        // Cleanup
        let textures = std::mem::take(&mut self.textures);
        for id in &textures.free {
            self.renderer.free_texture(id);
        }
    }
}

impl Gui {
    /// Create a `Gui`.
    fn new() -> Self {
        Self { 
            open_boid_window: false,
            open_predator_window: false,
            open_debug_window: true,
        }
    }

    /// Create the UI using egui.
    fn ui(&mut self, ctx: &Context, world: &mut World) {
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Setting", |ui| {
                    if ui.button("Boid").clicked() {
                        self.open_boid_window = true;
                        ui.close_menu();
                    } else if ui.button("Predator").clicked() {
                        self.open_predator_window = true;
                        ui.close_menu();
                    } else if ui.button("Debug").clicked() {
                        self.open_debug_window = true;
                        ui.close_menu();
                    }
                })
            });
        });

        egui::Window::new("Boid")
            .open(&mut self.open_boid_window)
            .show(ctx, |ui| {
                ui.add(Slider::new(&mut world.option.avoid_factor, 0.0..=1.0).text("Avoid factor"));
                ui.add(Slider::new(&mut world.option.matching_factor, 0.0..=1.0).text("Matching factor"));
                ui.add(Slider::new(&mut world.option.centering_factor, 0.0..=1.0).text("Centering factor"));
                ui.add(Slider::new(&mut world.option.safe_radius, 0.0..=world.option.boid_vision_radius).text("Safe radius"));
                ui.add(Slider::new(&mut world.option.boid_vision_radius, 0.0..=WIDTH as f32).text("Vision radius"));
                ui.separator();
                ui.add(Slider::new(&mut world.option.boid_max_speed, world.option.boid_min_speed..=100).text("Max speed"));
                ui.add(Slider::new(&mut world.option.boid_min_speed, 0..=world.option.boid_max_speed).text("Min speed"));
                ui.separator();
                ui.add(Slider::new(&mut world.option.margin, 0..=500).text("Margin"));
                ui.add(Slider::new(&mut world.option.turn_factor, 0..=30).text("Turn factor"));
                ui.separator();
                ui.add(Slider::new(&mut world.option.boid_view_angle, 0.0..=365.0).text("View angle"));
                ui.add(Checkbox::new(&mut world.option.noise, "Add Noise"));
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    if ui.add(Button::new("Restart")).clicked() {
                        world.restart();
                    }
                    if ui.add(Button::new("Clear")).clicked() {
                        world.clear_all();
                    }
                });
            });

        egui::Window::new("Predator")
            .open(&mut self.open_predator_window)
            .show(ctx, |ui| {
                ui.add(Slider::new(&mut world.option.fear_factor, 0.0..=1.0).text("Fear factor"));
                ui.add(Slider::new(&mut world.option.fear_radius, 0.0..=WIDTH as f32).text("Fear radius"));
                ui.separator();
                ui.add(Slider::new(&mut world.option.predator_max_speed, world.option.predator_min_speed..=100).text("Max speed"));
                ui.add(Slider::new(&mut world.option.predator_min_speed, 0..=world.option.predator_max_speed).text("Min speed"));
                ui.separator();
                ui.add(Slider::new(&mut world.option.predator_vision_radius, 0.0..=WIDTH as f32).text("Vision radius"));
                ui.add(Slider::new(&mut world.option.predator_view_angle, 0.0..=365.0).text("View angle"));
                ui.separator();
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    if ui.add(Button::new("Restart")).clicked() {
                        world.restart();
                    }
                    if ui.add(Button::new("Clear")).clicked() {
                        world.clear_all();
                    }
                });
            });

        egui::Window::new("Debug")
            .open(&mut self.open_debug_window)
            .show(ctx, |ui| {
                ui.add(Checkbox::new(&mut world.option.show_quad_tree, "Show quad tree"));
                ui.add(Checkbox::new(&mut world.option.show_safe_radius, "Show safe radius"));
                ui.add(Checkbox::new(&mut world.option.show_vision_radius, "Show vision radius"));
                ui.add(Checkbox::new(&mut world.option.show_facing_direction_with_speed, "Show facing direction with speed"));
                ui.separator();
                ui.label(format!("FPS: {}", min(world.draw_fps as u16, world.update_fps as u16)));
                ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    if ui.add(Button::new("Restart")).clicked() {
                        world.restart();
                    }
                    if ui.add(Button::new("Clear")).clicked() {
                        world.clear_all();
                    }
                });
            });
    }
}
