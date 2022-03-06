BEGIN;
CREATE TABLE IF NOT EXISTS account ( 
    id INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE,
    fullname VARCHAR(100) NOT NULL,
    email VARCHAR(62) NOT NULL UNIQUE,
    passhash VARCHAR(60) NOT NULL,
    account_type INTEGER NOT NULL
);


CREATE TABLE IF NOT EXISTS doctor ( 
    id INTEGER PRIMARY KEY NOT NULL UNIQUE,
    specialty VARCHAR(100) NOT NULL default "",
    details TEXT NOT NULL default "",
    starting_hour char(5) NOT NULL default "08:00",
    ending_hour char(5) NOT NULL default "18:00",
    FOREIGN KEY(id) REFERENCES account(id)
);


CREATE TABLE IF NOT EXISTS appointments ( 
    id INTEGER PRIMARY KEY AUTOINCREMENT UNIQUE,
    doctor INTEGER NOT NULL,
    patient INTEGER NOT NULL,
    appointment_status INTEGER NOT NULL,
    starting_date INTEGER NOT NULL,
    duration_mins INTEGER NOT NULL 
    check (duration_mins >= 15 and duration_mins <= 120),
    FOREIGN KEY(patient) REFERENCES account(id), 
    FOREIGN KEY(doctor) REFERENCES account(id)
);
COMMIT;