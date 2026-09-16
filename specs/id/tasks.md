---
spec: id.spec.md
---

## Tasks

- [x] Specify the id grammar, the six public entry points, and the three `Display` impls. Evidence: `specs/id/id.spec.md`, `status: active`.
- [x] Map every REQ to real inline tests. Evidence: `specs/id/testing.md`, `cargo test --bin hi id::tests` (10 passing).
- [x] Refuse a zero-padded numeric level instead of normalizing it (REQ-id-011, hi: ID-1.c). Evidence: `IdError::PaddedLevel` in `src/id.rs`, `rejects_a_zero_padded_level`, `tests/cli.rs::a_zero_padded_id_is_refused_rather_than_silently_renamed`.
- [ ] Add a direct unit test for `root_number`, including the `None` branch on an `Id` built without `parse` (REQ-id-008).
- [ ] Add a test pinning the rendered `Display` sentence of each `IdError` variant, since those strings are user-facing `hi check` output (REQ-id-004, REQ-id-011).
- [ ] Add cases for a multi-letter level (`SEND-1.aa`), an uppercase level (`SEND-1.A`), and a digit level wider than `u32::MAX` (REQ-id-004, REQ-id-005).

## Gaps

- `root_number` has no direct test; it is only exercised through `capture::refuses_an_id_that_already_exists` building the "next free is SEND-2" hint and the rendered hint in `tests/cli.rs`.
- No test asserts the full `Display` text of `IdError`. `Alternation`'s `depth` and `expected` are compared by value rather than through the rendered sentence, and `PaddedLevel`'s sentence is pinned only by the substring "leading zero" in `tests/cli.rs`.
- Untested but reachable inputs: a multi-letter level, an uppercase or mixed level, and a digit level that overflows `u32`.
- Nothing asserts that a zero-padded id already written into a file surfaces as a `check` problem rather than resolving to the unpadded criterion; only the capture path is covered.
- Nothing tests an `Id` built directly through its public fields, which is the one way to hold an id that does not satisfy the alternation rule (`capture` does this deliberately).

## Review Sign-offs

- **Product**: pending
- **QA**: pending
- **Design**: not applicable (no user-facing surface beyond error text)
- **Dev**: pending
