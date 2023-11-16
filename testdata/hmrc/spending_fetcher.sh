#!/bin/bash



# install some dependencies if you need them.
#                    you will need lynx, uncomment the line below if you don't have it
# brew install lynx

#                    you will need wget, uncomment the line below if you don't have it
# brew install wget
# brew install chardep

# NOTE: There is a command line web browser called lynx that is a million years old. It's how I scrape the web.
#       Here is an example of using it to construct the back history of the hmrc spending data.

#
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
# make and step into a raw directory

rmdir -rf raw
mkdir raw
cd raw



cp ../final_csv_links.txt .

wget --no-check-certificate -i final_csv_links.txt

# if we accidentally grabbed a file twice (happens) we can delete the copies using this:
find . -type f -name "*.csv.*" -delete

# step back out from the raw directory

# As we have no idea what characterset this data is in, we can assume it's UTF8.
# I have tried this and my python readers failed. 

# first thing to look at is a byte frequency report, to see what's going on bytefreq -r CP, 
cd ..

cat raw/*csv | bytefreq -r CP > CharFreq.raw.txt

# notice in the report that the number of carriage returns doesn't match the newlines.
# notice also there are lots of "replacement characters" which means the code page is NOT utf8.
# we need to find out what codepage the data is. 

# The first thing to do is probe each file, to guess the characterset and conversions to try out, to build up a UTF8 dataset.
# the command below will inspect and probe each file, discover high probably charactersets, and the sed command will generate the conversion scriptd
# that we can execute later to generate clean UTF8 data.

chardet raw/*csv | sed 's/^\(.*\): \(.*\) with confidence .*$/cat \1 |iconv -f \2 -t UTF-8\/\/TRANSLIT\/\/IGNORE > \1.UTF8.csv/' | sed 's/|iconv -f ascii.*>/>/' | sed 's/|iconv -f UTF.*>/>/' > Convert_to_UTF8.sh

sh Convert_to_UTF8.sh

# the above command will use iconv to convert all the data to UTF8.
# To double check things we need to re-run the bytefreq character profiler on the UTF8 data, to see things are fixed:

cat raw/*UTF*csv | bytefreq -r CP > CharFreq.utf8.txt

# when we examine the utf8 profiles, we see there are Non-Breaking-Spaces, and Horizontal Tabs, and most worrying, a different number of LF and CR characters.
# (keep in mind this dataset is called "Transactions over £25k" so losing 10 rows of data, could be losing £250000 of cash from our reports !!!)

# first lets clean up some of the weird stuff. 
# -- translate the tabs to spaces
# -- translate non-breaking-spaces to normal spaces


# this line says, take the first row as is (header) and all others that have a matching "HMRC" in the first column, as this is a real data row, and then switchout non-breaking spaces
# cat raw/*UTF*csv | awk 'NR==1 || $1 ~ /HMRC/ {gsub(/\xEF\xBB\xBF/, " "); print}' 
 
# this will do a quick csvclean report for each file and print outputs to the screen so we get some progress and assurance
ls raw/*UTF* | gawk '{print "echo "$0"; csvclean -n "$0}'  | sh

# It seems like there are no errors in the files ... which is good.
# so lets loop through each file and start the final converstions

for f in `ls raw/*UTF*csv`; do
	echo "processing: \t\t\t"${f}
        echo "orig wc -l"; wc -l ${f}	
#	- I discovered there are malformed newlines, and carriage returns in our data... it's a mess. let's fix it!
        cat ${f} | tr -d '\n\r' | tee ${f}.string | sed 's/\t/ /g ; s/\xEF\xBB\xBF/ /g ;s/HMRC,HMRC/\nHMRC,HMRC/g ' > ${f}.newlines
        echo "post wc -l:" ; wc -l ${f}.newlines
#	- convert them to pipe delimited files
	cat ${f}.newlines | sed 's/|/;/g' |csvformat -D \| | gawk -F"|" 'NF <15 {print$0}' | tee ${f}.pip | bytefreq > ${f}.newlines.pip.freq
	head ${f}.newlines.pip.freq

	cat ${f}.pip  | bytefreq -E 

done

#		- test that each file has the right number of fields
#		- if there are bad newlines and carriage returns as the byte frequencies suggest, we can fix them, using a trick.
#		- finally push all the clean, pipe delimited data through a scrubber to remove all the weird characters.

#		- finally with the parsed data, create a Data Profiling report for each file, and for all data concatenated together. double check it all looks good.

# then finally move the cleaned file to a staging directory, called stg so they are ready for analytics.
# we will create a simple report to demonstrate the process works.







# Then lets doublecheck the data, strip out bad line endings, and use a marker in the content to rebuild them properly.
# Then with this done we move to profiling the content:

# 1) use csvfix to correct the data, hopefully it's all good!




# then use csvformat -D \| to convert it to pipe delimited.
# then we can use bytefreq to understand the columns in the data, and if there is data quality issues.





