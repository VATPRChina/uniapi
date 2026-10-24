-- Test route database schema and seed data.
-- Populate a new database with: sqlite3 -bail data/routes.db < assets/test/routes.sql
CREATE TABLE Routes (
    Dep TEXT NOT NULL,
    Arr TEXT NOT NULL,
    Waypoint1 TEXT NOT NULL,
    Waypoint2 TEXT NOT NULL,
    Name TEXT NOT NULL,
    EvenOdd TEXT NOT NULL,
    AltList TEXT NOT NULL,
    MinAlt TEXT NOT NULL,
    Route TEXT NOT NULL,
    Remarks TEXT NOT NULL,
    CHECK ((Dep <> '') <> (Waypoint1 <> '')),
    CHECK ((Arr <> '') <> (Waypoint2 <> ''))
);
CREATE INDEX idx_airports ON Routes (Dep, Arr);
CREATE INDEX idx_outbound ON Routes (Dep, Waypoint2);
CREATE INDEX idx_inbound ON Routes (Waypoint1, Arr);
CREATE INDEX idx_transit ON Routes (Waypoint1, Waypoint2);
CREATE INDEX uniapi_routes_endpoints ON Routes (
    coalesce(nullif(trim(Dep), ''), trim(Waypoint1)) COLLATE NOCASE,
    coalesce(nullif(trim(Arr), ''), trim(Waypoint2)) COLLATE NOCASE
);
CREATE TABLE Metadata (CycleVersion TEXT NOT NULL);
INSERT INTO Metadata VALUES ('TEST');
INSERT INTO Routes (Dep, Arr, Waypoint1, Waypoint2, Name, EvenOdd, AltList, MinAlt, Route, Remarks)
VALUES
    ('ZBAA', 'ZSPD', '', '', 'ZBAA-ZSPD-01', 'SO', '', '7165',
     'ELKUR W40 YQG W142 DALIM A593 DPX A470 DALNU W166 ZJ W167 SASAN R343 EKIMU', 'SOP'),
    ('ZSPD', 'ZBAA', '', '', 'ZSPD-ZBAA-01', 'SE', '', '7165',
     'POMOK G330 PIMOL A593 DALIM W157 AVBOX', '');
