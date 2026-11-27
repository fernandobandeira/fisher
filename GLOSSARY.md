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

**Starting position**:
The position a game of standard chess starts from, which UCI calls `startpos`. A search can start from any position, and that one is its root.
_Avoid_: initial position, opening position

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

**Opposite check**:
Said of a position in which the king of the color that is not the side to move is attacked. No legal move leads to one, so a FEN that describes one is rejected.
_Avoid_: other king in check, illegal check

**Terminal position**:
A position with no legal move: its side to move is checkmated or stalemated. A position drawn by the fifty-move rule or by repetition still has legal moves and is not one.
_Avoid_: final position, game over, end of the game

**Reply**:
A move that answers the move before it, played by the other color. What fisher sends over UCI is a response, never a reply.
_Avoid_: answer, counter-move, response (for a move)

**Fullmove**:
A move by White followed by a move by Black. The fullmove number starts at 1 and goes up after Black moves.
_Avoid_: turn, round

**Halfmove clock**:
The number of plies since the last capture or pawn move, used for the fifty-move rule. The name is the one FEN gives it.
_Avoid_: fifty-move counter, draw counter

### Search

**Bench signature**:
The total of the node counts from the last finished depth of an iterative deepening to a fixed depth on each position in `expectations/perft.epd`. Bench uses `deepening`'s own search, so each count includes the earlier depths as a `go` reports them. The total is compared between revisions to show whether a change altered which nodes the search reaches.
_Avoid_: benchmark, bench number

**Ply**:
One move by one color, used as the unit of distance in the game tree. Depth is counted in plies.
_Avoid_: half-move (except in the name halfmove clock), level

**Root**:
The position a search starts from, at ply 0.
_Avoid_: starting position (which names the position a game of chess starts from), search position

**Root move**:
A legal move of the root. The search chooses one, and every variation it reports starts with one.
_Avoid_: first move, candidate move, top-level move

**Best move**:
The root move a search chooses, which fisher sends as `bestmove`. A held move is a best move not yet sent.
_Avoid_: chosen move, selected move, top move

**Horizon**:
The ply at which a search stops playing moves. A position there is scored as it stands, without asking whether it has a legal move.
_Avoid_: frontier, search edge, leaf depth

**Score**:
What a search says a position is worth to that position's side to move: a number of centipawns, or a mate distance. The score fisher reports is the root's.
_Avoid_: rating, assessment

**Mate distance**:
The number of plies from the root to a checkmate the search has found. UCI counts it in moves instead, and gives it a minus sign when the root's side to move is the one checkmated: `mate 2` is three plies and `mate -1` is two.
_Avoid_: mate in N (without saying plies or moves), mate depth

**Minimax**:
The score and principal variation found by playing every legal move to a fixed depth and keeping, at each position, the move best for its side to move. A cutoff does not change its score.
_Avoid_: negamax, brute force, full-width search

**Bound**:
A value that a position's score is known to be at least, or at most, when the score itself is not known. A bound is never reported or compared as a score. An SPRT bound is a different thing, an Elo difference, and its name is always written in full.
_Avoid_: estimate, approximate score

**Cutoff**:
Leaving the remaining moves of a position unplayed, because a reply already found proves the position cannot change the result at the root. It does not interrupt the search: the depth in hand still finishes, with the same score.
_Avoid_: cut, refutation, witness

**Move order**:
The order in which the search tries the legal moves of a position, so that moves likely to cause a cutoff can be tried first.
_Avoid_: move ranking, move sort

**Ordering**:
One rule that changes the move order, such as trying captures first, trying killer moves or using history. Each ordering is judged on its own by comparing nodes and time on the same positions against the revision before it.
_Avoid_: move ordering (for the whole order, use move order), ordering (for the memory used by a rule, use move order memory)

**Capture**:
A move that takes an opposing piece, including en passant, which takes a pawn from a square other than the destination square. For move order, captures and promotions to a queen form one group, including promotions to a queen that take no piece.
_Avoid_: take (as a noun), attack

**Quiet move**:
For move order, any move outside the group of captures and promotions to a queen. This includes a promotion to a knight, bishop or rook that takes no piece.
_Avoid_: safe move, non-capture (as a name for this group)

**Killer move**:
A move remembered because it caused a cutoff in a position at the same ply. After captures and promotions to a queen, the search tries a matching move among the quiet moves still untried in the position in hand.
_Avoid_: refutation, winning move

**History**:
A record of quiet moves that caused cutoffs within one `go`, across positions and plies. The search uses it to order the quiet moves left after the killer moves.
_Avoid_: history heuristic (for the record), history on its own for command histories, played moves or Git revisions

**Move order memory**:
The one value `search` exports to hold what the search remembers for move order. Its contents are private to `search`, and `deepening` creates it at each `go` and passes it to every depth.
_Avoid_: search state, history (which names only the record of quiet moves that caused cutoffs)

**Perft**:
The number of leaf nodes in the tree of legal moves to a fixed depth from a position, used to check move generation against published counts.
_Avoid_: move count, node test

**Search**:
What fisher does between accepting a `go` and sending its `bestmove`: the work of choosing a move, and any wait for `stop` after it.
_Avoid_: thinking, calculation

