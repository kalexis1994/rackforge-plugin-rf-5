//! Prophet-5 Rev 3 polyphonic voice assignment.
//!
//! The technical manual specifies a physical five-slot queue: the first five
//! distinct notes take voices 1 through 5, then the least-recently-used voice
//! is reassigned. Repeating a note uses the same physical voice and moves it
//! to the newest end of the queue.
//!
//! What no manual says is what a key RELEASE does to that queue, and the
//! answer is audible: hold a three-note chord, play a few notes over it and
//! let them go, and the next one takes the chord rather than one of the
//! voices already decaying. The owner's manual, the Rev 3 service manual and
//! Sequential's own 2020 manual all describe the same case -- more than five
//! keys held at once -- and none of them describes this one. The service
//! manual does say the stolen voice's "pitch disappears even though the key
//! may still be held", which reads as an assigner that never consults the
//! keyboard at all, and that is what [`VoiceAllocation::Original`] is.
//!
//! It is also, most likely, why Sequential added a Round Robin allocation
//! mode to the Rev 4 in 2023 instead of changing what the original did. This
//! module follows that shape exactly: the original is the default and is
//! untouched, and [`VoiceAllocation::ReleasedFirst`] is there for a player
//! who would rather keep the chord.

use crate::VOICE_COUNT;

/// Which voice the assigner takes when a new note arrives and none is free.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum VoiceAllocation {
    /// The Rev 3 assigner: the earliest-assigned voice, whether or not its
    /// key is still down.
    #[default]
    Original,
    /// The earliest voice whose key has already been let go; a held voice
    /// only when every voice is held.
    ReleasedFirst,
}

