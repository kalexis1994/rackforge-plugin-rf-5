#[derive(Clone, Copy, Debug)]
pub struct ControlGroup {
    pub title: &'static str,
    pub parameter_ids: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub struct PanelSection {
    pub id: &'static str,
    pub label: &'static str,
    pub caption: &'static str,
    pub groups: &'static [ControlGroup],
}

const POLY_MOD: ControlGroup = ControlGroup {
    title: "POLY-MOD",
    parameter_ids: &[
        "poly-mod-filter-envelope-amount",
        "poly-mod-oscillator-b-amount",
        "poly-mod-oscillator-a-frequency",
        "poly-mod-oscillator-a-pulse-width",
        "poly-mod-filter",
    ],
};
const LFO: ControlGroup = ControlGroup {
    title: "LFO",
    parameter_ids: &["lfo-frequency", "lfo-saw", "lfo-triangle", "lfo-square"],
};
const WHEEL_MOD: ControlGroup = ControlGroup {
    title: "WHEEL-MOD",
    parameter_ids: &[
        "wheel-mod-source-mix",
        "wheel-mod-oscillator-a-frequency",
        "wheel-mod-oscillator-b-frequency",
        "wheel-mod-oscillator-a-pulse-width",
        "wheel-mod-oscillator-b-pulse-width",
        "wheel-mod-filter",
    ],
};
const OSCILLATOR_A: ControlGroup = ControlGroup {
    title: "OSCILLATOR A",
    parameter_ids: &[
        "oscillator-a-frequency",
        "oscillator-a-saw",
        "oscillator-a-pulse",
        "oscillator-a-pulse-width",
        "oscillator-sync",
    ],
};
const OSCILLATOR_B: ControlGroup = ControlGroup {
    title: "OSCILLATOR B",
    parameter_ids: &[
        "oscillator-b-frequency",
        "oscillator-b-detune",
        "oscillator-b-saw",
        "oscillator-b-triangle",
        "oscillator-b-pulse",
        "oscillator-b-pulse-width",
        "oscillator-b-low-frequency",
        "oscillator-b-keyboard",
    ],
};
const MIXER: ControlGroup = ControlGroup {
    title: "MIXER",
    parameter_ids: &["oscillator-a-level", "oscillator-b-level", "noise-level"],
};
const FILTER: ControlGroup = ControlGroup {
    title: "FILTER",
    parameter_ids: &[
        "filter-cutoff",
        "filter-resonance",
        "filter-envelope-amount",
        "filter-keyboard",
    ],
};
const FILTER_ENVELOPE: ControlGroup = ControlGroup {
    title: "FILTER ENVELOPE",
    parameter_ids: &[
        "filter-attack",
        "filter-decay",
        "filter-sustain",
        "filter-release",
    ],
};
const AMPLIFIER: ControlGroup = ControlGroup {
    title: "AMPLIFIER",
    parameter_ids: &["amp-attack", "amp-decay", "amp-sustain", "amp-release"],
};
const PERFORMANCE: ControlGroup = ControlGroup {
    title: "PERFORMANCE",
    parameter_ids: &["glide", "unison", "release-enable"],
};
const OUTPUT: ControlGroup = ControlGroup {
    title: "OUTPUT",
    parameter_ids: &["master-tune", "master-volume", "vintage-spread"],
};
const SCALE: ControlGroup = ControlGroup {
    title: "SCALE MODE",
    parameter_ids: &[
        "scale-c",
        "scale-c-sharp",
        "scale-d",
        "scale-d-sharp",
        "scale-e",
        "scale-f",
        "scale-f-sharp",
        "scale-g",
        "scale-g-sharp",
        "scale-a",
        "scale-a-sharp",
        "scale-b",
    ],
};

const MODULATION_GROUPS: &[ControlGroup] = &[POLY_MOD, LFO, WHEEL_MOD];
const OSCILLATOR_GROUPS: &[ControlGroup] = &[OSCILLATOR_A, OSCILLATOR_B, MIXER];
const FILTER_GROUPS: &[ControlGroup] = &[FILTER, FILTER_ENVELOPE, AMPLIFIER];
const VOICE_GROUPS: &[ControlGroup] = &[PERFORMANCE, OUTPUT];
const SCALE_GROUPS: &[ControlGroup] = &[SCALE];

pub const SECTIONS: &[PanelSection] = &[
    PanelSection {
        id: "modulation",
        label: "MODULATION",
        caption: "POLY-MOD · LFO · WHEEL-MOD",
        groups: MODULATION_GROUPS,
    },
    PanelSection {
        id: "oscillators",
        label: "OSCILLATORS",
        caption: "OSCILLATOR A · OSCILLATOR B · MIXER",
        groups: OSCILLATOR_GROUPS,
    },
    PanelSection {
        id: "filter",
        label: "FILTER + ENVELOPES",
        caption: "FILTER · FILTER ENV · AMPLIFIER",
        groups: FILTER_GROUPS,
    },
    PanelSection {
        id: "voice",
        label: "VOICE",
        caption: "GLIDE · UNISON · OUTPUT",
        groups: VOICE_GROUPS,
    },
    PanelSection {
        id: "scale",
        label: "SCALE MODE",
        caption: "TWELVE PROGRAMMABLE NOTE OFFSETS",
        groups: SCALE_GROUPS,
    },
];

pub fn section(id: &str) -> &'static PanelSection {
    SECTIONS
        .iter()
        .find(|section| section.id == id)
        .unwrap_or(&SECTIONS[0])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn panel_maps_every_public_parameter_exactly_once() {
        let mut ids = BTreeSet::new();
        for section in SECTIONS {
            for group in section.groups {
                for id in group.parameter_ids {
                    assert!(ids.insert(*id), "duplicate panel parameter {id}");
                }
            }
        }
        assert_eq!(ids.len(), rf_5_contract::PARAMETER_COUNT);

        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../plugin/package/metadata/parameters.json"
        ))
        .unwrap();
        let schema_ids = schema["parameters"]
            .as_array()
            .unwrap()
            .iter()
            .map(|parameter| parameter["id"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids, schema_ids);
    }

    #[test]
    fn section_ids_are_stable_and_unique() {
        let ids = SECTIONS
            .iter()
            .map(|section| section.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), SECTIONS.len());
        assert!(SECTIONS.iter().all(|section| {
            !section.label.is_empty()
                && !section.caption.is_empty()
                && section.groups.iter().all(|group| !group.title.is_empty())
        }));
        assert_eq!(section("missing").id, "modulation");
    }
}
