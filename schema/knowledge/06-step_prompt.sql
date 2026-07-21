-- knowledge.db — maps a semantic step to the prompt handed to the calling
-- agent. One row per step; step.kind must be 'semantic' for a row to
-- exist here (enforced in application code, same reasoning as
-- step_script).

CREATE TABLE IF NOT EXISTS step_prompt (
    step_id   INTEGER NOT NULL REFERENCES step(id) ON DELETE CASCADE,
    prompt_id INTEGER NOT NULL REFERENCES prompt(id) ON DELETE CASCADE,
    PRIMARY KEY (step_id, prompt_id)
);
