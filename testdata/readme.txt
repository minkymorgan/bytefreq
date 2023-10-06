

# Download the test files using this script:
# (for files too large, I will document how to download the original opendata files I use for testing)

# Some dirty data files, which are too big for github are these, which outline where electric vehicles can be charged.

# visit this site to get details:  https://www.gov.uk/guidance/find-and-use-data-on-public-electric-vehicle-chargepoints
wget https://chargepoints.dft.gov.uk/api/retrieve/registry/format/json -O chargepoints.json
wget https://chargepoints.dft.gov.uk/api/retrieve/registry/format/csv -O chargepoints.csv



