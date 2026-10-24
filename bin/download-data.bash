#!/bin/bash
set -x

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)
DATA_DIR="${SCRIPT_DIR}/../data"

mkdir -p ${DATA_DIR}

gh release download --repo VATPRChina/navdata --pattern 'NavigraphDFDv2-*.zip' --output /tmp/navdata.zip --clobber
unzip -p /tmp/navdata.zip db.s3db > ${DATA_DIR}/navdata_naip.db
rm /tmp/navdata.zip

gh release download --repo VATPRChina/prc-route --pattern 'Route.db3' --output ${DATA_DIR}/routes.db --clobber

gh release download --repo VATPRChina/uniapi-supplemental-data --pattern 'AircraftTypes.json' --output ${DATA_DIR}/AircraftTypes.json --clobber
gh release download --repo VATPRChina/uniapi-supplemental-data --pattern 'Clearance.txt' --output ${DATA_DIR}/Clearance.txt --clobber
gh release download --repo VATPRChina/uniapi-supplemental-data --pattern 'ICAO_Airlines.txt' --output ${DATA_DIR}/ICAO_Airlines.txt --clobber
gh release download --repo VATPRChina/uniapi-supplemental-data --pattern 'Sid.txt' --output ${DATA_DIR}/Sid.txt --clobber
gh release download --repo VATPRChina/uniapi-supplemental-data --pattern 'Wingspan.txt' --output ${DATA_DIR}/Wingspan.txt --clobber

curl --fail --location --output /tmp/navdata.zip https://developers.navigraph.com/downloads/navigation-data/navigraph-dfd-sample-sqlite-dfdv2-2401.zip
unzip -p /tmp/navdata.zip ng_jeppesen_fwdfd_2401.s3db > data/navdata.db
rm /tmp/navdata.zip
