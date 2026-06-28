# Contributing to Chhoto URL

First of all, thanks for your interest in Chhoto URL. Any kind of contribution is welcome, as long as it adheres to the following rules.

1. No AI contribution is allowed. I can't stop you from using AI privately, but I expect all submitted code to be written, reviewed, and
   owned by the contributor. Do not list any AI tools as co-authors. You must take full responsibility for what you want to merge.
1. If it's a new feature, open a [discussion](https://github.com/SinTan1729/chhoto-url/discussions) first. This helps avoid spending time
   implementing features that aren't a good fit for the project.
1. Bug fixes do not require opening a discussion first unless they significantly change behavior.
1. When you open the PR, be very clear about what it does, and what problems it solves.
1. If you're making any changes to the backend, run `make test` to make sure that all the tests pass. For new features, you probably also
   need to create new tests.
1. Please format your code with the standard formatters before you submit a PR e.g. `rustfmt` for Rust, and `prettier` for the frontend
   stuff, and docs.
1. Everything must remain backwards compatible.
1. If you're making any changes to the frontend, be sure that it renders properly. Play around with it a bit, and add some screenshots in
   the PR.
