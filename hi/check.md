---
hi: 1
families: [CHECK]
owner: leif
---

# Check

## Intent

hi must be installable on a Friday afternoon without turning anyone's build
red. It has no opinion about whether a criterion is any good, whether it is
finished, or whether anything downstream implements it. Incomplete intent is
the normal state of intent.

The only thing it will fail on is a file that is structurally wrong, because
that is the one case where being quiet would let ids rot.

## Criteria

- **CHECK-1**  I can add hi to an existing repo and it will not turn the build red.
  - **CHECK-1.a**  A criterion with nothing implementing it is never an error.
  - **CHECK-1.b**  An unfinished file is never an error.
- **CHECK-2**  hi fails only when a file is structurally wrong.
  - **CHECK-2.a**  Two criteria sharing one id is an error.
  - **CHECK-2.b**  A case whose parent does not exist is an error.
  - **CHECK-2.c**  Reusing a retired id is an error.
  - **CHECK-2.d**  A line that is shaped like an id but is not a valid one is an error.
  - **CHECK-2.e**  A criterion sitting outside every section is an error, because nothing would read it there.
  - **CHECK-2.f**  A family a file never declared is an error.
- **CHECK-3**  Every problem names the file and the line, so I can go straight to it.
- **CHECK-4**  Checking works offline and reads nothing but my own files.
