use ash::{
    vk::{Semaphore, SemaphoreCreateInfo, Fence, FenceCreateInfo, FenceCreateFlags},
    Device,
    version::DeviceV1_0
};

pub struct FrameSyncData {
    pub image_available_semaphores: Vec<Semaphore>,
    pub render_finished_semaphores: Vec<Semaphore>,
    pub in_flight_fences: Vec<Fence>
}

pub fn create(device: &Device, max_frames_in_flight: usize) -> FrameSyncData {
    let mut image_available_semaphores = Vec::with_capacity(max_frames_in_flight);
    let mut render_finished_semaphores = Vec::with_capacity(max_frames_in_flight);
    let mut in_flight_fences = Vec::with_capacity(max_frames_in_flight);

    let semaphore_create_info = SemaphoreCreateInfo::default();
    let fence_create_info = FenceCreateInfo::builder()
        .flags(FenceCreateFlags::SIGNALED)
        .build();

    (0..max_frames_in_flight).into_iter().for_each(|_| {
        unsafe {
            let image_available_frame_semaphore = device.create_semaphore(&semaphore_create_info, None).expect("Failed to create image_available semaphore!");
            let render_finished_frame_semaphore = device.create_semaphore(&semaphore_create_info, None).expect("Failed to create render_finished semaphore!");
            let fence = device.create_fence(&fence_create_info, None).expect("Failed to create in-flight fence!");
            image_available_semaphores.push(image_available_frame_semaphore);
            render_finished_semaphores.push(render_finished_frame_semaphore);
            in_flight_fences.push(fence);
        }
    });


    FrameSyncData {
        image_available_semaphores,
        render_finished_semaphores,
        in_flight_fences
    }
}