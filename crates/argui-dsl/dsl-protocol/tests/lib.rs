use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_protocol::{
    CompatibilityError, LiveMessage, LivePackageEnvelope, PackageHeader, RuntimeVersions, decode,
    encode, read_frame, write_frame,
};

fn package() -> LivePackageEnvelope {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Text } from "@argui/ui"
export component Main { in property title: string Text { content: title } }"#,
        )],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    LivePackageEnvelope {
        header: PackageHeader::current(compiled.public_api_hash, 7),
        roots: compiled.roots,
        ir: compiled.ir,
        assets: Vec::new(),
    }
}

#[test]
fn validated_ir_round_trips_without_semantic_loss() {
    let message = LiveMessage::Package(Box::new(package()));
    let decoded: LiveMessage = decode(&encode(&message).unwrap()).unwrap();
    assert_eq!(decoded, message);

    let mut framed = Vec::new();
    write_frame(&mut framed, &message).unwrap();
    assert_eq!(
        read_frame::<LiveMessage>(&mut framed.as_slice()).unwrap(),
        message
    );
}

#[test]
fn every_compatibility_dimension_is_checked_exactly() {
    let versions = RuntimeVersions::current();
    let current = package().header;
    assert_eq!(versions.check(&current), Ok(()));

    let mut wrong = current.clone();
    wrong.protocol_version += 1;
    assert!(matches!(
        versions.check(&wrong),
        Err(CompatibilityError::Protocol { .. })
    ));
    let mut wrong = current.clone();
    wrong.ir_format_version += 1;
    assert!(matches!(
        versions.check(&wrong),
        Err(CompatibilityError::Ir { .. })
    ));
    let mut wrong = current;
    wrong.engine_version.push_str("-different");
    assert!(matches!(
        versions.check(&wrong),
        Err(CompatibilityError::Engine { .. })
    ));
}
