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
        let input = format!(
            "https://fumen.zui.jp/?{prefix}115@ghE8?JeAgH\n \u{b}\u{85}\u{2003}vhAAAA&unused=1"
        );
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
    // Character validation still precedes parsing, and offsets count UTF-8 bytes.
    assert_eq!(
        rumen::parse("v115@\u{2003}AAé"),
        Err(DecodeError::InvalidCharacter {
            character: 'é',
            byte_offset: 10
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

    // tetris-fumen@1.1.3: quiz -> ordinary -> empty -> quiz, each with inheritance.
    let reset = rumen::parse(
        "v115@vhIAglAAtVAFLDmClcJSAVDEHBEooRBJoAVB0AAAAA?AeAAtIAP8y4DuCSvEAAeAAtAAAAeAAtVAFLDmClcJSAVDEH?BEooRBUoAVBpAAAAAAe",
    )
    .unwrap();
    let expected = [
        ("", false),
        ("#Q=[](I)T", true),
        ("#Q=[](I)T", false),
        ("ordinary", true),
        ("ordinary", false),
        ("", true),
        ("", false),
        ("#Q=[](T)I", true),
        ("#Q=[](T)I", false),
    ];
    assert_eq!(reset.pages.len(), expected.len());
    for (page, (notes, updated)) in reset.pages.iter().zip(expected) {
        assert_eq!(page.notes, notes);
        assert_eq!(page.flag.note, updated);
    }
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
    for input in [
        "v115@D8KeE8bhJiHvhAAAA",
        // An unchanged field run must detect undefined cells before reading its
        // repeat counter or the following (truncated) action.
        "v115@D8KeE8bhJiHvh",
        "v115@D8KeE8bhJiHvhA",
    ] {
        assert_eq!(
            rumen::parse(input),
            Err(DecodeError::UndefinedCell { index: 0 }),
            "{input}"
        );
    }
}

#[test]
fn nonadjacent_line_clears_preserve_row_order_and_garbage() {
    // tetris-fumen@1.1.3: full rows at bottom-up y=0 and y=2, with
    // distinct markers below, between, and above them (including the top row).
    let fumen = rumen::parse("v115@g02gwwOeJ8whHeglJ8IeQ4AgHvhAAAA").unwrap();
    let mut expected = [[Block::Empty; 10]; 24];
    expected[2][0] = Block::J;
    expected[20][4] = Block::T;
    expected[22][0] = Block::I;
    expected[22][9] = Block::L;
    expected[23][9] = Block::S;
    assert_eq!(fumen.pages[1].field, expected);
}
