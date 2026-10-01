use engine::{
    Bitboard, Board, CastleSide, CastlingRights, Color, FenError, Piece, PieceType, Rank, Square,
};

const VALID: &str = include_str!("data/fen_valid.txt");
const INVALID: &str = include_str!("data/fen_invalid.txt");

/// The data lines of a test file, without comments and blank lines.
fn data_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn parse(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"))
}

/// Position of `error`'s variant in the enum: listing every variant here
/// makes adding one without an invalid FEN a compile error.
fn variant_index(error: FenError) -> usize {
    match error {
        FenError::FieldCount(_) => 0,
        FenError::Rank(_) => 1,
        FenError::UnknownPiece(_) => 2,
        FenError::SideToMove => 3,
        FenError::Castling => 4,
        FenError::EnPassant => 5,
        FenError::Counter => 6,
        FenError::KingCount => 7,
        FenError::PawnOnBackRank => 8,
        FenError::OpponentInCheck => 9,
    }
}

// Reading and writing

#[test]
fn round_trip() {
    for fen in data_lines(VALID) {
        assert_eq!(parse(fen).to_fen(), fen);
    }
}

#[test]
fn invalid_fens_give_the_expected_error() {
    let mut seen = [false; 10];
    for line in data_lines(INVALID) {
        let (fen, expected) = line.split_once('|').expect("FEN | error");
        let (fen, expected) = (fen.trim(), expected.trim());
        let error = Board::from_fen(fen).expect_err(fen);
        assert_eq!(format!("{error:?}"), expected, "{fen}");
        assert!(!error.to_string().is_empty());
        seen[variant_index(error)] = true;
    }
    assert!(
        seen.iter().all(|&s| s),
        "a FenError variant has no test: {seen:?}"
    );
}

#[test]
fn unplayable_en_passant_is_dropped() {
    let board = parse("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
    assert_eq!(board.en_passant(), None);
    assert_eq!(
        board.to_fen(),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
    );
}

#[test]
fn playable_en_passant_is_kept() {
    let board = parse("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    assert_eq!(board.en_passant(), Some(Square::F6));
}

#[test]
fn missing_counters_default() {
    let board = parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -");
    assert_eq!(board.halfmove_clock(), 0);
    assert_eq!(board.fullmove_number(), 1);
    assert_eq!(board, Board::startpos());

    let board = parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 7");
    assert_eq!(board.halfmove_clock(), 7);
    assert_eq!(board.fullmove_number(), 1);
}

#[test]
fn shredder_and_standard_castling_agree() {
    let shredder = parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1");
    assert_eq!(shredder, Board::startpos());
    assert_eq!(shredder.to_fen(), Board::STARTPOS_FEN);
}

#[test]
fn chess960_two_rooks_on_the_same_side() {
    // King on f1, rooks on g1 and h1: `K` is the outer rook, `G` the other.
    let outer = parse("4k3/8/8/8/8/8/8/5KRR w K - 0 1");
    let inner = parse("4k3/8/8/8/8/8/8/5KRR w G - 0 1");
    let rook = |board: &Board| {
        board
            .castling_config()
            .rook_start(Color::White, CastleSide::King)
    };
    assert_eq!(rook(&outer), Some(Square::H1));
    assert_eq!(rook(&inner), Some(Square::G1));
    assert_eq!(outer.castling_rights(), inner.castling_rights());
    assert_ne!(outer, inner);

    let config = inner.castling_config();
    let rook_moves =
        |from, to| config.rights_after(CastlingRights::ALL, Color::White, false, from, to);
    assert_eq!(
        rook_moves(Square::G1, Square::G2),
        !CastlingRights::WHITE_KING_SIDE
    );
    assert_eq!(rook_moves(Square::H1, Square::H2), CastlingRights::ALL);
    assert_eq!(outer.to_fen(), "4k3/8/8/8/8/8/8/5KRR w K - 0 1");
    assert_eq!(inner.to_fen(), "4k3/8/8/8/8/8/8/5KRR w G - 0 1");
    // Shredder letters for the outer rook read back as standard ones.
    assert_eq!(parse("4k3/8/8/8/8/8/8/5KRR w H - 0 1"), outer);
}

// Accessors

#[test]
fn startpos() {
    let board = Board::startpos();
    assert_eq!(board.occupied().count(), 32);
    assert_eq!(board.piece_on(Square::E1), Some(Piece::WhiteKing));
    assert_eq!(board.piece_on(Square::D8), Some(Piece::BlackQueen));
    assert_eq!(board.king_square(Color::Black), Square::E8);
    assert_eq!(board.side_to_move(), Color::White);
    assert_eq!(board.castling_rights(), CastlingRights::ALL);
    assert_eq!(board.en_passant(), None);
    assert!(!board.in_check());
    assert!(board.checkers().is_empty());
    assert!(board.compute_pinned().is_empty());
    assert_eq!(
        board.piece_bb(Color::White, PieceType::Pawn),
        Bitboard::from_rank(Rank::R2)
    );
}

#[test]
fn checks_and_pins_are_computed() {
    // Black knight on d3 checks e1; the white bishop on d2 is pinned by b4.
    let board = parse("4k3/8/8/8/1b6/3n4/3B4/4K3 w - - 0 1");
    assert_eq!(board.checkers(), Bitboard::from_square(Square::D3));
    assert_eq!(board.compute_pinned(), Bitboard::from_square(Square::D2));
}

#[test]
fn display() {
    let board = parse("4k3/8/8/8/8/8/8/4K2r w - - 0 1");
    let text = board.to_string();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "8  . . . . k . . .");
    assert_eq!(lines[7], "1  . . . . K . . r");
    assert_eq!(lines[8], "   a b c d e f g h");
    assert!(text.contains("FEN:      4k3/8/8/8/8/8/8/4K2r w - - 0 1"));
    assert!(text.contains(&format!("{:016X}", board.hash())));
    assert!(text.contains("Checkers: h1"));
    assert!(Board::startpos().to_string().contains("Checkers: -"));
}

// Mirror

#[test]
fn mirror_is_an_involution() {
    for fen in data_lines(VALID) {
        let board = parse(fen);
        assert_eq!(board.mirror().mirror(), board, "{fen}");
    }
}

#[test]
fn mirror_of_startpos() {
    let black_to_move = parse("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
    assert_eq!(Board::startpos().mirror(), black_to_move);
}

#[test]
fn mirror_of_perft_position_4() {
    let board = parse("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let mirrored = parse("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1");
    assert_eq!(board.mirror(), mirrored);
}

#[test]
fn mirror_keeps_en_passant() {
    let board = parse("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    assert_eq!(board.mirror().en_passant(), Some(Square::F3));
    assert_eq!(board.mirror().side_to_move(), Color::Black);
}
