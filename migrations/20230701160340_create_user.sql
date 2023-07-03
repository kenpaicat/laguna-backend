-- For UUIDs.
-- uuid_generate_v4()
-- More: 
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE Role AS ENUM ('Normie', 'Verified', 'Mod', 'Admin');

CREATE TABLE IF NOT EXISTS "User" (
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    username VARCHAR(20) UNIQUE NOT NULL,
    email VARCHAR(40) UNIQUE NOT NULL,
    password TEXT NOT NULL,
    -- AKA. date joined
    first_login TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    avatar_url TEXT,
    role Role NOT NULL DEFAULT 'Normie',
    is_active BOOLEAN DEFAULT TRUE,
    has_verified_email BOOLEAN DEFAULT FALSE,
    is_history_private BOOLEAN DEFAULT TRUE,
    is_profile_private BOOLEAN DEFAULT TRUE
);