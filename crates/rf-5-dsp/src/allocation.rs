//! Prophet-5 Rev 3 polyphonic voice assignment.
//!
//! The technical manual specifies a physical five-slot queue: the first five
//! distinct notes take voices 1 through 5, then the least-recently-used voice
//! is reassigned. Repeating a note uses the same physical voice and moves it
//! to the newest end of the queue.

use crate::VOICE_COUNT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Assignment {
    channel: u8,
    note: u8,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PolyAllocator {
    oldest_to_newest: [usize; VOICE_COUNT],
    assignments: [Option<Assignment>; VOICE_COUNT],
}

impl Default for PolyAllocator {
    fn default() -> Self {
        Self {
            oldest_to_newest: core::array::from_fn(|index| index),
            assignments: [None; VOICE_COUNT],
        }
    }
}

impl PolyAllocator {
    pub(crate) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(crate) fn assign(&mut self, channel: u8, note: u8) -> usize {
        let requested = Assignment { channel, note };
        let voice = self
            .assignments
            .iter()
            .position(|assignment| *assignment == Some(requested))
            .unwrap_or(self.oldest_to_newest[0]);
        self.assignments[voice] = Some(requested);
        self.mark_newest(voice);
        voice
    }

    fn mark_newest(&mut self, voice: usize) {
        let position = self
            .oldest_to_newest
            .iter()
            .position(|candidate| *candidate == voice)
            .expect("every physical voice remains in the assignment queue");
        self.oldest_to_newest
            .copy_within(position + 1..VOICE_COUNT, position);
        self.oldest_to_newest[VOICE_COUNT - 1] = voice;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_five_notes_take_physical_voices_in_order() {
        let mut allocator = PolyAllocator::default();
        let assigned = core::array::from_fn(|offset| allocator.assign(0, 60 + offset as u8));
        assert_eq!(assigned, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn notes_after_the_first_five_steal_the_earliest_used_voice() {
        let mut allocator = PolyAllocator::default();
        for note in 60..65 {
            let _ = allocator.assign(0, note);
        }
        assert_eq!(allocator.assign(0, 65), 0);
        assert_eq!(allocator.assign(0, 66), 1);
        assert_eq!(allocator.assign(0, 67), 2);
    }

    #[test]
    fn a_repeated_note_keeps_its_voice_and_becomes_the_newest() {
        let mut allocator = PolyAllocator::default();
        assert_eq!(allocator.assign(0, 60), 0);
        assert_eq!(allocator.assign(0, 62), 1);
        assert_eq!(allocator.assign(0, 60), 0);

        assert_eq!(allocator.assign(0, 64), 2);
        assert_eq!(allocator.assign(0, 65), 3);
        assert_eq!(allocator.assign(0, 67), 4);
        assert_eq!(allocator.assign(0, 69), 1);
    }

    #[test]
    fn stealing_replaces_the_old_pitch_identity() {
        let mut allocator = PolyAllocator::default();
        for note in 60..66 {
            let _ = allocator.assign(0, note);
        }
        assert_eq!(allocator.assign(0, 60), 1);
    }

    #[test]
    fn midi_channels_are_part_of_note_identity() {
        let mut allocator = PolyAllocator::default();
        assert_eq!(allocator.assign(0, 60), 0);
        assert_eq!(allocator.assign(1, 60), 1);
        assert_eq!(allocator.assign(0, 60), 0);
    }
}
