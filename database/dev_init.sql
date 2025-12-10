-- use this to fill the database with example data

INSERT INTO users (username)
VALUES 
('iamuser'),
('whoami')
ON CONFLICT (username) DO NOTHING;