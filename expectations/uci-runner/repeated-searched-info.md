# Repeated searched info examples

Source: fisher-5sg.1 and fisher-r6y D15, with fisher-5sg decision 1
approved by the owner on 2026-11-24. An independent author chose the
verdicts before the runner change. The full spelling follows fisher-r6y
D41 and Stockfish sf_19. Under fisher-r6y D5, the principal variation is
as long as the depth searched, shorter only when it ends in checkmate or
stalemate.

[repeated-searched-info.json](repeated-searched-info.json) holds the
fixtures for fisher-5sg.2. Each case starts a fresh scripted session.

- `transcript` is an array of lines. Append LF to each and concatenate
  them to obtain the input to the transcript loader.
- `script` lists batches of events. Each batch uses `after_send`, the
  one-based ordinal of a transcript send, counting every `>` record
including one that ends in CR. Release its events only after
  that send has been flushed. There is no unsolicited output.
- Each `events` entry is `[milliseconds, kind, value]`. `stdout` emits
  the string followed by LF, `eof` closes stdout (value is null), and
  `exit` makes the given integer process status observable. Deliver events
  at the same time in order. Between events stdout and status stay as
  they were.
- A batch with `clock: "observation"` starts its controlled clock when
  the runner begins reading the repetition immediately after its send.
  Every other batch uses immediate events at time zero. Use controlled
  time for the timed batches.
- `expected.verdict` is `accept`, `load-reject`, or `run-reject`.
  `expected.reason` describes the observation that determines the verdict,
  independent of error wording. For load rejection require no sends; the script of a load-reject case
is never released.
  When the transcript rejects early, later scripted events need not run.

The two `readyok` timing cases differ only in when the delimiter
arrives. In both, matching output continues to 4.8 seconds. The exit
timing case closes stdout within five seconds and supplies the clean
status after the read timeout would have expired.

The mandatory-depth case places the exact depth 1 reply before the
repetition. In the greedy case, depth 4 matches the repetition and gets
consumed, so `readyok` reaches the ordinary depth 4 comparison and
fails. The principal variation cases exercise the depth-dependent bound at
depths below and above four.
