ALTER TABLE public.training
    ADD COLUMN self_reflection_sheet_filing_id uuid
        REFERENCES public.sheet_filing(id);

CREATE INDEX ix_training_self_reflection_sheet_filing_id
    ON public.training (self_reflection_sheet_filing_id);

INSERT INTO public.sheet (id, name)
VALUES ('training-self-reflection', 'Training Self Reflection');

INSERT INTO public.sheet_field (
    sheet_id, id, sequence, name_zh, name_en, kind, single_choice_options
)
VALUES (
    'training-self-reflection', 'reflection', 0,
    '自我反思', 'Self Reflection', 'long-text', ARRAY[]::text[]
);
