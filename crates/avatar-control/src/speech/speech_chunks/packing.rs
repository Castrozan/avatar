use super::{ChunkLanguage, Piece, utf16_length};

pub(super) fn packed(pieces: Vec<Piece>, maximum: usize) -> Vec<Piece> {
    let mut chunks = Vec::new();
    for piece in pieces {
        append_piece(&mut chunks, piece, maximum);
    }
    chunks
}

fn append_piece(chunks: &mut Vec<Piece>, piece: Piece, maximum: usize) {
    match chunks
        .last_mut()
        .filter(|last| joins(last, &piece, maximum))
    {
        Some(last) => join_into(last, &piece),
        None => chunks.push(piece),
    }
}

fn joins(chunk: &Piece, piece: &Piece, maximum: usize) -> bool {
    utf16_length(&chunk.text) + utf16_length(&piece.text) <= maximum
        && languages_join(chunk.language, piece.language)
}

fn languages_join(chunk: Option<ChunkLanguage>, piece: Option<ChunkLanguage>) -> bool {
    chunk == piece
        || (chunk.is_some()
            && piece.is_some()
            && (chunk == Some(ChunkLanguage::Undecided) || piece == Some(ChunkLanguage::Undecided)))
}

fn join_into(chunk: &mut Piece, piece: &Piece) {
    chunk.text.push_str(&piece.text);
    chunk.language = match chunk.language {
        Some(ChunkLanguage::Undecided) => piece.language,
        language => language,
    };
}