impl VoiceAllocation {
    /// Reads the global parameter, which is normalized like every other.
    pub(crate) fn from_normalized(value: f32) -> Self {
        if value >= 0.5 {
            Self::ReleasedFirst
        } else {
            Self::Original
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Assignment {
    channel: u8,
    note: u8,
    /// Whether the key that took this voice has since been let go.
    ///
    /// Only [`VoiceAllocation::ReleasedFirst`] reads it. It is maintained
    /// either way, so switching modes between two notes cannot observe a
    /// stale flag and needs no rebuild.
    released: bool,
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

    pub(crate) fn assign(&mut self, channel: u8, note: u8, mode: VoiceAllocation) -> usize {
        // A repeated pitch keeps its physical voice, in both modes and
        // whether or not that voice is still sounding. The comparison is on
        // channel and note alone rather than the whole assignment: a pitch
        // struck again after its key came up is the same note to the
        // assigner, which is what "repeated notes key the same voice" means.
        let same_pitch = self.assignments.iter().position(|assignment| {
            assignment.is_some_and(|held| held.channel == channel && held.note == note)
        });
        let voice = same_pitch
            .or_else(|| match mode {
                VoiceAllocation::Original => None,
                // The queue is already oldest-first, so the first entry that
                // is free or released is the earliest such voice.
                VoiceAllocation::ReleasedFirst => self
                    .oldest_to_newest
                    .iter()
                    .copied()
                    .find(|voice| self.assignments[*voice].is_none_or(|held| held.released)),
            })
            .unwrap_or(self.oldest_to_newest[0]);
        self.assignments[voice] = Some(Assignment {
            channel,
            note,
            released: false,
        });
        self.mark_newest(voice);
        voice
    }

    /// Records that a voice's key came up.
    ///
    /// Called from the one place a voice enters its release, so every path
    /// that reaches it -- a key up, the sustain pedal rising, all-notes-off
    /// -- is covered without any of them having to remember to.
    pub(crate) fn mark_released(&mut self, voice: usize) {
        if let Some(assignment) = self.assignments.get_mut(voice).and_then(Option::as_mut) {
            assignment.released = true;
        }
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

    /// Every test that existed before the mode did drives this one, so the
    /// default's behaviour is still held to exactly what it was.
    const ORIGINAL: VoiceAllocation = VoiceAllocation::Original;

    #[test]
    fn first_five_notes_take_physical_voices_in_order() {
        let mut allocator = PolyAllocator::default();
        let assigned =
            core::array::from_fn(|offset| allocator.assign(0, 60 + offset as u8, ORIGINAL));
        assert_eq!(assigned, [0, 1, 2, 3, 4]);
    }

    #[test]
    fn notes_after_the_first_five_steal_the_earliest_used_voice() {
        let mut allocator = PolyAllocator::default();
        for note in 60..65 {
            let _ = allocator.assign(0, note, ORIGINAL);
        }
        assert_eq!(allocator.assign(0, 65, ORIGINAL), 0);
        assert_eq!(allocator.assign(0, 66, ORIGINAL), 1);
        assert_eq!(allocator.assign(0, 67, ORIGINAL), 2);
    }

    #[test]
    fn a_repeated_note_keeps_its_voice_and_becomes_the_newest() {
        let mut allocator = PolyAllocator::default();
        assert_eq!(allocator.assign(0, 60, ORIGINAL), 0);
        assert_eq!(allocator.assign(0, 62, ORIGINAL), 1);
        assert_eq!(allocator.assign(0, 60, ORIGINAL), 0);

        assert_eq!(allocator.assign(0, 64, ORIGINAL), 2);
        assert_eq!(allocator.assign(0, 65, ORIGINAL), 3);
        assert_eq!(allocator.assign(0, 67, ORIGINAL), 4);
        assert_eq!(allocator.assign(0, 69, ORIGINAL), 1);
    }

    #[test]
    fn stealing_replaces_the_old_pitch_identity() {
        let mut allocator = PolyAllocator::default();
        for note in 60..66 {
            let _ = allocator.assign(0, note, ORIGINAL);
        }
        assert_eq!(allocator.assign(0, 60, ORIGINAL), 1);
    }

    #[test]
    fn midi_channels_are_part_of_note_identity() {
        let mut allocator = PolyAllocator::default();
        assert_eq!(allocator.assign(0, 60, ORIGINAL), 0);
        assert_eq!(allocator.assign(1, 60, ORIGINAL), 1);
        assert_eq!(allocator.assign(0, 60, ORIGINAL), 0);
    }

    /// Releasing a key changes nothing for the original assigner.
    ///
    /// This is the behaviour the default has to keep, and it is also the
    /// reason the alternative exists: the chord goes before a voice that is
    /// merely decaying.
    #[test]
    fn the_original_assigner_ignores_whether_a_key_came_up() {
        let mut allocator = PolyAllocator::default();
        for note in [60, 64, 67] {
            allocator.assign(0, note, ORIGINAL);
        }
        let fourth = allocator.assign(0, 72, ORIGINAL);
        allocator.mark_released(fourth);
        let fifth = allocator.assign(0, 74, ORIGINAL);
        allocator.mark_released(fifth);
        assert_eq!(
            allocator.assign(0, 76, ORIGINAL),
            0,
            "el original toma el acorde, que es lo que documenta el manual"
        );
    }

    /// The alternative recycles what is already decaying and leaves the
    /// chord alone.
    #[test]
    fn released_first_recycles_the_decaying_voices_and_leaves_the_chord() {
        const MODE: VoiceAllocation = VoiceAllocation::ReleasedFirst;
        let mut allocator = PolyAllocator::default();
        let chord: [usize; 3] =
            core::array::from_fn(|index| allocator.assign(0, [60, 64, 67][index], MODE));

        let mut taken = [0_usize; 5];
        for (slot, note) in taken.iter_mut().zip([72_u8, 74, 76, 77, 79]) {
            *slot = allocator.assign(0, note, MODE);
            allocator.mark_released(*slot);
        }
        for voice in taken {
            assert!(
                !chord.contains(&voice),
                "la voz {voice} era del acorde y fue robada"
            );
        }
    }

    /// With every key held there is nothing decaying to prefer, so the
    /// alternative is the original, note for note.
    #[test]
    fn released_first_steals_the_earliest_when_every_key_is_held() {
        let mut original = PolyAllocator::default();
        let mut alternative = PolyAllocator::default();
        for offset in 0..VOICE_COUNT as u8 + 3 {
            assert_eq!(
                original.assign(0, 60 + offset, VoiceAllocation::Original),
                alternative.assign(0, 60 + offset, VoiceAllocation::ReleasedFirst),
                "difieren en la nota {offset} con todas las teclas sostenidas"
            );
        }
    }

    /// A pitch struck again after its key came up keeps its own voice rather
    /// than counting as a new note, in both modes.
    #[test]
    fn a_released_pitch_struck_again_keeps_its_voice() {
        for mode in [VoiceAllocation::Original, VoiceAllocation::ReleasedFirst] {
            let mut allocator = PolyAllocator::default();
            let first = allocator.assign(0, 60, mode);
            allocator.mark_released(first);
            assert_eq!(allocator.assign(0, 60, mode), first, "modo {mode:?}");
        }
    }

    #[test]
    fn the_parameter_reads_as_the_original_below_half() {
        assert_eq!(
            VoiceAllocation::from_normalized(0.0),
            VoiceAllocation::Original
        );
        assert_eq!(
            VoiceAllocation::from_normalized(0.49),
            VoiceAllocation::Original
        );
        assert_eq!(
            VoiceAllocation::from_normalized(0.5),
            VoiceAllocation::ReleasedFirst
        );
        assert_eq!(
            VoiceAllocation::from_normalized(1.0),
            VoiceAllocation::ReleasedFirst
        );
    }
}
