use crate::buffer::VEBuffer;
use crate::command_buffer::VECommandBuffer;
use crate::device::VEDevice;

pub struct VEVertexBuffer<'dev, 'mem, 'buf> {
    device: &'dev VEDevice,
    pub buffer: &'buf VEBuffer<'dev, 'mem>,
    pub vertex_count: u32,
}

impl<'dev, 'mem, 'buf> VEVertexBuffer<'dev, 'mem, 'buf> {
    pub fn new(
        device: &'dev VEDevice,
        buffer: &'buf VEBuffer<'dev, 'mem>,
        vertex_count: u32,
    ) -> VEVertexBuffer<'dev, 'mem, 'buf> {
        VEVertexBuffer {
            device,
            buffer,
            vertex_count,
        }
    }

    pub fn draw_instanced(&self, command_buffer: &VECommandBuffer, instances: u32) {
        unsafe {
            self.device.device.cmd_bind_vertex_buffers(
                command_buffer.handle,
                0,
                &[self.buffer.buffer],
                &[0],
            );
            self.device
                .device
                .cmd_draw(command_buffer.handle, self.vertex_count, instances, 0, 0);
        }
    }
}
