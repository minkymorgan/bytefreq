#!/bin/bash

#                    you will need lynx, uncomment the line below if you don't have it
# brew install lynx

#                    you will need wget, uncomment the line below if you don't have it
# brew install wget



# NOTE: There is a command line web browser called lynx that is a million years old. It's how I scrape the web.
#       Here is an example of using it to construct the back history of the hmrc spending data.

# Step 1: visit the main page, extract the links to a file. There will be a lot of junk to filter out...
lynx -listonly -dump 'https://www.gov.uk/government/collections/spending-over-25-000' > links.txt

# Step 2: filter out all the junk, leaving the stuff we need to vist, to find the specific monthly announcement pages, which have should have a link to the csv.
cat links.txt | grep "publications"| grep "hmrc-spending-over-25000-" | awk '/http/{print $2}'  > filtered_links.txt

# Step 3: remove and create an empty list of final csv links

>final_csv_links.txt
touch final_csv_links.txt


# loop through all the month by month annoucement pages, and extract the actual CSV data link we are after. Append the csv data link to the final list of links.
while read -r line; do
    lynx -listonly -dump "$line" | grep HMRC_spending | grep -v preview | sed 's/^.*https/https/' | sort | uniq >> final_csv_links.txt
done < filtered_links.txt


# Finally we can automate fetching hundreds of csv files using the wget command, feeding it our list.

wget --no-check-certificate -i final_csv_links.txt

# if we accidentally grabbed a file twice (happens) we can delete the copies using this:
find . -type f -name "*.csv.*" -delete

