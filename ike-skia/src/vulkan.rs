use std::slice;

use ash::vk::{self, Handle};
use ike_core::Color;
use raw_window_handle::{DisplayHandle, WindowHandle};

use crate::{SkiaCanvas, SkiaPainter};

pub struct SkiaVulkanContext {
    entry:        ash::Entry,
    instance:     ash::Instance,
    physical:     vk::PhysicalDevice,
    device:       ash::Device,
    family_index: u32,
    queue:        vk::Queue,
}

impl Drop for SkiaVulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl SkiaVulkanContext {
    pub fn new(display: DisplayHandle) -> Self {
        let entry = unsafe { ash::Entry::load().unwrap() };
        let raw_display = display.as_raw();

        let api_version = unsafe {
            entry
                .try_enumerate_instance_version()
                .ok()
                .flatten()
                .unwrap_or(vk::API_VERSION_1_0)
        };

        tracing::debug!(
            "initializing vulkan with API v{}.{}.{}",
            vk::api_version_major(api_version),
            vk::api_version_minor(api_version),
            vk::api_version_patch(api_version),
        );

        let app_info = vk::ApplicationInfo::default()
            .engine_name(c"skia")
            .api_version(api_version);

        let extensions = ash_window::enumerate_required_extensions(raw_display).unwrap();
        let validation_layers = [c"VK_LAYER_KHRONOS_validation"];
        let validation_layer_names = validation_layers
            .into_iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();

        let mut instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(extensions);

        if cfg!(debug_assertions) {
            let is_validation_supported = unsafe {
                validation_layers.iter().all(|name| {
                    entry
                        .enumerate_instance_layer_properties()
                        .is_ok_and(|properties| {
                            properties
                                .iter()
                                .any(|p| p.layer_name_as_c_str() == Ok(*name))
                        })
                })
            };

            if is_validation_supported {
                instance_info = instance_info.enabled_layer_names(&validation_layer_names);
                tracing::debug!("initializing with validation layers");
            }
        }

        let instance = unsafe { entry.create_instance(&instance_info, None).unwrap() };
        let physical = unsafe { instance.enumerate_physical_devices().unwrap()[0] };

        let family_index = unsafe {
            instance
                .get_physical_device_queue_family_properties(physical)
                .iter()
                .position(|family| {
                    tracing::trace!(queue_family = ?family, "found queue family");
                    family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                })
                .unwrap()
        };

        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(family_index as u32)
            .queue_priorities(&[1.0])];

        let device_extensions = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];

        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_extension_names(&device_extensions);

        let device = unsafe {
            instance
                .create_device(physical, &device_info, None)
                .unwrap()
        };
        let queue = unsafe { device.get_device_queue(family_index as u32, 0) };

        Self {
            entry,
            instance,
            physical,
            device,
            family_index: family_index as u32,
            queue,
        }
    }
}

pub struct SkiaVulkanSurface {
    entry:            ash::Entry,
    instance:         ash::Instance,
    device:           ash::Device,
    queue:            vk::Queue,
    skia_context:     skia_safe::gpu::DirectContext,
    surface:          vk::SurfaceKHR,
    capabilities:     vk::SurfaceCapabilitiesKHR,
    present_mode:     vk::PresentModeKHR,
    surface_format:   vk::SurfaceFormatKHR,
    swapchain:        vk::SwapchainKHR,
    pre_transform:    vk::SurfaceTransformFlagsKHR,
    composite_alpha:  vk::CompositeAlphaFlagsKHR,
    command_pool:     vk::CommandPool,
    command_buffers:  Vec<vk::CommandBuffer>,
    image_available:  Vec<vk::Semaphore>,
    render_finished:  Vec<vk::Semaphore>,
    in_flight:        Vec<vk::Fence>,
    swapchain_images: Vec<vk::Image>,
    skia_surfaces:    Vec<(skia_safe::Surface, vk::Image)>,
    current_frame:    u32,
    width:            u32,
    height:           u32,
    msaa:             bool,
}

impl Drop for SkiaVulkanSurface {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();

