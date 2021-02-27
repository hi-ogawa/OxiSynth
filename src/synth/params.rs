use crate::{Status, Synth};

use crate::engine::channel::InterpMethod;

/**
Synthesis parameters
 */
impl Synth {
    /**
    Set the master gain
     */
    pub fn set_gain(&mut self, gain: f64) {
        self.handle.set_gain(gain)
    }

    /**
    Get the master gain
     */
    pub fn get_gain(&self) -> f64 {
        self.handle.get_gain()
    }

    /**
    Set the polyphony limit (FluidSynth >= 1.0.6)
     */
    pub fn set_polyphony(&mut self, polyphony: u32) -> Status {
        Synth::zero_ok(self.handle.set_polyphony(polyphony as _))
    }

    /**
    Get the polyphony limit (FluidSynth >= 1.0.6)
     */
    pub fn get_polyphony(&self) -> u32 {
        self.handle.get_polyphony()
    }

    /**
    Get the internal buffer size. The internal buffer size if not the
    same thing as the buffer size specified in the
    settings. Internally, the synth *always* uses a specific buffer
    size independent of the buffer size used by the audio driver. The
    internal buffer size is normally 64 samples. The reason why it
    uses an internal buffer size is to allow audio drivers to call the
    synthesizer with a variable buffer length. The internal buffer
    size is useful for client who want to optimize their buffer sizes.
     */
    pub fn get_internal_buffer_size(&self) -> usize {
        self.handle.get_internal_bufsize()
    }

    /** Set the interpolation method for one channel (`Some(chan)`) or all channels (`None`) */
    pub fn set_interp_method(&mut self, chan: Option<u8>, interp_method: InterpMethod) {
        self.handle.set_interp_method(chan, interp_method)
    }
}
