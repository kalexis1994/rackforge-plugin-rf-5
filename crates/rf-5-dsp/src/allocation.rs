//! Prophet-5 Rev 3 polyphonic voice assignment.
//!
//! The technical manual specifies a physical five-slot queue: the first five
//! distinct notes take voices 1 through 5, then the least-recently-used voice
//! is reassigned. Repeating a note uses the same physical voice and moves it
//! to the newest end of the queue.
//!
//! A voice whose key has come up is taken before one whose key is still
//! down. No manual says so, and that absence is the whole of the evidence
//! either way, so it is worth writing down what was checked.
//!
//! The owner's manual, the Rev 3 service manual and Sequential's 2020
//! reissue manual all describe one case and the same one: more than five
//! keys held AT ONCE. "If more than five keys are held down at the same
//! time, the computer will reassign the earliest used voices first." The
//! service manual adds that the stolen voice's "pitch disappears even though
//! the key may still be held", which says holding does not protect a voice
//! -- not that releasing fails to free one. None of them describes a key
//! that came up.
//!
//! Against that silence: a five-voice instrument on which holding a triad
//! and playing a melody over it destroys the triad after two notes would be
//! a famous complaint, and the Prophet-5 was played exactly that way for
//! decades. A quirk that severe would be documented somewhere, and it is
//! documented nowhere.
//!
//! This was also, until it was fixed, not a modelled behaviour at all:
//! `note_off` released the voice and never told the assigner, so a voice
//! kept its place in the queue as though its key were still down. That is a
//! missing call rather than a decision, which is the strongest argument of
//! the three.
//!
//! What would settle it is a Prophet-5 playing that gesture. Until then this
//! is the reading, and reverting it is one commit.

use crate::VOICE_COUNT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Assignment {
    channel: u8,
    note: u8,
    /// Whether the key that took this voice has since come up.
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

    pub(crate) fn assign(&mut self, channel: u8, note: u8) -> usize {
        // A repeated pitch keeps its physical voice whether or not that
        // voice is still sounding, so the comparison is on channel and note
        // alone: a pitch struck again after its key came up is the same note
        // to the assigner. That is what the manual means by "repeated notes
        // key the same voice".
        let voice = self
            .assignments
            .iter()
            .position(|assignment| {
                assignment.is_some_and(|held| held.channel == channel && held.note == note)
            })
            // The queue is oldest-first, so the earliest entry that is free
            // or already released is the one to take. With every key down
            // there is no such entry and this falls through to the queue's
            // head, which is the documented behaviour, note for note.
            .or_else(|| {
                self.oldest_to_newest
                    .iter()
                    .copied()
                    .find(|voice| self.assignments[*voice].is_none_or(|held| held.released))
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
    /// Called from the one place a voice enters its release, so a key up,
    /// the sustain pedal rising and all-notes-off are all covered without
    /// any of them having to remember to.
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

    /// A chord survives a melody played over it and let go.
    ///
    /// This is the case no manual describes and the reason the assigner
    /// learns about releases at all. Without `mark_released` the three notes
    /// of the chord are the oldest entries in the queue and go first.
    #[test]
    fn a_released_voice_is_taken_before_a_held_one() {
        let mut allocator = PolyAllocator::default();
        let chord: [usize; 3] =
            core::array::from_fn(|index| allocator.assign(0, [60, 64, 67][index]));

        for note in [72_u8, 74, 76, 77, 79] {
            let taken = allocator.assign(0, note);
            assert!(
                !chord.contains(&taken),
                "la nota {note} se llevo la voz {taken}, que era del acorde"
            );
            allocator.mark_released(taken);
        }
    }

    /// With every key still down there is nothing released to prefer, so
    /// this is the documented behaviour unchanged.
    #[test]
    fn with_every_key_held_it_is_the_documented_queue() {
        let mut allocator = PolyAllocator::default();
        for note in 60..65 {
            let _ = allocator.assign(0, note);
        }
        assert_eq!(allocator.assign(0, 65), 0);
        assert_eq!(allocator.assign(0, 66), 1);
        assert_eq!(allocator.assign(0, 67), 2);
    }

    /// A pitch struck again after its key came up keeps its own voice rather
    /// than counting as a new note.
    #[test]
    fn a_released_pitch_struck_again_keeps_its_voice() {
        let mut allocator = PolyAllocator::default();
        let first = allocator.assign(0, 60);
        allocator.mark_released(first);
        assert_eq!(allocator.assign(0, 60), first);
    }

    #[test]
    fn midi_channels_are_part_of_note_identity() {
        let mut allocator = PolyAllocator::default();
        assert_eq!(allocator.assign(0, 60), 0);
        assert_eq!(allocator.assign(1, 60), 1);
        assert_eq!(allocator.assign(0, 60), 0);
    }
}
