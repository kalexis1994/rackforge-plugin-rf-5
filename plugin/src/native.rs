use super::Rf5Processor;
use rackforge_plugin_api::abi::{
    ABI_VERSION, HostApiV1, MidiEventV1, ParameterEventV1, PluginApiV1, ProcessBlockV1,
    STATUS_INVALID_ARGUMENT, STATUS_INVALID_STATE, STATUS_OK, STATUS_UNKNOWN_PARAMETER,
    copy_to_host_buffer,
};
use rackforge_plugin_sdk::{MidiEvent, ParameterEvent, Processor};
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::slice;

const RUNTIME_DESCRIPTOR: &[u8] = include_bytes!("../package/metadata/runtime.json");
const PARAMETER_SCHEMA: &[u8] = include_bytes!("../package/metadata/parameters.json");
const PRESET_CATALOG: &[u8] = include_bytes!("../package/metadata/presets.json");
const MAX_REALTIME_EVENTS: usize = 256;

#[derive(Default)]
struct NativeInstance {
    processor: Rf5Processor,
    active: bool,
    maximum_frames: u32,
    output_channels: u32,
}

unsafe extern "C" fn write_runtime_descriptor(destination: *mut u8, capacity: usize) -> usize {
    // SAFETY: RackForge owns and describes the destination buffer.
    unsafe { copy_to_host_buffer(RUNTIME_DESCRIPTOR, destination, capacity) }
}

unsafe extern "C" fn write_parameter_schema(destination: *mut u8, capacity: usize) -> usize {
    // SAFETY: RackForge owns and describes the destination buffer.
    unsafe { copy_to_host_buffer(PARAMETER_SCHEMA, destination, capacity) }
}

unsafe extern "C" fn write_preset_catalog(destination: *mut u8, capacity: usize) -> usize {
    // SAFETY: RackForge owns and describes the destination buffer.
    unsafe { copy_to_host_buffer(PRESET_CATALOG, destination, capacity) }
}

unsafe extern "C" fn create(_host: *const HostApiV1) -> *mut c_void {
    Box::into_raw(Box::new(NativeInstance::default())).cast()
}

unsafe extern "C" fn destroy(instance: *mut c_void) {
    if !instance.is_null() {
        // SAFETY: `create` returned this allocation and RackForge destroys it once.
        unsafe { drop(Box::from_raw(instance.cast::<NativeInstance>())) };
    }
}

unsafe extern "C" fn activate(
    instance: *mut c_void,
    sample_rate: f64,
    maximum_frames: u32,
    input_channels: u32,
    output_channels: u32,
) -> i32 {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if maximum_frames == 0
        || !instance
            .processor
            .prepare(sample_rate, maximum_frames, input_channels, output_channels)
    {
        return STATUS_INVALID_ARGUMENT;
    }
    instance.maximum_frames = maximum_frames;
    instance.output_channels = output_channels;
    instance.active = true;
    STATUS_OK
}

unsafe extern "C" fn deactivate(instance: *mut c_void) -> i32 {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    instance.active = false;
    STATUS_OK
}

unsafe extern "C" fn reset(instance: *mut c_void) -> i32 {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    instance.processor.reset();
    STATUS_OK
}

unsafe extern "C" fn set_parameter(instance: *mut c_void, parameter_index: u32, value: f64) -> i32 {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if !value.is_finite() {
        return STATUS_INVALID_ARGUMENT;
    }
    if instance.processor.set_parameter(parameter_index, value) {
        STATUS_OK
    } else {
        STATUS_UNKNOWN_PARAMETER
    }
}

unsafe extern "C" fn get_parameter(
    instance: *mut c_void,
    parameter_index: u32,
    destination: *mut f64,
) -> i32 {
    let (Some(instance), Some(destination)) = (
        unsafe { instance.cast::<NativeInstance>().as_ref() },
        unsafe { destination.as_mut() },
    ) else {
        return STATUS_INVALID_ARGUMENT;
    };
    let Some(value) = instance.processor.get_parameter(parameter_index) else {
        return STATUS_UNKNOWN_PARAMETER;
    };
    *destination = value;
    STATUS_OK
}

unsafe extern "C" fn save_state(
    instance: *mut c_void,
    destination: *mut u8,
    capacity: usize,
) -> usize {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_ref() }) else {
        return 0;
    };
    let mut state = [0_u8; 4_096];
    let Some(length) = instance.processor.save_state(&mut state) else {
        return 0;
    };
    // SAFETY: RackForge owns and describes the destination buffer.
    unsafe { copy_to_host_buffer(&state[..length], destination, capacity) }
}

