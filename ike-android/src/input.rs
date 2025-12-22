use std::ptr;

use ike_core::{Point, TouchId, WindowId};

use crate::{EventLoop, WindowState};

#[derive(Debug)]
pub(super) enum InputQueueEvent {
    Created(*mut ndk_sys::AInputQueue),
    Destroyed(*mut ndk_sys::AInputQueue),
}

unsafe impl Send for InputQueueEvent {}

impl<'a, T> EventLoop<'a, T> {
    pub fn handle_input_queue_event(&mut self, event: InputQueueEvent) {
        match event {
            InputQueueEvent::Created(queue) => {
                tracing::trace!(
                    looper = ?self.looper,
                    queue = ?queue,
                    "attaching looper to input queue",
                );

                unsafe {
                    ndk_sys::AInputQueue_attachLooper(
                        queue,
                        self.looper,
                        42069,
                        None,
                        ptr::null_mut(),
                    );
                };

                self.input_queue = Some(queue);
            }

            InputQueueEvent::Destroyed(queue) => {
                tracing::trace!(
                    queue = ?queue,
                    "input queue destroyed",
                );

                self.input_queue = None;
            }
        }
    }

    pub fn handle_input_events(&mut self) {
        if let Some(queue) = self.input_queue {
            unsafe {
                let mut event = ptr::null_mut();
                while ndk_sys::AInputQueue_getEvent(queue, &mut event) >= 0 {
                    let handled = self.handle_input_event(event);
                    ndk_sys::AInputQueue_finishEvent(queue, event, handled as i32);
                }
            }
        }
    }

    fn handle_input_event(&mut self, event: *mut ndk_sys::AInputEvent) -> bool {
        let WindowState::Open(ref window) = self.window else {
            return false;
        };

        let Some(id) = window.id else {
            return false;
        };

        let kind = unsafe { ndk_sys::AInputEvent_getType(event) };

        match kind as u32 {
            ndk_sys::AINPUT_EVENT_TYPE_MOTION => self.handle_motion_event(id, event),
            _ => false,
        }
    }

    fn handle_motion_event(
        &mut self,
        window_id: WindowId,
        event: *mut ndk_sys::AInputEvent,
    ) -> bool {
        let action = unsafe { ndk_sys::AMotionEvent_getAction(event) as u32 };
        let index = ((action & ndk_sys::AMOTION_EVENT_ACTION_POINTER_INDEX_MASK)
            >> ndk_sys::AMOTION_EVENT_ACTION_POINTER_INDEX_SHIFT) as usize;
        let action = action & ndk_sys::AMOTION_EVENT_ACTION_MASK;

        match action {
            ndk_sys::AMOTION_EVENT_ACTION_DOWN => {
                let x = unsafe { ndk_sys::AMotionEvent_getX(event, index) };
                let y = unsafe { ndk_sys::AMotionEvent_getY(event, index) };
                let point = Point::new(
                    x / self.scale_factor,
                    y / self.scale_factor,
                );

                let id = unsafe { ndk_sys::AMotionEvent_getPointerId(event, index) };
                let touch_id = TouchId::from_u64(id as u64);

                self.context.touch_down(window_id, touch_id, point)
            }

            ndk_sys::AMOTION_EVENT_ACTION_UP => {
                let x = unsafe { ndk_sys::AMotionEvent_getX(event, index) };
                let y = unsafe { ndk_sys::AMotionEvent_getY(event, index) };
                let point = Point::new(
                    x / self.scale_factor,
                    y / self.scale_factor,
                );

                let id = unsafe { ndk_sys::AMotionEvent_getPointerId(event, index) };
                let touch_id = TouchId::from_u64(id as u64);

                self.context.touch_up(window_id, touch_id, point)
            }

            ndk_sys::AMOTION_EVENT_ACTION_MOVE => {
                let count = unsafe { ndk_sys::AMotionEvent_getPointerCount(event) };
                let mut handled = false;

                for i in 0..count {
                    let x = unsafe { ndk_sys::AMotionEvent_getX(event, i) };
                    let y = unsafe { ndk_sys::AMotionEvent_getY(event, i) };
                    let point = Point::new(
                        x / self.scale_factor,
                        y / self.scale_factor,
                    );

                    let id = unsafe { ndk_sys::AMotionEvent_getPointerId(event, i) };
                    let tool = unsafe { ndk_sys::AMotionEvent_getToolType(event, i) };
                    let touch_id = TouchId::from_u64(id as u64);

                    tracing::trace!(index, ?point, tool, "move event");

                    handled |= self.context.touch_moved(window_id, touch_id, point);
                }

                handled
            }

            _ => false,
        }
    }
}
