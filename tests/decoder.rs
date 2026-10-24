use rumen::{Block, DecodeError, Fumen, ViewMode};
use serde::Deserialize;

#[derive(Deserialize)]
struct Corpus {
    reference: String,
    unpaired_surrogate: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    fumen: String,
    pages: Vec<ExpectedPage>,
}

#[derive(Deserialize)]
struct ExpectedPage {
    field: [String; 24],
    operation: Option<ExpectedOperation>,
    comment: String,
    flags: ExpectedFlags,
}

#[derive(Deserialize)]
struct ExpectedOperation {
    block: String,
    facing: String,
    x: u8,
    y: u8,
    positions: [[i16; 2]; 4],
}

#[derive(Deserialize)]
struct ExpectedFlags {
    up: bool,
    mirror: bool,
    color_scheme: String,
    lock: bool,
}

#[test]
fn matches_published_reference_page_by_page() {
    let corpus: Corpus = serde_json::from_str(include_str!("fixtures/v115.json")).unwrap();
    assert_eq!(corpus.reference, "tetris-fumen@1.1.3");
    for case in corpus.cases {
        let fumen =
            rumen::parse(&case.fumen).unwrap_or_else(|error| panic!("{}: {error}", case.name));
        assert_eq!(fumen.pages.len(), case.pages.len(), "{}", case.name);
        for (index, (actual, expected)) in fumen.pages.iter().zip(case.pages).enumerate() {
            let context = format!("{} page {index}", case.name);
            for (row, expected_row) in actual.field.iter().zip(expected.field) {
                let symbols: String = row
                    .iter()
                    .map(|block| b"_ILOZTJSX"[*block as usize] as char)
                    .collect();
                assert_eq!(symbols, expected_row, "{context}: field");
            }
            assert_eq!(actual.notes, expected.comment, "{context}: comment");
            assert_eq!(actual.flag.up, expected.flags.up, "{context}: rise");
            assert_eq!(
                actual.flag.mirror, expected.flags.mirror,
                "{context}: mirror"
            );
            assert_eq!(actual.flag.lock, expected.flags.lock, "{context}: lock");
            assert_eq!(
                format!("{:?}", actual.flag.color_scheme),
                expected.flags.color_scheme,
                "{context}: color"
            );
            match expected.operation {
                Some(operation) => {
                    assert_eq!(
                        format!("{:?}", actual.flag.mino),
                        operation.block,
                        "{context}: mino"
                    );
                    assert_eq!(
                        format!("{:?}", actual.flag.face.unwrap()),
                        operation.facing,
                        "{context}: facing"
                    );
                    let location = actual.flag.location.unwrap();
                    assert_eq!(
                        (location.x(), location.y()),
                        (operation.x, operation.y),
                        "{context}: wire location"
                    );
                    let mut positions = actual.flag.positions().unwrap();
                    positions.sort_unstable();
                    let mut expected_positions = operation.positions.map(|[x, y]| (x, y));
                    expected_positions.sort_unstable();
                    assert_eq!(positions, expected_positions, "{context}: mino cells");
                }
                None => {
                    assert_eq!(actual.flag.mino, Block::Empty, "{context}: no active mino");
                    assert_eq!(actual.flag.face, None, "{context}");
                    assert_eq!(actual.flag.location, None, "{context}");
                    assert_eq!(actual.flag.positions(), None, "{context}");
                }
            }
        }
    }
}

#[test]
fn accepts_viewer_urls_separators_and_all_view_modes() {
    let normal = rumen::parse("v115@ghE8JeAgHvhAAAA").unwrap();
    for (prefix, mode) in [
        ('v', ViewMode::Normal),
        ('m', ViewMode::Mini),
        ('d', ViewMode::List),
        ('D', ViewMode::MiniList),
    ] {
        let input = format!("https://fumen.zui.jp/?{prefix}115@ghE8?JeAgH\n vhAAAA&unused=1");
        let fumen: Fumen = input.parse().unwrap();
        assert_eq!(fumen.mode, mode);
        assert_eq!(fumen.version, 115);
        assert_eq!(fumen.pages, normal.pages);
    }
    assert!(rumen::parse("v115@").unwrap().pages.is_empty());
}

#[test]
fn rejects_invalid_headers_and_unsupported_versions() {
    for input in ["", "v115!vhAAAA", "vabc@vhAAAA", "x115@vhAAAA"] {
        assert_eq!(
            rumen::parse(input),
            Err(DecodeError::InvalidHeader),
            "{input}"
        );
    }
    assert_eq!(
        rumen::parse("v110@vhAAAA"),
        Err(DecodeError::UnsupportedVersion(110))
    );
    assert_eq!(
        rumen::parse("v115@vhA*AA"),
        Err(DecodeError::InvalidCharacter {
            character: '*',
            byte_offset: 8
        })
    );
}

#[test]
fn truncated_fields_actions_and_comments_return_errors() {
    let complete = "v115@ghE8JeAgWDABUYCA";
    assert!(rumen::parse(complete).is_ok());
    for end in 6..complete.len() {
        assert!(
            matches!(
                rumen::parse(&complete[..end]),
                Err(DecodeError::UnexpectedEnd { .. })
            ),
            "prefix {end}: {}",
            &complete[..end]
        );
    }
    assert!(matches!(
        rumen::parse("v115@vhBAAA"),
        Err(DecodeError::UnexpectedEnd { .. })
    ));
}

#[test]
fn malformed_field_data_returns_errors_instead_of_overflowing() {
    assert_eq!(
        rumen::parse("v115@ghvh"),
        Err(DecodeError::InvalidFieldRun {
            index: 225,
            count: 240
        })
    );
    assert_eq!(
        rumen::parse("v115@AA"),
        Err(DecodeError::InvalidBlock {
            index: 0,
            value: -8
        })
    );
    assert_eq!(
        rumen::parse("v115@//"),
        Err(DecodeError::InvalidBlock { index: 0, value: 9 })
    );
}

#[test]
fn explicit_empty_comment_is_distinct_from_no_comment_update() {
    let absent = rumen::parse("v115@vhAAgH").unwrap();
    let empty = rumen::parse("v115@vhAAgWAA").unwrap();
    assert!(!absent.pages[0].flag.note);
    assert!(empty.pages[0].flag.note);
    assert_eq!(absent.pages[0].notes, "");
    assert_eq!(empty.pages[0].notes, "");
}

#[test]
fn unrepresentable_comment_text_returns_an_explicit_error() {
    let corpus: Corpus = serde_json::from_str(include_str!("fixtures/v115.json")).unwrap();
    assert_eq!(
        rumen::parse(&corpus.unpaired_surrogate),
        Err(DecodeError::InvalidUnicode)
    );
    assert_eq!(
        rumen::parse("v115@vhAAgWBAfBAAA"),
        Err(DecodeError::InvalidCommentDigit(95))
    );
}

#[test]
fn undefined_field_cells_are_reported_not_invented() {
    // The reference exposes NaN here after an extended-array line clear.
    assert_eq!(
        rumen::parse("v115@D8KeE8bhJiHvhAAAA"),
        Err(DecodeError::UndefinedCell { index: 0 })
    );
}
