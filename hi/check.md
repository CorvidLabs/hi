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

- **CHECK-1**  As a maintainer, I can add hi to an existing repo and it will not turn the build red.
  - **CHECK-1.a**  As a developer, a criterion with nothing implementing it is never an error.
  - **CHECK-1.b**  As a developer, an unfinished file is never an error.
- **CHECK-2**  As a developer, hi fails only when a file is structurally wrong.
  - **CHECK-2.a**  As a maintainer, two criteria sharing one id is an error.
  - **CHECK-2.b**  As a reader, a case whose parent is not in the same file is an error, because a case belongs with the criterion it is a case of.
  - **CHECK-2.c**  As a maintainer, reusing a retired id is an error.
  - **CHECK-2.d**  As a developer, a line that is shaped like an id but is not a valid one is an error, because otherwise it would read as prose and vanish.
  - **CHECK-2.e**  As a developer, a criterion outside the criteria and retired sections is an error, because nothing would read it there.
  - **CHECK-2.f**  As a maintainer, using a family the file never declared is an error.
- **CHECK-3**  As a developer, every problem names the file and the line, so I can go straight to it.
- **CHECK-4**  As a developer, checking works offline and reads nothing but my own files.
- **CHECK-5**  As a developer, one run tells me every problem in every file, so I fix them all in one pass.
