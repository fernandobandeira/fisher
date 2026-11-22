# Fisher

The language of a chess engine: the game as the rules describe it, and the few words engine programmers add to it.

This is a seed. Each entry is here because mixing it up with its neighbour causes bugs. Challenge them in grilling, and add a term only when confusing it has a cost.

A `_Banned in code_` line is enforced by `scripts/check` on every identifier under `crates/`. An `_Avoid_` line is guidance for prose.

## Language

### The board

**Square**:
One of the 64 places on the board where a piece can stand, named by its file and rank, such as e4.
_Avoid_: field, cell, tile
_Banned in code_: tile

**File**:
One of the eight vertical lines of squares, lettered a to h. Never a file on disk, which is always called a path or a document.
_Avoid_: column
_Banned in code_: column, col

**Rank**:
One of the eight horizontal lines of squares, numbered 1 to 8 starting from White's side.
_Avoid_: row
_Banned in code_: row

### Pieces

**Color**:
White or Black.
_Avoid_: side, player

**Piece kind**:
Pawn, knight, bishop, rook, queen or king, without a color.
_Avoid_: piece type, role, figure
_Banned in code_: piece type, figure

**Piece**:
A piece kind together with a color, such as a white knight. A pawn is a piece here, unlike in a chess player's usage.
_Avoid_: man, figure

### The game

**Position**:
Everything the rules need to continue a game: which piece stands on which square, the side to move, the castling rights, the en passant square, the halfmove clock and the fullmove number.
_Avoid_: board, state, game state

**Side to move**:
The color that makes the next move in a position.
_Avoid_: turn, active color, player to move
_Banned in code_: turn

**Move**:
One action by one color: a piece goes from one square to another, including castling, an en passant capture and a promotion.
_Avoid_: step, action

**Pseudo-legal move**:
A move that follows the way its piece moves but may leave the mover's own king in check.
_Avoid_: candidate move, possible move

**Legal move**:
A pseudo-legal move that does not leave the mover's own king in check.
_Avoid_: valid move, allowed move
_Banned in code_: valid move

**Fullmove**:
A move by White followed by a move by Black. The fullmove number starts at 1 and goes up after Black moves.
_Avoid_: turn, round

**Halfmove clock**:
The number of plies since the last capture or pawn move, used for the fifty-move rule. The name is the one FEN gives it.
_Avoid_: fifty-move counter, draw counter

### Search

**Ply**:
One move by one color, used as the unit of distance in the game tree. Depth is counted in plies.
_Avoid_: half-move (except in the name halfmove clock), level

**Perft**:
The number of leaf nodes in the tree of legal moves to a fixed depth from a position, used to check move generation against published counts.
_Avoid_: move count, node test
