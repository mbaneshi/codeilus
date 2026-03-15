-- Seed 8 canonical badges (idempotent)
INSERT OR IGNORE INTO badges (name, description, icon) VALUES
    ('First Steps',      'Complete your first section',       'rocket'),
    ('Chapter Champion', 'Complete any full chapter',         'trophy'),
    ('Graph Explorer',   'Visit 10 unique graph nodes',      'map'),
    ('Quiz Master',      'Pass 5 quizzes',                   'brain'),
    ('Deep Diver',       'Read 20 symbol explanations',      'microscope'),
    ('Completionist',    'Reach 100% learning progress',     'star'),
    ('Polyglot',         'Explore code in 3+ languages',     'globe'),
    ('Code Detective',   'Find 3 anti-patterns',             'magnifying-glass');
