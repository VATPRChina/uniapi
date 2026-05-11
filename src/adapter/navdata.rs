use std::path::Path;

use reqwest::header::AsHeaderName;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::{
    model::navdata::{Airport, AnyFix, Leg, ResolvedLeg},
    repository::navdata::airway,
};

pub struct NavdataAdapter {
    pub remote_data_url: String,
    pub local_data_path: String,
    pub db: Pool<Sqlite>,
}

impl NavdataAdapter {
    pub async fn new(
        remote_data_url: String,
        local_data_path: String,
        download_file: bool,
    ) -> Self {
        if download_file {
            let response = reqwest::get(&remote_data_url).await.unwrap();
            let bytes = response.bytes().await.unwrap();
            std::fs::write(&local_data_path, bytes).unwrap();
        }

        let local_data_path = std::fs::canonicalize(&local_data_path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let db = SqlitePool::connect(&format!("sqlite:{local_data_path}"))
            .await
            .unwrap();
        Self {
            remote_data_url,
            local_data_path,
            db,
        }
    }

    pub fn find_airport(&self, ident: &str) -> Option<Airport> {
        todo!()
    }

    pub fn find_nearest_fix(&self, latitude: f64, longitude: f64) -> Option<AnyFix> {
        todo!()
    }

    pub fn exists_sid(&self, airport_ident: &str, ident: &str) -> bool {
        todo!()
    }

    pub fn exists_star(&self, airport_ident: &str, ident: &str) -> bool {
        todo!()
    }

    pub fn exists_airway_with_fix(&self, airway_ident: &str, fix_ident: &str) -> bool {
        todo!()
    }

    pub fn list_airway_legs_between(
        &self,
        airway_ident: &str,
        from_ident: &str,
        to_ident: &str,
    ) -> Vec<ResolvedLeg> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const DFD_V2_SAMPLE_DATA_URL: &str = "https://developers.navigraph.com/downloads/navigation-data/navigraph-dfd-sample-sqlite-dfdv2-2401.zip";
    const LOCAL_DATA_PATH: &str = "Data/ng_jeppesen_fwdfd_2401.s3db";

    async fn get_navdata_adapter() -> NavdataAdapter {
        if !std::fs::exists(LOCAL_DATA_PATH).unwrap() {
            let response = reqwest::get(DFD_V2_SAMPLE_DATA_URL).await.unwrap();
            let bytes = response.bytes().await.unwrap();

            let zipfile = std::io::Cursor::new(bytes);
            let mut archive = zip::ZipArchive::new(zipfile).unwrap();
            let mut file = archive.by_name("ng_jeppesen_fwdfd_2401.s3db").unwrap();
            let mut out = std::fs::File::create(LOCAL_DATA_PATH).unwrap();
            std::io::copy(&mut file, &mut out).unwrap();
        }

        NavdataAdapter::new(
            DFD_V2_SAMPLE_DATA_URL.to_string(),
            LOCAL_DATA_PATH.to_string(),
            false,
        )
        .await
    }

    #[tokio::test]
    async fn test_navdata_adapter() {
        let adapter = get_navdata_adapter().await;
        assert!(adapter.db.acquire().await.is_ok());
    }
}
