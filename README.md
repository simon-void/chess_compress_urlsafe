chess_compress_urlsafe
======================

A library to compress and decompress a sequence of valid chess moves to/from a form that is:
- shorter than PNG
- easier to parse than PNG (since it never requires disambiguation)
- is url-safe (contains only characters from [url-safe base64](https://datatracker.ietf.org/doc/html/rfc4648#section-5) minus the character for padding)

## the basic idea

the easiest parsable representation of a chess game would be to simply denote a move by writing down its start and end position,
e.g.
```text
e4 e5 Sf3 Sc6 Lb5 .. Rxe1+ .. b1=Q 
```
would become
```text
e2e4e7e5g1f3b8c6f1b5..e8e1..b2b1Q 
```
Since the length of an url hasn't any upper limit according to the [specification](https://www.rfc-editor.org/rfc/rfc2616),
there seems to be a [practical limit set by browsers](https://stackoverflow.com/a/812962) with Microsoft Internet Explorer
supporting by far the least amount of characters with an upper bound of 2048 characters.
With the number of characters being 4 per move so far, we could reach 500 half moves which would be a match with 250 moves.
That should suffice for most games, but since it's fun (and to save internet bandwidth by generating shorter urls), let's compress this representation.

A chess board has 64 fields and base64 encoding provides 64 characters (without the padding character which we don't use).
So we can encode every position (but one with pawn promotion) into 2 characters, e.g. `e2e4` becomes `Mc`.

This gets our example down to
```text
Mc0kGV5qFh..8E..JBQ
```
as you can see, in case of pawn promotion we just attach a 'Q'ueen, 'R'ook, 'K'night' or 'B'ishop to the base 4 representation.
All 4 characters are also valid base64 values, so while decoding we have to decide from context,
if the next character is the promotion type or the next position.

Now that we halved the length of the generated value, I could have stopped but there was one more kind of low-hanging fruit!
Inspired by the PNG notation of pawn moves that often only use the position the pawn is going to, I realized I could drop
the from-position of a move if it's the only move that can reach its to-position.
(I only check if a figure is in principle capable of reaching a field, not if it's bound to the king,
because looking for checks would increase the complexity of the implementation by quite a bit.) 
E.g. since our first move is `e2e4` and the e-pawn is the only figure at that moment that can reach `e4`, we only have to encode `e4`->`c`.
(Notice that if our first move was `c2c3`, we wouldn't be able to drop `c2` because the knight on `b1` is also able to go to `c3`.)

This brings our encoded value down to (assuming the checking rook and the promoting pawn are the only figures to move to their target when it's their turn)
```text
ckGV5qh..E..BQ
```

This means we end up with 50%-75% fewer characters compared to our first idea of a parsable representation!