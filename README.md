# LH-Chess
A left-chess engine.

## Left-Chess
Is a chess variant with the following rules:

- Each piece moves as if it were the first piece that is encountered 
moving to the left of it, from the player's perspective.
- If a piece is the first on its rank, the search continues by wrapping 
around to the right end of the chessboard (therefore a piece that is alone on a rank moves normally)

I don't know who originated left-chess; the name is a literal translation from the Italian "scacchi a sinistra", 
which is the name by which I originally leaned it.

Example:

In the following position:

```
8  ♜ ♞ ♝ ♛ . ♚ . ♜
7  ♟︎ ♟︎ . ♝ ♟︎ ♙ ♞ ♟︎
6  . . . ♙ . . . .
5  ♟︎ . . . . . ♟︎ ♕
4  . . . . . . . .
3  . . . . . ♙ . .
2  . ♙ ♙ ♙ ♙ ♖ ♗ .
1  ♖ ♘ ♗ . ♔ . ♘ .

   A B C D E F G H
```

- The black rook on `A8` only has the knight move, `rA8B6`, since on its left (from black's perspective) there is a knight.
- The white queen on `H5` only has the pawn move, `QH5H6`, since on its left (from white's perspective) there is a pawn.
- The black pawn on `G5` moves like a queen and has a whopping 16 moves, namely:
  - `pG5F6`
  - `pG5G6`
  - `pG5H6`
  - `pG5B5`
  - `pG5C5`
  - `pG5D5`
  - `pG5E5`
  - `pG5F5`
  - `pG5xQH5`
  - `pG5F4`
  - `pG5G4`
  - `pG5H4`
  - `pG5E3`
  - `pG5G3`
  - `pG5xpD2`
  - `pG5xbG2`

## The engine
To simplify the game, the engine plays under the following rules:
- Castling is not allowed, since it is hard to formally define in
left-chess: whenever a king and a rook are in suitable position for castling,
the king does not move as a king, as it is not alone on the rank.
- Moving a pawn by two squares on its first move or capturing en-paissant is not allowed due to the fact 
that a pawn may come back to its original position after moving, thus making it harder to determine
whether the move is legal in a given position (without having a full game history).