unsafe extern "C" fn load_state(instance: *mut c_void, source: *const u8, length: usize) -> i32 {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if source.is_null() || length == 0 {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: RackForge guarantees `length` readable bytes for this call.
    let state = unsafe { slice::from_raw_parts(source, length) };
    if instance.processor.load_state(state) {
        STATUS_OK
    } else {
        STATUS_INVALID_ARGUMENT
    }
}

unsafe extern "C" fn load_preset(
    instance: *mut c_void,
    preset_id: *const u8,
    length: usize,
) -> i32 {
    let Some(instance) = (unsafe { instance.cast::<NativeInstance>().as_mut() }) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if preset_id.is_null() || length == 0 {
        return STATUS_INVALID_ARGUMENT;
    }
    // SAFETY: RackForge guarantees `length` readable bytes for this call.
    let bytes = unsafe { slice::from_raw_parts(preset_id, length) };
    let Ok(id) = str::from_utf8(bytes) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if instance.processor.load_preset(id) {
        STATUS_OK
    } else {
        STATUS_INVALID_ARGUMENT
    }
}

unsafe extern "C" fn process(instance: *mut c_void, block: *const ProcessBlockV1) -> i32 {
    let (Some(instance), Some(block)) = (
        unsafe { instance.cast::<NativeInstance>().as_mut() },
        unsafe { block.as_ref() },
    ) else {
        return STATUS_INVALID_ARGUMENT;
    };
    if !instance.active
        || block.struct_size < size_of::<ProcessBlockV1>() as u32
        || block.frames > instance.maximum_frames
        || block.input_channels != 0
        || block.output_channels != instance.output_channels
        || block.output_interleaved.is_null()
    {
        return STATUS_INVALID_STATE;
    }

    let midi = match native_midi_events(block) {
        Some(events) => events,
        None => return STATUS_INVALID_ARGUMENT,
    };
    let parameters = match native_parameter_events(block) {
        Some(events) => events,
        None => return STATUS_INVALID_ARGUMENT,
    };
    if !events_are_ordered(midi, block.frames)
        || !parameters_are_ordered(parameters, block.frames)
        || midi.len() > MAX_REALTIME_EVENTS
        || parameters.len() > MAX_REALTIME_EVENTS
    {
        return STATUS_INVALID_ARGUMENT;
    }

    let output_length = block.frames as usize * block.output_channels as usize;
    // SAFETY: RackForge sizes the output buffer from the validated block fields.
    let output = unsafe { slice::from_raw_parts_mut(block.output_interleaved, output_length) };
    let mut midi_events = [MidiEvent {
        frame: 0,
        data: [0; 3],
        length: 1,
    }; MAX_REALTIME_EVENTS];
    for (destination, event) in midi_events.iter_mut().zip(midi) {
        *destination = MidiEvent {
            frame: event.frame,
            data: event.data,
            length: event.length,
        };
    }
    let mut parameter_events = [ParameterEvent {
        frame: 0,
        index: 0,
        value: 0.0,
    }; MAX_REALTIME_EVENTS];
    for (destination, event) in parameter_events.iter_mut().zip(parameters) {
        *destination = ParameterEvent {
            frame: event.frame,
            index: event.parameter_index,
            value: event.value,
        };
    }
    instance.processor.process(
        &[],
        output,
        &midi_events[..midi.len()],
        &parameter_events[..parameters.len()],
        block.frames,
        0,
        block.output_channels,
    );
    STATUS_OK
}

fn native_midi_events(block: &ProcessBlockV1) -> Option<&[MidiEventV1]> {
    if block.midi_event_count == 0 {
        return Some(&[]);
    }
    if block.midi_events.is_null() {
        return None;
    }
    // SAFETY: the host provides exactly `midi_event_count` events for the call.
    Some(unsafe { slice::from_raw_parts(block.midi_events, block.midi_event_count as usize) })
}

fn native_parameter_events(block: &ProcessBlockV1) -> Option<&[ParameterEventV1]> {
    if block.parameter_event_count == 0 {
        return Some(&[]);
    }
    if block.parameter_events.is_null() {
        return None;
    }
    // SAFETY: the host provides exactly `parameter_event_count` events for the call.
    Some(unsafe {
        slice::from_raw_parts(block.parameter_events, block.parameter_event_count as usize)
    })
}

fn events_are_ordered(events: &[MidiEventV1], frames: u32) -> bool {
    events
        .iter()
        .all(|event| event.frame < frames && (1..=3).contains(&event.length))
        && events.windows(2).all(|pair| pair[0].frame <= pair[1].frame)
}

fn parameters_are_ordered(events: &[ParameterEventV1], frames: u32) -> bool {
    events
        .iter()
        .all(|event| event.frame < frames && event.value.is_finite())
        && events.windows(2).all(|pair| pair[0].frame <= pair[1].frame)
}

static PLUGIN_API: PluginApiV1 = PluginApiV1 {
    struct_size: size_of::<PluginApiV1>() as u32,
    api_version: ABI_VERSION,
    runtime_descriptor_json: write_runtime_descriptor,
    parameter_schema_json: write_parameter_schema,
    preset_catalog_json: write_preset_catalog,
    create,
    destroy,
    activate,
    deactivate,
    reset,
    set_parameter,
    get_parameter,
    save_state,
    load_state,
    load_preset,
    process,
};

#[unsafe(no_mangle)]
pub extern "C" fn rackforge_plugin_entry_v1() -> *const PluginApiV1 {
    ptr::addr_of!(PLUGIN_API)
}
