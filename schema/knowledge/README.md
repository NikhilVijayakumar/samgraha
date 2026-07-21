# knowledge.db — samgraha's own execution schema

Samgraha-owned tables (`usecase`, `script`, `prompt`, `step`, `step_script`,
`step_prompt`, `execution`) plus one catalog table for standard-owned data
(`custom_data_tables`). Samgraha creates and migrates every table in this
directory. It never creates or migrates whatever tables a standard's own
scripts add to the same `knowledge.db` file — `custom_data_tables` only
records that they exist, their purpose, and (once introspected) their
shape.

Loaded by `crates/registry`'s Rust migrations, not a Python loader — no
runtime dependency on this directory's `.sql` files; they're the canonical
reference copy of what the Rust `const` migrations implement.

See `../registration/` for `registry.db` (repository registration —
separate file, separate concern, unchanged from the existing
implementation).
