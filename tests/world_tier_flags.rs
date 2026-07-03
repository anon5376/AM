use am001::world::theta::WorldTheta;

#[test]
fn reserved_world_tier_flags_are_rejected() {
    let mut theta = WorldTheta {
        twins: true,
        ..WorldTheta::default()
    };
    assert_unimplemented("twins", &theta);

    theta = WorldTheta {
        motion: true,
        ..WorldTheta::default()
    };
    assert_unimplemented("motion", &theta);

    theta = WorldTheta {
        confound: true,
        ..WorldTheta::default()
    };
    assert_unimplemented("confound", &theta);

    theta = WorldTheta {
        vision_radius: Some(3),
        ..WorldTheta::default()
    };
    assert_unimplemented("vision_radius", &theta);

    theta = WorldTheta {
        rule_resample: true,
        ..WorldTheta::default()
    };
    assert_unimplemented("rule_resample", &theta);
}

fn assert_unimplemented(flag: &str, theta: &WorldTheta) {
    let err = theta.validate().unwrap_err().to_string();
    assert!(err.contains("UnimplementedTier"), "{err}");
    assert!(err.contains(flag), "{err}");
}