            let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);
            device.destroy_swapchain(self.swapchain, None);

            for &image_available in &self.image_available {
                self.device.destroy_semaphore(image_available, None);
            }

            for &render_finished in &self.render_finished {
                self.device.destroy_semaphore(render_finished, None);
            }

            for &in_flight in &self.in_flight {
                self.device.destroy_fence(in_flight, None);
            }

            self.skia_surfaces.clear();
            self.device.destroy_command_pool(self.command_pool, None);

            let instance = ash::khr::surface::Instance::new(&self.entry, &self.instance);

            instance.destroy_surface(self.surface, None);
        }
    }
}

impl SkiaVulkanSurface {
    const SDR_FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
        format:      vk::Format::R8G8B8A8_UNORM,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };

    const HDR_FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
        format:      vk::Format::A2B10G10R10_UNORM_PACK32,
        color_space: vk::ColorSpaceKHR::HDR10_ST2084_EXT,
    };

    /// # Safety
    /// - `window` must have a valid display and window handle.
    /// - Display and window handles from `window` must be vaild for the `display` `context` was
    ///   created with.
    pub unsafe fn new(
        context: &mut SkiaVulkanContext,
        display: DisplayHandle,
        window: WindowHandle,
        width: u32,
        height: u32,
    ) -> Self {
        let skia_context = unsafe {
            let get_proc = |gpo| match gpo {
                skia_safe::gpu::vk::GetProcOf::Instance(instance, name) => {
                    let instance = vk::Instance::from_raw(instance as _);
                    context
                        .entry
                        .get_instance_proc_addr(instance, name)
                        .unwrap() as _
                }

                skia_safe::gpu::vk::GetProcOf::Device(device, name) => {
                    let device = vk::Device::from_raw(device as _);
                    (context.instance.fp_v1_0().get_device_proc_addr)(device, name).unwrap() as _
                }
            };

            skia_safe::gpu::direct_contexts::make_vulkan(
                &skia_safe::gpu::vk::BackendContext::new(
                    context.instance.handle().as_raw() as _,
                    context.physical.as_raw() as _,
                    context.device.handle().as_raw() as _,
                    (
                        context.queue.as_raw() as _,
                        context.family_index as usize,
                    ),
                    &get_proc,
                ),
                None,
            )
            .unwrap()
        };

        let surface = unsafe {
            ash_window::create_surface(
                &context.entry,
                &context.instance,
                display.as_raw(),
                window.as_raw(),
                None,
            )
            .unwrap()
        };

        let instance = ash::khr::surface::Instance::new(&context.entry, &context.instance);

        let capabilities = unsafe {
            instance
                .get_physical_device_surface_capabilities(context.physical, surface)
                .unwrap()
        };

        let present_modes = unsafe {
            instance
                .get_physical_device_surface_present_modes(context.physical, surface)
                .unwrap()
        };

        let present_mode = if present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
            vk::PresentModeKHR::MAILBOX
        } else if present_modes.contains(&vk::PresentModeKHR::IMMEDIATE) {
            vk::PresentModeKHR::IMMEDIATE
        } else {
            vk::PresentModeKHR::FIFO
        };

        let surface_formats = unsafe {
            instance
                .get_physical_device_surface_formats(context.physical, surface)
                .unwrap()
        };

        let surface_format = if surface_formats.contains(&Self::HDR_FORMAT) {
            Self::HDR_FORMAT
        } else if surface_formats.contains(&Self::SDR_FORMAT) {
            Self::SDR_FORMAT
        } else {
            panic!("could not find appropriate surface format");
        };

        let composite_alpha = if capabilities
            .supported_composite_alpha
            .contains(vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED)
        {
            vk::CompositeAlphaFlagsKHR::PRE_MULTIPLIED
        } else {
            vk::CompositeAlphaFlagsKHR::OPAQUE
        };

        tracing::debug!(
            ?present_mode,
            format = ?surface_format.format,
            color_space = ?surface_format.color_space,
            ?composite_alpha,
            "creating vulkan window",
        );

        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            context
                .device
                .create_command_pool(&pool_info, None)
                .unwrap()
        };

        let mut this = Self {
            entry: context.entry.clone(),
            instance: context.instance.clone(),
            device: context.device.clone(),
            queue: context.queue,
            skia_context,
            surface,
            capabilities,
            present_mode,
            surface_format,
            swapchain: vk::SwapchainKHR::null(),
            pre_transform: capabilities.current_transform,
            composite_alpha,
            command_pool,
            command_buffers: Vec::new(),
            image_available: Vec::new(),
            render_finished: Vec::new(),
            in_flight: Vec::new(),
            swapchain_images: Vec::new(),
            skia_surfaces: Vec::new(),
            current_frame: 0,
            width,
            height,
            msaa: !cfg!(any(
                target_os = "android",
                target_os = "ios",
            )),
        };

        this.resize(width, height);

        this
    }

    pub(crate) fn create_render_target(
        &mut self,
        width: u32,
        height: u32,
        is_primary: bool,
    ) -> skia_safe::Surface {
        let (color_type, color_space) = if self.surface_format == Self::HDR_FORMAT {
            if is_primary {
                (
                    skia_safe::ColorType::RGBA1010102,
                    skia_safe::ColorSpace::new_cicp(
                        skia_safe::named_primaries::CicpId::Rec2020,
                        skia_safe::named_transfer_fn::CicpId::PQ,
                    )
                    .unwrap(),
                )
            } else {
                (
                    skia_safe::ColorType::RGBAF16Norm,
                    skia_safe::ColorSpace::new_srgb(),
                )
            }
        } else {
            (
                skia_safe::ColorType::RGBA8888,
                skia_safe::ColorSpace::new_srgb(),
            )
        };

        let image_info = skia_safe::ImageInfo::new(
            skia_safe::ISize::new(
                width.max(1) as i32,
                height.max(1) as i32,
            ),
            color_type,
            skia_safe::AlphaType::Premul,
            color_space,
        );

        let samples = if self.msaa { 4 } else { 1 };
        skia_safe::gpu::surfaces::render_target(
            &mut self.skia_context,
            skia_safe::gpu::Budgeted::Yes,
            &image_info,
            Some(samples),
            skia_safe::gpu::SurfaceOrigin::TopLeft,
            None,
            false,
            None,
        )
        .unwrap()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        unsafe { self.device.device_wait_idle().unwrap() };

        self.skia_surfaces.clear();

        let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);

        unsafe { device.destroy_swapchain(self.swapchain, None) };

        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(self.surface)
            .min_image_count(self.capabilities.min_image_count.max(2))
            .image_array_layers(1)
            .image_format(self.surface_format.format)
            .image_color_space(self.surface_format.color_space)
            .image_extent(vk::Extent2D {
                width:  width.max(1),
                height: height.max(1),
            })
            .image_usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(self.pre_transform)
            .composite_alpha(self.composite_alpha)
            .present_mode(self.present_mode);

        self.swapchain = unsafe { device.create_swapchain(&swapchain_info, None).unwrap() };
        self.swapchain_images = unsafe { device.get_swapchain_images(self.swapchain).unwrap() };
        self.width = width;
        self.height = height;

        if self.command_buffers.len() < self.swapchain_images.len() {
            let buffer_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(self.command_pool)
                .command_buffer_count(self.swapchain_images.len() as u32)
                .level(vk::CommandBufferLevel::PRIMARY);

            self.command_buffers =
                unsafe { self.device.allocate_command_buffers(&buffer_info).unwrap() };
        }

        while self.image_available.len() < self.swapchain_images.len() {
            let image_available = unsafe {
                self.device
                    .create_semaphore(&Default::default(), None)
                    .unwrap()
            };

            self.image_available.push(image_available);
        }

        while self.render_finished.len() < self.swapchain_images.len() {
            let render_finished = unsafe {
                self.device
                    .create_semaphore(&Default::default(), None)
                    .unwrap()
            };

            self.render_finished.push(render_finished);
        }

        while self.in_flight.len() < self.swapchain_images.len() {
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            let in_flight = unsafe { self.device.create_fence(&fence_info, None).unwrap() };

            self.in_flight.push(in_flight);
        }

        while self.skia_surfaces.len() < self.swapchain_images.len() {
            let mut surface = self.create_render_target(width, height, true);
            let target = skia_safe::gpu::surfaces::get_backend_render_target(
                &mut surface,
                skia_safe::surface::BackendHandleAccess::FlushRead,
            )
            .unwrap();

            let image = vk::Image::from_raw(*target.vulkan_image_info().unwrap().image() as _);

            self.skia_surfaces.push((surface, image));
        }
    }

    pub fn draw<T>(
        &mut self,
        painter: &mut SkiaPainter,
        clear_color: Color,
        scale_factor: f32,
        pre_present: impl FnOnce(),
        f: impl FnOnce(&mut SkiaCanvas) -> T,
    ) -> Option<T> {
        let command_buffer = self.command_buffers[self.current_frame as usize];
        let image_available = self.image_available[self.current_frame as usize];
        let in_flight = self.in_flight[self.current_frame as usize];

        unsafe {
            // wait for resources to be available
            self.device
                .wait_for_fences(&[in_flight], true, u64::MAX)
                .unwrap();

            self.device.reset_fences(&[in_flight]).unwrap();

            // acquire swapchain image
            let device = ash::khr::swapchain::Device::new(&self.instance, &self.device);

            let (image_index, suboptimal) = match device.acquire_next_image(
                self.swapchain,
                u64::MAX,
                image_available,
                vk::Fence::null(),
            ) {
                Ok(image) => image,
                Err(error) => {
                    tracing::warn!(
                        ?error,
                        "error acquiring swapchain image",
                    );
                    return None;
                }
            };

            if suboptimal {
                tracing::trace!("swapchain suboptimal");
            }

            let swapchain_image = self.swapchain_images[image_index as usize];
            let render_finished = self.render_finished[image_index as usize];

            // do rendering
            let (mut surface, skia_image) = self.skia_surfaces[image_index as usize].clone();
            let canvas = surface.canvas();

            canvas.reset_matrix();
            canvas.scale((scale_factor, scale_factor));
            canvas.clear(skia_safe::Color4f::new(
                clear_color.r,
                clear_color.g,
                clear_color.b,
                clear_color.a,
            ));

            let output = {
                let mut canvas = SkiaCanvas {
                    surface: self,
                    painter,
                    canvas,
                };
                f(&mut canvas)
            };

            if self.msaa {
                // IMPORTANT: resolve skia msaa
                skia_safe::gpu::surfaces::resolve_msaa(&mut surface);
            }

            self.skia_context.flush_and_submit();

            // record command buffer
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            self.device
                .begin_command_buffer(command_buffer, &begin_info)
                .unwrap();

            let range = vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1)
                .level_count(1);

            let skia_to_transfer_src = vk::ImageMemoryBarrier::default()
                .image(skia_image)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .src_access_mask(vk::AccessFlags::NONE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .subresource_range(range);

            let swapchain_to_transfer_dst = vk::ImageMemoryBarrier::default()
                .image(swapchain_image)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .subresource_range(range);

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[skia_to_transfer_src, swapchain_to_transfer_dst],
            );

            self.device.cmd_copy_image(
                command_buffer,
                skia_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                swapchain_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[vk::ImageCopy::default()
                    .src_offset(Default::default())
                    .src_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .dst_offset(Default::default())
                    .dst_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .layer_count(1),
                    )
                    .extent(
                        vk::Extent3D::default()
                            .width(self.width)
                            .height(self.height)
                            .depth(1),
                    )],
            );

            let skia_to_transfer_dst = vk::ImageMemoryBarrier::default()
                .image(skia_image)
                .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .subresource_range(range);

            let swapchain_to_present = vk::ImageMemoryBarrier::default()
                .image(swapchain_image)
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                .subresource_range(range);

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[skia_to_transfer_dst, swapchain_to_present],
            );

            self.device.end_command_buffer(command_buffer).unwrap();

            let submit = vk::SubmitInfo::default()
                .wait_semaphores(slice::from_ref(&image_available))
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::TRANSFER])
                .command_buffers(slice::from_ref(&command_buffer))
                .signal_semaphores(slice::from_ref(&render_finished));

            self.device
                .queue_submit(self.queue, &[submit], in_flight)
                .unwrap();

            let present_info = vk::PresentInfoKHR::default()
                .swapchains(slice::from_ref(&self.swapchain))
                .image_indices(slice::from_ref(&image_index))
                .wait_semaphores(slice::from_ref(&render_finished));

            pre_present();
            device.queue_present(self.queue, &present_info).unwrap();

            self.current_frame = (self.current_frame + 1) % self.swapchain_images.len() as u32;

            Some(output)
        }
    }
}
