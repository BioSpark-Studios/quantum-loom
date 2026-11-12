//! The validation laws. Lore parameters are strict: they are checked, not
//! tuned for aesthetics.

use crate::manifest::{CapacityMetadata, FactionCrest, GrowthMode, HeraldicPosition};

/// Octave Capacity Law. A full container grows one octave if it is dynamic,
/// and refuses the insert if it is fixed.
pub fn assert_capacity(meta: &mut CapacityMetadata) -> Result<(), &'static str> {
    let limit = 2u32.pow(meta.current_octave);
    if meta.child_count < limit {
        return Ok(());
    }
    match meta.growth_mode {
        GrowthMode::Dynamic => {
            meta.current_octave += 1;
            meta.max_children_current = 2u32.pow(meta.current_octave);
            Ok(())
        }
        GrowthMode::Fixed => Err("Capacity Violation: Container is full at fixed octave ceiling."),
    }
}

/// Three-way alignment at birth: structural level, role and symbolic type agree.
/// 0 Genesis → Seal · 1 Mythos → Crest · 2 Container → Glyph/Device/Emblem ·
/// 3 Capsule → Trait/Mark/Token/Sigil.
pub fn validate_three_way_alignment(
    level: u32,
    birth: HeraldicPosition,
) -> Result<(), &'static str> {
    use HeraldicPosition::*;
    match (level, birth) {
        (0, Seal) | (1, Crest) | (2, Glyph | Device | Emblem) | (3, Trait | Mark | Token | Sigil) => Ok(()),
        _ => Err("Three-Way Alignment Failure: Structural depth level mismatches the symbolic birth authority!"),
    }
}

/// Crest Border Weight Law: a layer's border weight must be its faction's.
pub fn validate_crest_lore(crest: FactionCrest, border_weight: f32) -> Result<(), &'static str> {
    if (border_weight - crest.border_weight()).abs() > f32::EPSILON {
        Err("Lore Violation: Border weight does not align with canonical faction specifications.")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The checks `main()` ran in the original `qgcp-manifest-builder.rs`.
    #[test]
    fn original_source_checks_pass() {
        assert!(validate_crest_lore(FactionCrest::Syntaran, 2.0).is_ok());
        assert!(validate_crest_lore(FactionCrest::Luminarite, 1.5).is_err());
        assert!(validate_three_way_alignment(3, HeraldicPosition::Sigil).is_ok());
        assert!(validate_three_way_alignment(3, HeraldicPosition::Crest).is_err());
        let mut c = CapacityMetadata {
            growth_mode: GrowthMode::Dynamic,
            current_octave: 4,
            max_children_current: 16,
            child_count: 16,
        };
        assert!(assert_capacity(&mut c).is_ok());
        assert_eq!((c.current_octave, c.max_children_current), (5, 32));
    }

    #[test]
    fn fixed_containers_refuse_overflow_and_dynamic_ones_keep_growing() {
        let mut f = CapacityMetadata {
            growth_mode: GrowthMode::Fixed,
            current_octave: 2,
            max_children_current: 4,
            child_count: 4,
        };
        assert!(assert_capacity(&mut f).is_err());
        f.child_count = 3;
        assert!(assert_capacity(&mut f).is_ok());
        let mut d = CapacityMetadata {
            growth_mode: GrowthMode::Dynamic,
            current_octave: 1,
            max_children_current: 2,
            child_count: 2,
        };
        for n in 0..3 {
            d.child_count = d.max_children_current;
            assert!(assert_capacity(&mut d).is_ok(), "{n}");
        }
        assert_eq!(d.current_octave, 4);
    }

    #[test]
    fn every_level_has_exactly_its_birth_types() {
        use HeraldicPosition::*;
        let all = [
            Seal, Crest, Glyph, Device, Emblem, Sigil, Trait, Mark, Token, Trace,
        ];
        let ok = |l| {
            all.iter()
                .filter(|p| validate_three_way_alignment(l, **p).is_ok())
                .count()
        };
        assert_eq!([ok(0), ok(1), ok(2), ok(3), ok(4)], [1, 1, 3, 4, 0]);
    }

    #[test]
    fn border_weights_are_ordered_by_stratum() {
        use FactionCrest::*;
        let order = [Luminarite, Venturan, Sylvanid, Hydralis, Syntaran];
        assert!(order
            .windows(2)
            .all(|w| w[0].border_weight() < w[1].border_weight()));
        assert!(order.windows(2).all(|w| w[0].stratum() > w[1].stratum()));
    }
}
