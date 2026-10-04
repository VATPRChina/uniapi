-- Apply to a COPY of the navdata database to reproduce the indexed corpus run.
CREATE INDEX IF NOT EXISTS corpus_tbl_pa_airports ON tbl_pa_airports (airport_identifier);
CREATE INDEX IF NOT EXISTS corpus_tbl_d_vhfnavaids ON tbl_d_vhfnavaids (coalesce(navaid_identifier, dme_ident));
CREATE INDEX IF NOT EXISTS corpus_tbl_db_enroute_ndbnavaids ON tbl_db_enroute_ndbnavaids (navaid_identifier);
CREATE INDEX IF NOT EXISTS corpus_tbl_pn_terminal_ndbnavaids ON tbl_pn_terminal_ndbnavaids (navaid_identifier);
CREATE INDEX IF NOT EXISTS corpus_tbl_ea_enroute_waypoints ON tbl_ea_enroute_waypoints (waypoint_identifier);
CREATE INDEX IF NOT EXISTS corpus_tbl_pc_terminal_waypoints ON tbl_pc_terminal_waypoints (waypoint_identifier);
CREATE INDEX IF NOT EXISTS corpus_tbl_er_enroute_airways ON tbl_er_enroute_airways (route_identifier, area_code, seqno);
CREATE INDEX IF NOT EXISTS corpus_tbl_pd_sids ON tbl_pd_sids (procedure_identifier, airport_identifier, route_type, transition_identifier, seqno);
CREATE INDEX IF NOT EXISTS corpus_tbl_pe_stars ON tbl_pe_stars (procedure_identifier, airport_identifier, route_type, transition_identifier, seqno);
ANALYZE;