**Node**:
A position the search reaches by playing a move. The root is not a node.
_Avoid_: visit, child

**Variation**:
A sequence of moves from a position, each one legal after the one before.
_Avoid_: line, continuation, sequence

**Principal variation**:
The moves the search expects both colors to play from the position it searched, starting with the move it chose.
_Avoid_: best line, main line, expected line

**Iterative deepening**:
Searching a position to depth 1, then to depth 2, and so on, each time from the start, and keeping the result of the last finished depth.
_Avoid_: progressive deepening, incremental search

**Finished depth**:
A depth at which every node was searched. Only a finished depth has a score and a principal variation.
_Avoid_: completed depth, complete depth, iteration

**Limit**:
What the GUI's `go` allows a search: a depth, a number of nodes, a time for the move, or the clocks.
_Avoid_: budget, allowance, ceiling

**Depth cap**:
The greatest depth fisher searches to, whatever the GUI asks. It is not a limit: a search under `go infinite` that reaches it waits for `stop`.
_Avoid_: maximum depth, cap on its own

**Interrupt**:
To end the work of a search before the depth in hand has finished. A `stop`, a limit, `quit` and the end of input all interrupt, and the unfinished depth is thrown away.
_Avoid_: cancel, abort, stop (which names only the UCI command)

### The book

**Book**:
A table that lists, for each position it knows, the moves to play there without searching. It is keyed by position, not by the moves that led there, so it holds no opening lines.
_Avoid_: opening book, opening database, repertoire

**Book move**:
A move the book lists for a position, together with a weight that says how strongly the book prefers it.

**Out of book**:
Said of a position for which the book lists no book move.

### The protocol

**UCI**:
The Universal Chess Interface, the text protocol a GUI uses to drive an engine. "The protocol" always means UCI.
_Avoid_: chess interface, chess engine protocol

**GUI**:
The program at the other end of UCI, which starts the engine and tells it what to search, whether or not it has a window. A match runner is a GUI. The name is the one the UCI specification gives it.
_Avoid_: host, client, front end, chess app

**Session**:
One run of fisher, from the start of the process to `quit` or the end of standard input. A position carries over from one search to the next within it.
_Avoid_: connection, game

**UCI move**:
A move as UCI writes it: the square a piece leaves, the square it reaches, and a piece kind when a pawn promotes, such as e7e8q. It does not say whether it castles or captures; only a position can turn it into a move.
_Avoid_: long algebraic notation, coordinate move, move string

**Held move**:
The best move or `0000` withheld after `go infinite` until `stop`.

### The clock

**Clock**:
The time a color has left for the rest of its moves, as the GUI sends it with `go`.
_Avoid_: timer, time left, clock (for the source fisher reads the time from)

**Increment**:
The time the GUI adds to a color's clock after each move that color makes.
_Avoid_: bonus, delay

**Moves to go**:
The number of moves a color must still make before the GUI refills its clock, counting the move it is about to make. When the GUI sends none, the clock is never refilled.
_Avoid_: moves left, moves remaining

**Time control**:
The rule a game's clocks follow: the starting clock, the increment and the number of moves between refills, if any.
_Avoid_: time format, time control (for the moment a clock is refilled)

**Time for the move**:
How long a search may take from its `go` to its `bestmove`. The GUI gives it with `go movetime`, or fisher works it out from the clock, the increment and the moves to go.
_Avoid_: budget, allocation, think time

**Margin**:
The part of a time for the move that fisher does not use, kept back so that `bestmove` reaches the GUI in time.
_Avoid_: overhead (except in the option name Move Overhead), buffer

**Loss on time**:
A game the GUI gives against a color whose clock ran out before its move arrived.
_Avoid_: time forfeit, flag, timeout (which names only the hang timeout)

### Strength

**Series**:
The recorded games between two revisions of fisher under numbered conditions. They are played under a node limit and judge no claim of strength.
_Avoid_: strength test, SPRT (for the record)

**SPRT**:
A test of one claim about a change, played against the revision the change is based on: that it plays stronger, or that it does not play weaker. It plays opening pairs under a time control and gives a verdict, pass or fail, when the results first accept one of the two hypotheses its SPRT bounds fix. A test that plays its most pairs without a verdict is inconclusive, and the claim is not accepted.
_Avoid_: series (for the test)

**SPRT bounds**:
The two Elo differences an SPRT chooses between. They are not the two values of the log-likelihood ratio at which the test ends, which are its thresholds. Always write the name in full.
_Avoid_: decision bounds, bounds on its own for the Elo differences

### Quality checks

**Hang timeout**:
The longest a test waits for fisher before it fails as hung. Never an expected duration.
_Avoid_: hang bound, test deadline

**Mutant**:
One source change generated by the mutation tool to see whether the tests detect it.

**Surviving mutant**:
A mutant that builds and passes the tests. An unexplained survivor is a gap in the evidence.

**Equivalent mutant**:
A surviving mutant with a reviewed proof that the source change preserves all observable behavior.

**Unviable mutant**:
A mutant that fails to build, so the test suite cannot judge it.

**Region coverage**:
The share of LLVM-instrumented source regions executed by the tests. A region can be smaller or larger than a line.

**Coverage floor**:
The minimum region coverage a crate must retain, set from a measured baseline and rounded down to a half percentage point.
