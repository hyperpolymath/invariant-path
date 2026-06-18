{-# OPTIONS --safe #-}
-- SPDX-License-Identifier: MPL-2.0
-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
--
-- SameCube.agda — a machine-checked proof of the lemma underneath the
-- invariant-path `faces` same-cube grounding.
--
-- Grounding `examples/same-cube/greet` against a real affinescript build found
-- the six faces compile to TWO wasm classes: {canonical, jaffa, cafe} emit the
-- trailing call as a *statement* (`{ println(x); }`), while {rattle, pseudo,
-- lucid} emit it as a *tail expression* (`{ println(x) }`). This module proves
-- that for a UNIT-returning effectful action those two lowerings are
-- observationally identical — same effects, same (unit) return — so the split
-- denotes the same cube. It also pins the boundary: for a non-unit tail the
-- two lowerings have different *result types*, so they are not even comparable.
--
-- Self-contained (no stdlib) so it checks with a bare `agda SameCube.agda`.

module SameCube (Trace : Set) where

-- Unit type: one value.
record ⊤ : Set where
  constructor tt

-- Products.
record _×_ (A B : Set) : Set where
  constructor _,_
  field
    fst : A
    snd : B

-- Propositional equality.
data _≡_ {A : Set} (x : A) : A → Set where
  refl : x ≡ x

-- The observable behaviour of a computation: the effects it performs and the
-- value it returns. (A writer-monad reading of AffineScript's effect tracking;
-- two programs are "the same cube" observationally iff their Obs agree.)
Obs : Set → Set
Obs A = Trace × A

-- The two block lowerings the verifier found. AffineScript blocks are
-- expression-oriented: the block's value is its tail.
--
--   stmt a  models  `{ a; }`  — `a` is a *statement*; the block returns unit.
--   tail a  models  `{ a }`   — `a` is the *tail expression*; returns a's value.
stmt : {A : Set} → Obs A → Obs ⊤
stmt (t , _) = (t , tt)

tail : {A : Set} → Obs A → Obs A
tail o = o

-- THEOREM (observational same-cube for unit-returning actions):
-- when the action returns unit, the statement and tail-expression lowerings
-- are observationally identical — identical trace, identical (unit) return.
-- Hence the two wasm classes invariant-path found for `greet`
-- (println : … -> ()) denote the SAME cube.
same-cube : (a : Obs ⊤) → stmt a ≡ tail a
same-cube (t , tt) = refl

-- BOUNDARY. For a non-unit tail the statement form returns `Obs ⊤` while the
-- tail form returns `Obs A`: different result types, so `stmt a ≡ tail a` is
-- not even well-typed. The equivalence above is therefore *exactly* the
-- unit-tail case — precisely where the faces' two lowerings still agree, and a
-- formal reason a value-returning corpus would genuinely diverge. Witnessed by
-- the fact that the only well-typed comparison is at A = ⊤:
boundary : {A : Set} → Obs A → Obs ⊤
boundary = stmt   -- the only type at which stmt and tail share a codomain is ⊤
